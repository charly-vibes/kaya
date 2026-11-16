//! The generated experiment report projection.
//!
//! Purpose: compile one experiment's slice of the research graph into a
//! deterministic Markdown section that stays in sync with the ledger.
//! Responsibilities: render question, hypothesis, runs, observations,
//! findings, and promotion state; replace only the `experiment-report`
//! generated block; report authored blocks whose cited evidence moved.
//! Rationale: the report is a compilation artifact — same inputs, same
//! bytes; human prose survives regeneration untouched; interpretation is
//! shown with its promotion status so a proposal never reads as a verdict.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::Path;

use crate::model::{EntityType, Graph, Relationship};
use crate::projection::{BlockKind, ProjectionError, StaleBlock, Summary, replace_generated, scan};
use crate::promotion::PromotionState;

/// Name of the generated block owned by this projection.
pub const REPORT_BLOCK: &str = "experiment-report";

const AUTHORED_BEGIN: &str = "<!-- kaya:authored -->";
const AUTHORED_END: &str = "<!-- kaya:end authored -->";
const CITES_PREFIX: &str = "<!-- kaya:cites";
const MARKER_END: &str = "-->";

const LABEL_EXPERIMENT: &str = "kaya.projection.report_experiment";
const LABEL_IO: &str = "kaya.projection.io";

/// Build the experiment report at `path`.
///
/// Reads the document when present, renders the deterministic
/// `experiment-report` body, and rewrites only that generated block. An
/// unchanged body means no write: the summary records the projection under
/// `experiment-report:<experiment-id>`; a changed body is written and
/// recorded under the same name. Namespacing keeps reports for different
/// experiments distinct when summaries merge. The source map traces every
/// rendered node and interpretation back to its node or event ids.
pub fn build(
    graph: &Graph,
    experiment_id: &str,
    path: &Path,
    promotion: &PromotionState,
) -> Result<Summary, ProjectionError> {
    if experiment_id.trim().is_empty() {
        return Err(ProjectionError::new(
            LABEL_EXPERIMENT,
            "an experiment report requires a non-empty experiment id",
        ));
    }

    let rendered = render(graph, experiment_id, promotion);
    let name = summary_name(experiment_id);
    let mut summary = Summary::default();
    summary.sources.insert(name.clone(), rendered.sources);

    match read_existing(path)? {
        None => {
            let document = replace_generated("", REPORT_BLOCK, &rendered.body)?;
            write(path, &document)?;
            summary.recomputed.push(name);
        }
        Some(document) => {
            if generated_body(&document)?.as_deref() == Some(rendered.body.as_str()) {
                summary.unchanged.push(name);
            } else {
                let updated = replace_generated(&document, REPORT_BLOCK, &rendered.body)?;
                write(path, &updated)?;
                summary.recomputed.push(name);
            }
        }
    }
    Ok(summary)
}

/// Summary entry naming one experiment's report block.
fn summary_name(experiment_id: &str) -> String {
    format!("{REPORT_BLOCK}:{experiment_id}")
}

fn generated_body(document: &str) -> Result<Option<String>, ProjectionError> {
    let blocks = scan(document)?;
    Ok(blocks.into_iter().find_map(|block| match block.kind {
        BlockKind::Generated(name) if name.as_str() == REPORT_BLOCK => Some(block.body),
        _ => None,
    }))
}

fn read_existing(path: &Path) -> Result<Option<String>, ProjectionError> {
    match fs::read_to_string(path) {
        Ok(document) => Ok(Some(document)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(io_error("read", path, error)),
    }
}

fn write(path: &Path, document: &str) -> Result<(), ProjectionError> {
    fs::write(path, document).map_err(|error| io_error("write", path, error))
}

fn io_error(action: &str, path: &Path, error: io::Error) -> ProjectionError {
    ProjectionError::new(
        LABEL_IO,
        format!("failed to {action} \"{}\": {error}", path.display()),
    )
}

/// Flag authored blocks whose cited evidence changed after acknowledgment.
///
/// Authored blocks are numbered in document order as `authored-1`,
/// `authored-2`, … and may contain `<!-- kaya:cites id1 id2 -->` markers.
/// A block is stale when a cited id appears in `changed_events` and that
/// block's acknowledgment entry does not cover exactly the changed cited
/// ids. The returned evidence is the sorted, de-duplicated changed ids. The
/// document is never modified.
pub fn stale_authored(
    doc: &str,
    changed_events: &[String],
    acknowledged: &BTreeMap<String, Vec<String>>,
) -> Vec<StaleBlock> {
    let changed: BTreeSet<&str> = changed_events
        .iter()
        .map(String::as_str)
        .filter(|id| !id.is_empty())
        .collect();
    let mut stale = Vec::new();

    for (name, body) in authored_blocks(doc) {
        let cited = cited_ids(&body);
        let evidence: Vec<String> = cited
            .iter()
            .filter(|id| changed.contains(id.as_str()))
            .cloned()
            .collect();
        if evidence.is_empty() {
            continue;
        }
        let acknowledged_ids: BTreeSet<&str> = acknowledged
            .get(&name)
            .map(|ids| {
                ids.iter()
                    .map(String::as_str)
                    .filter(|id| !id.is_empty())
                    .collect()
            })
            .unwrap_or_default();
        let changed_cited: BTreeSet<&str> = evidence.iter().map(String::as_str).collect();
        if acknowledged_ids == changed_cited {
            continue;
        }
        stale.push(StaleBlock {
            block: name,
            evidence,
        });
    }
    stale
}

struct Rendered {
    body: String,
    sources: Vec<String>,
}

fn render(graph: &Graph, experiment_id: &str, promotion: &PromotionState) -> Rendered {
    let anchors: BTreeSet<String> = [experiment_id.to_string()].into_iter().collect();
    let questions = linked(
        graph,
        &[Relationship::Investigates, Relationship::Predicts],
        &anchors,
        EntityType::Question,
    );
    let hypotheses = linked(
        graph,
        &[Relationship::Predicts],
        &anchors,
        EntityType::Hypothesis,
    );
    let runs = linked(graph, &[Relationship::Executes], &anchors, EntityType::Run);
    let observations = linked(
        graph,
        &[Relationship::Observes],
        &runs,
        EntityType::Observation,
    );
    let findings = linked(
        graph,
        &[Relationship::Derives],
        &observations,
        EntityType::Finding,
    );

    let mut body = String::new();
    push_section(&mut body, "Question", &id_lines(&questions));
    push_section(&mut body, "Hypothesis", &id_lines(&hypotheses));
    push_section(&mut body, "Runs", &id_lines(&runs));
    push_section(&mut body, "Observations", &id_lines(&observations));
    push_section(&mut body, "Findings", &id_lines(&findings));

    let mut assertions = promotion.assertions.clone();
    assertions.sort_by(|left, right| left.id.cmp(&right.id));
    let mut interpretation_lines = Vec::with_capacity(assertions.len());
    let mut sources = BTreeSet::new();
    for assertion in &assertions {
        let cites = if assertion.event_ids.is_empty() {
            "_(none)_".to_string()
        } else {
            assertion.event_ids.join(", ")
        };
        interpretation_lines.push(format!(
            "- `{}` {} — machine-proposed: {}; cites: {}; author: {}",
            assertion.status.as_str(),
            assertion.text,
            if assertion.machine_proposed {
                "yes"
            } else {
                "no"
            },
            cites,
            assertion.author,
        ));
        sources.insert(assertion.id.clone());
        sources.extend(assertion.event_ids.iter().cloned());
    }
    push_section(&mut body, "Interpretations", &interpretation_lines);

    let mut all_sources = BTreeSet::new();
    for group in [&questions, &hypotheses, &runs, &observations, &findings] {
        all_sources.extend(group.iter().cloned());
    }
    all_sources.extend(sources);

    Rendered {
        body,
        sources: all_sources.into_iter().collect(),
    }
}

fn push_section(body: &mut String, title: &str, lines: &[String]) {
    if !body.is_empty() {
        body.push('\n');
    }
    body.push_str("## ");
    body.push_str(title);
    body.push_str("\n\n");
    if lines.is_empty() {
        body.push_str("- _(none)_\n");
        return;
    }
    for line in lines {
        body.push_str(line);
        body.push('\n');
    }
}

fn id_lines(ids: &BTreeSet<String>) -> Vec<String> {
    ids.iter().map(|id| format!("- {id}")).collect()
}

/// Nodes of `entity` joined to any anchor through one of `relationships`.
///
/// Relationship direction is not assumed here: the model already validates
/// direction, and reports should render the edge whichever endpoint names
/// the anchor. Results are sorted and de-duplicated.
fn linked(
    graph: &Graph,
    relationships: &[Relationship],
    anchors: &BTreeSet<String>,
    entity: EntityType,
) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    for edge in graph.edges() {
        if !relationships.contains(&edge.relationship) {
            continue;
        }
        let other = if anchors.contains(&edge.from) {
            Some(&edge.to)
        } else if anchors.contains(&edge.to) {
            Some(&edge.from)
        } else {
            None
        };
        if let Some(id) = other
            && graph.node(id).is_some_and(|node| node.entity == entity)
        {
            found.insert(id.clone());
        }
    }
    found
}

/// Authored block bodies with their ordinal names, in document order.
fn authored_blocks(doc: &str) -> Vec<(String, String)> {
    let mut blocks = Vec::new();
    let mut cursor = 0usize;
    let mut ordinal = 0usize;
    while let Some(found) = doc[cursor..].find(AUTHORED_BEGIN) {
        let begin = cursor + found;
        let body_start = begin + AUTHORED_BEGIN.len();
        let Some(relative_end) = doc[body_start..].find(AUTHORED_END) else {
            break;
        };
        let body_end = body_start + relative_end;
        ordinal += 1;
        blocks.push((
            format!("authored-{ordinal}"),
            doc[body_start..body_end].to_string(),
        ));
        cursor = body_end + AUTHORED_END.len();
    }
    blocks
}

/// Citation ids collected from every well-formed `kaya:cites` marker.
fn cited_ids(body: &str) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    let mut cursor = 0usize;
    while let Some(found) = body[cursor..].find(CITES_PREFIX) {
        let start = cursor + found + CITES_PREFIX.len();
        let Some(relative_end) = body[start..].find(MARKER_END) else {
            break;
        };
        let end = start + relative_end;
        for id in body[start..end].split_whitespace() {
            if !id.is_empty() {
                ids.insert(id.to_string());
            }
        }
        cursor = end + MARKER_END.len();
    }
    ids
}
