//! The deterministic research map projection.
//!
//! [`build`] renders every declared relationship as one typed edge, every
//! admitted entity without an incident edge as one isolated row (except the
//! Artifact, Run, and Source nodes carried by the attribute detail
//! sections), then appends those Artifact, Run, and Source detail sections,
//! inside the `orientation.research-map` generated block. Only that block's bytes move:
//! authored sections, foreign managed blocks, and uncategorized prose are
//! preserved byte for byte. A rebuild whose rendered body already matches
//! reports the block `unchanged` and leaves the file untouched.

use std::collections::BTreeSet;
use std::io::ErrorKind;
use std::path::Path;

use serde_json::Value;

use crate::model::{EntityType, Graph, Node};

use super::{
    BlockKind, ProjectionError, Summary, attribute, key, replace_generated, scan, source_reference,
    title_or_key, validate_generated_blocks,
};

/// Name of the generated block this projection owns.
pub const RESEARCH_MAP_BLOCK: &str = "orientation.research-map";

/// The result of the index projection: bookkeeping plus the document IR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Built {
    /// Recomputed, unchanged, and source-map bookkeeping.
    pub summary: Summary,
    /// The document IR materialized from the planned document.
    pub ir: crate::projection::ir::DocumentIr,
}

/// Build the research map at `path` from `graph`.
///
/// The existing document is validated against the closed generated-block
/// vocabulary before any write (rendering is pure), so a legacy block is
/// refused with the file untouched. The file is created when missing. When
/// the rendered `orientation.research-map` block already equals the file's
/// block, nothing is written and the block is reported in
/// [`Summary::unchanged`]; otherwise the block is replaced and reported in
/// [`Summary::recomputed`]. [`Summary::sources`] maps the block name to every
/// rendered node id. The trailing summary line reports `event_count` events,
/// the number of ledger records the graph was built from.
pub fn build(graph: &Graph, path: &Path, event_count: usize) -> Result<Built, ProjectionError> {
    let existing = read_existing(path)?;
    let (document, mut summary) = plan(graph, path, existing.as_deref(), event_count)?;
    if existing.as_deref() != Some(document.as_str()) {
        write(path, &document)?;
    }
    let ir = crate::projection::ir::for_document(path, &document, &mut summary);
    Ok(Built { summary, ir })
}

/// The bytes and bookkeeping `build` would commit, without touching the disk.
///
/// Planning is the whole projection: the returned document is exactly what
/// `build` writes when it differs from `existing`, so `kaya check` can compare
/// the expected bytes against the worktree without a second renderer. `path`
/// names the document being planned and is accepted for symmetry with the
/// writer; planning itself never reads or writes it.
pub fn plan(
    graph: &Graph,
    _path: &Path,
    existing: Option<&str>,
    event_count: usize,
) -> Result<(String, Summary), ProjectionError> {
    let (rendered, sources) = render(graph, event_count);
    let mut summary = Summary::default();
    summary
        .sources
        .insert(RESEARCH_MAP_BLOCK.to_owned(), sources);

    let document = match existing {
        None => {
            summary.recomputed.push(RESEARCH_MAP_BLOCK.to_owned());
            replace_generated("", RESEARCH_MAP_BLOCK, &rendered)?
        }
        Some(document) => {
            validate_generated_blocks(document)?;
            if generated_body(document)?.as_deref() == Some(rendered.as_str()) {
                summary.unchanged.push(RESEARCH_MAP_BLOCK.to_owned());
                document.to_owned()
            } else {
                summary.recomputed.push(RESEARCH_MAP_BLOCK.to_owned());
                replace_generated(document, RESEARCH_MAP_BLOCK, &rendered)?
            }
        }
    };

    Ok((document, summary))
}

/// Render the generated body and collect the node ids backing its rows.
fn render(graph: &Graph, event_count: usize) -> (String, Vec<String>) {
    crate::projection::bump_render_calls();
    let mut body = String::new();
    let mut sources = BTreeSet::new();

    let relationships = relationship_section(graph, &mut sources);
    if !relationships.is_empty() {
        body.push_str("### Relationships\n");
        body.push_str(&relationships);
    }

    let isolated = isolated_section(graph, &mut sources);
    if !isolated.is_empty() {
        if !body.is_empty() {
            body.push('\n');
        }
        body.push_str("### Isolated\n");
        body.push_str(&isolated);
    }

    let artifacts = artifact_section(graph);
    if !artifacts.is_empty() {
        body.push('\n');
        body.push_str("### Artifacts\n");
        body.push_str(&artifacts);
    }

    let runs = run_section(graph);
    if !runs.is_empty() {
        body.push('\n');
        body.push_str("### Runs\n");
        body.push_str(&runs);
    }

    let cited = source_section(graph);
    if !cited.is_empty() {
        body.push('\n');
        body.push_str("### Sources\n");
        body.push_str(&cited);
    }

    // The source map traces rendered rows: an Artifact or Run always renders
    // one, a Source only when it carries a full `path@commit`.
    for node in graph.nodes() {
        let rendered = match node.entity {
            EntityType::Artifact | EntityType::Run => true,
            EntityType::Source => source_reference(node).is_some(),
            _ => false,
        };
        if rendered {
            sources.insert(node.id.clone());
        }
    }

    body.push('\n');
    body.push_str(&format!(
        "_{} nodes from {event_count} events._\n",
        sources.len()
    ));

    (body, sources.into_iter().collect())
}

/// Render every declared edge exactly once, sorted by `(from, relationship, to)`.
fn relationship_section(graph: &Graph, sources: &mut BTreeSet<String>) -> String {
    let mut edges: Vec<(&str, &str, &str)> = graph
        .edges()
        .map(|edge| {
            (
                edge.from.as_str(),
                edge.relationship.as_str(),
                edge.to.as_str(),
            )
        })
        .collect();
    edges.sort_unstable();
    edges.dedup();

    let mut section = String::new();
    for (from, relationship, to) in edges {
        section.push_str("- ");
        section.push_str(from);
        section.push_str(" -[");
        section.push_str(relationship);
        section.push_str("]-> ");
        section.push_str(to);
        section.push('\n');
        sources.insert(from.to_owned());
        sources.insert(to.to_owned());
    }
    section
}

/// Render admitted entities with no incident edge, sorted by id.
///
/// Artifact, Run, and Source nodes are skipped: the attribute detail sections
/// below already carry those ids, so rendering them here would list them
/// twice.
fn isolated_section(graph: &Graph, sources: &mut BTreeSet<String>) -> String {
    let mut incident: BTreeSet<&str> = BTreeSet::new();
    for edge in graph.edges() {
        incident.insert(edge.from.as_str());
        incident.insert(edge.to.as_str());
    }

    let mut section = String::new();
    for node in graph.nodes() {
        if incident.contains(node.id.as_str())
            || matches!(
                node.entity,
                EntityType::Artifact | EntityType::Run | EntityType::Source
            )
        {
            continue;
        }
        section.push_str("- ");
        section.push_str(&node.id);
        section.push_str(" — ");
        section.push_str(node.entity.as_str());
        section.push_str(" — ");
        section.push_str(&title_or_key(node));
        section.push('\n');
        sources.insert(node.id.clone());
    }
    section
}

fn artifact_section(graph: &Graph) -> String {
    let mut section = String::new();
    for node in graph
        .nodes()
        .filter(|node| node.entity == EntityType::Artifact)
    {
        section.push_str("- ");
        section.push_str(&node.id);
        section.push_str(" — ");
        section.push_str(&artifact_label(node));
        section.push('\n');
    }
    section
}

fn run_section(graph: &Graph) -> String {
    let mut section = String::new();
    for node in graph.nodes().filter(|node| node.entity == EntityType::Run) {
        section.push_str("- ");
        section.push_str(&node.id);
        section.push_str(" — passed ");
        section.push_str(&count(node, "passed"));
        section.push_str(", failed ");
        section.push_str(&count(node, "failed"));
        section.push_str(", ignored ");
        section.push_str(&count(node, "ignored"));
        section.push('\n');
    }
    section
}

/// Render repo files as `path@commit`, pinned to the recorded commit.
///
/// A Source node missing or blank in `path` or `commit` has no resolvable
/// reference, so it renders no row rather than a bare `@`.
fn source_section(graph: &Graph) -> String {
    let mut section = String::new();
    for node in graph
        .nodes()
        .filter(|node| node.entity == EntityType::Source)
    {
        let Some(reference) = source_reference(node) else {
            continue;
        };
        section.push_str("- ");
        section.push_str(&node.id);
        section.push_str(" — ");
        section.push_str(&reference);
        section.push('\n');
    }
    section
}

fn artifact_label(node: &Node) -> String {
    attribute(node, "path")
        .or_else(|| attribute(node, "sha"))
        .unwrap_or_else(|| key(node))
}

fn count(node: &Node, name: &str) -> String {
    node.attributes
        .get(name)
        .and_then(Value::as_u64)
        .unwrap_or(0)
        .to_string()
}

fn generated_body(document: &str) -> Result<Option<String>, ProjectionError> {
    let blocks = scan(document)?;
    Ok(blocks.into_iter().find_map(|block| match block.kind {
        BlockKind::Generated(name) if name.as_str() == RESEARCH_MAP_BLOCK => Some(block.body),
        _ => None,
    }))
}

fn read_existing(path: &Path) -> Result<Option<String>, ProjectionError> {
    match std::fs::read_to_string(path) {
        Ok(document) => Ok(Some(document)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(io_error("read", path, error)),
    }
}

fn write(path: &Path, document: &str) -> Result<(), ProjectionError> {
    std::fs::write(path, document).map_err(|error| io_error("write", path, error))
}

fn io_error(action: &str, path: &Path, error: std::io::Error) -> ProjectionError {
    ProjectionError::new(
        "kaya.projection.io",
        format!("failed to {action} \"{}\": {error}", path.display()),
    )
}
