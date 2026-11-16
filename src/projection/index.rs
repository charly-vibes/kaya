//! The deterministic research index projection.
//!
//! [`build`] renders every Question, Hypothesis, Experiment, Finding and
//! Decision node as one Markdown line, then appends Artifact and Run
//! sections from their attributes, inside the `research-index` generated
//! block. Only that block's bytes move: authored sections, foreign managed
//! blocks, and uncategorized prose are preserved byte for byte. A rebuild
//! whose rendered body already matches reports the block `unchanged` and
//! leaves the file untouched.

use std::io::ErrorKind;
use std::path::Path;

use serde_json::Value;

use crate::model::{EntityType, Graph, Node};

use super::{BlockKind, ProjectionError, Summary, replace_generated, scan};

/// Name of the generated block this projection owns.
pub const INDEX_BLOCK: &str = "research-index";

const INDEX_ENTITIES: [EntityType; 5] = [
    EntityType::Question,
    EntityType::Hypothesis,
    EntityType::Experiment,
    EntityType::Finding,
    EntityType::Decision,
];

/// Build the research index at `path` from `graph`.
///
/// The file is created when missing. When the rendered `research-index`
/// block already equals the file's block, nothing is written and the block
/// is reported in [`Summary::unchanged`]; otherwise the block is replaced
/// and reported in [`Summary::recomputed`]. [`Summary::sources`] maps the
/// block name to every rendered node id. The trailing summary line reports
/// `event_count` events, the number of ledger records the graph was built
/// from.
pub fn build(graph: &Graph, path: &Path, event_count: usize) -> Result<Summary, ProjectionError> {
    let (rendered, sources) = render(graph, event_count);
    let mut summary = Summary::default();
    summary.sources.insert(INDEX_BLOCK.to_owned(), sources);

    match read_existing(path)? {
        None => {
            let document = replace_generated("", INDEX_BLOCK, &rendered)?;
            write(path, &document)?;
            summary.recomputed.push(INDEX_BLOCK.to_owned());
        }
        Some(document) => {
            if generated_body(&document)?.as_deref() == Some(rendered.as_str()) {
                summary.unchanged.push(INDEX_BLOCK.to_owned());
            } else {
                let updated = replace_generated(&document, INDEX_BLOCK, &rendered)?;
                write(path, &updated)?;
                summary.recomputed.push(INDEX_BLOCK.to_owned());
            }
        }
    }

    Ok(summary)
}

/// Render the generated body and collect the node ids backing its lines.
fn render(graph: &Graph, event_count: usize) -> (String, Vec<String>) {
    let mut body = String::new();
    let mut sources = Vec::new();
    let mut rendered_nodes = 0usize;

    for node in graph.nodes() {
        if !INDEX_ENTITIES.contains(&node.entity) {
            continue;
        }
        body.push_str("- ");
        body.push_str(&node.id);
        body.push_str(" — ");
        body.push_str(node.entity.as_str());
        body.push_str(" — ");
        body.push_str(&title_or_key(node));
        body.push('\n');
        sources.push(node.id.clone());
        rendered_nodes += 1;
    }

    let artifacts = artifact_section(graph);
    if !artifacts.is_empty() {
        body.push('\n');
        body.push_str("### Artifacts\n");
        body.push_str(&artifacts);
        for node in graph
            .nodes()
            .filter(|node| node.entity == EntityType::Artifact)
        {
            sources.push(node.id.clone());
            rendered_nodes += 1;
        }
    }

    let runs = run_section(graph);
    if !runs.is_empty() {
        body.push('\n');
        body.push_str("### Runs\n");
        body.push_str(&runs);
        for node in graph.nodes().filter(|node| node.entity == EntityType::Run) {
            sources.push(node.id.clone());
            rendered_nodes += 1;
        }
    }

    body.push('\n');
    body.push_str(&format!(
        "_{rendered_nodes} nodes from {event_count} events._\n"
    ));

    sources.sort();
    sources.dedup();
    (body, sources)
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

fn title_or_key(node: &Node) -> String {
    attribute(node, "title").unwrap_or_else(|| key(node))
}

fn artifact_label(node: &Node) -> String {
    attribute(node, "path")
        .or_else(|| attribute(node, "sha"))
        .unwrap_or_else(|| key(node))
}

fn attribute(node: &Node, name: &str) -> Option<String> {
    let value = node.attributes.get(name).and_then(Value::as_str)?;
    let single_line: String = value
        .chars()
        .map(|c| if c == '\n' || c == '\r' { ' ' } else { c })
        .collect();
    let trimmed = single_line.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_owned())
    }
}

fn count(node: &Node, name: &str) -> String {
    node.attributes
        .get(name)
        .and_then(Value::as_u64)
        .unwrap_or(0)
        .to_string()
}

fn key(node: &Node) -> String {
    node.id
        .rsplit('/')
        .next()
        .unwrap_or(node.id.as_str())
        .to_owned()
}

fn generated_body(document: &str) -> Result<Option<String>, ProjectionError> {
    let blocks = scan(document)?;
    Ok(blocks.into_iter().find_map(|block| match block.kind {
        BlockKind::Generated(name) if name.as_str() == INDEX_BLOCK => Some(block.body),
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
