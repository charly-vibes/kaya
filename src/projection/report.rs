//! The generated experiment report projection.
//!
//! Purpose: compile one experiment's slice of the research graph into
//! deterministic Markdown blocks that stay in sync with the ledger.
//! Responsibilities: classify document blocks against the closed family/kind
//! vocabulary, split declared intent (`experiment-specification`) from
//! observed execution (`experiment-record`), replace only those generated
//! blocks, report empty kinds as omitted, and flag authored blocks whose
//! cited evidence moved.
//! Rationale: the report is a compilation artifact — same inputs, same
//! bytes; human prose survives regeneration untouched; interpretation is
//! shown with its promotion status so a proposal never reads as a verdict.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::model::{EntityType, Graph, Relationship};
use crate::projection::{
    ProjectionError, StaleBlock, Summary, generated_body, read_existing, remove_generated,
    replace_generated, validate_generated_blocks, write,
};
use crate::promotion::PromotionState;

/// Generated block carrying an experiment's declared intent.
pub const SPECIFICATION_BLOCK: &str = "exploration.experiment-specification";
/// Generated block carrying an experiment's observed execution.
pub const RECORD_BLOCK: &str = "exploration.experiment-record";
/// Reason reported for a family-kind with no source records.
pub const REASON_NO_RECORDS: &str = "no-records";
/// Label refusing a family or kind outside the closed vocabulary.
pub const LABEL_UNKNOWN_KIND: &str = "kaya.documents.unknown_kind_refused";
/// Label refusing a generated block outside the closed block vocabulary.
pub const LABEL_LEGACY_BLOCK: &str = "kaya.documents.legacy_block_refused";

const AUTHORED_BEGIN: &str = "<!-- kaya:authored -->";
const AUTHORED_END: &str = "<!-- kaya:end authored -->";
const CITES_PREFIX: &str = "<!-- kaya:cites";
const MARKER_END: &str = "-->";

const LABEL_EXPERIMENT: &str = "kaya.projection.report_experiment";

/// One document family in the closed taxonomy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Family {
    Orientation,
    Exploration,
    Understanding,
    ReuseHandoff,
}

impl Family {
    /// Every family, in canonical order.
    pub const ALL: [Family; 4] = [
        Family::Orientation,
        Family::Exploration,
        Family::Understanding,
        Family::ReuseHandoff,
    ];

    /// The family's vocabulary name.
    pub fn as_str(self) -> &'static str {
        match self {
            Family::Orientation => "orientation",
            Family::Exploration => "exploration",
            Family::Understanding => "understanding",
            Family::ReuseHandoff => "reuse-handoff",
        }
    }

    /// The kinds admitted in this family.
    pub fn kinds(self) -> &'static [Kind] {
        match self {
            Family::Orientation => &[Kind::ProjectOverview, Kind::ResearchMap],
            Family::Exploration => &[
                Kind::ExplorationBrief,
                Kind::ExperimentSpecification,
                Kind::ExperimentRecord,
            ],
            Family::Understanding => &[Kind::FindingsAndAnalysis, Kind::DecisionRecord],
            Family::ReuseHandoff => &[
                Kind::TechnicalReference,
                Kind::ReproductionPackage,
                Kind::KnowledgeBrief,
                Kind::Handoff,
            ],
        }
    }

    /// Parse a family vocabulary name.
    pub fn parse(value: &str) -> Option<Family> {
        Family::ALL
            .into_iter()
            .find(|family| family.as_str() == value)
    }
}

/// One document kind in the closed taxonomy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    ProjectOverview,
    ResearchMap,
    ExplorationBrief,
    ExperimentSpecification,
    ExperimentRecord,
    FindingsAndAnalysis,
    DecisionRecord,
    TechnicalReference,
    ReproductionPackage,
    KnowledgeBrief,
    Handoff,
}

impl Kind {
    /// Every kind, in canonical (block vocabulary) order.
    pub const ALL: [Kind; 11] = [
        Kind::ProjectOverview,
        Kind::ResearchMap,
        Kind::ExplorationBrief,
        Kind::ExperimentSpecification,
        Kind::ExperimentRecord,
        Kind::FindingsAndAnalysis,
        Kind::DecisionRecord,
        Kind::TechnicalReference,
        Kind::ReproductionPackage,
        Kind::KnowledgeBrief,
        Kind::Handoff,
    ];

    /// The family admitting this kind.
    pub fn family(self) -> Family {
        match self {
            Kind::ProjectOverview | Kind::ResearchMap => Family::Orientation,
            Kind::ExplorationBrief | Kind::ExperimentSpecification | Kind::ExperimentRecord => {
                Family::Exploration
            }
            Kind::FindingsAndAnalysis | Kind::DecisionRecord => Family::Understanding,
            Kind::TechnicalReference
            | Kind::ReproductionPackage
            | Kind::KnowledgeBrief
            | Kind::Handoff => Family::ReuseHandoff,
        }
    }

    /// The kind's bare vocabulary name.
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::ProjectOverview => "project-overview",
            Kind::ResearchMap => "research-map",
            Kind::ExplorationBrief => "exploration-brief",
            Kind::ExperimentSpecification => "experiment-specification",
            Kind::ExperimentRecord => "experiment-record",
            Kind::FindingsAndAnalysis => "findings-and-analysis",
            Kind::DecisionRecord => "decision-record",
            Kind::TechnicalReference => "technical-reference",
            Kind::ReproductionPackage => "reproduction-package",
            Kind::KnowledgeBrief => "knowledge-brief",
            Kind::Handoff => "handoff",
        }
    }

    /// The generated block name, `<family>.<kind>`.
    pub fn block_name(self) -> String {
        format!("{}.{}", self.family().as_str(), self.as_str())
    }

    /// Parse a bare kind vocabulary name.
    pub fn parse(value: &str) -> Option<Kind> {
        Kind::ALL.into_iter().find(|kind| kind.as_str() == value)
    }
}

/// The closed block vocabulary: every `family.kind` name, in canonical order.
pub fn block_vocabulary() -> String {
    Kind::ALL
        .iter()
        .map(|kind| kind.block_name())
        .collect::<Vec<_>>()
        .join(", ")
}

/// Refuse a family or kind outside the closed vocabulary.
pub fn refuse_kind(value: &str) -> ProjectionError {
    ProjectionError::new(
        LABEL_UNKNOWN_KIND,
        format!(
            "unknown document family or kind \"{value}\"; the closed families are: {}; \
             the closed block vocabulary is: {}",
            family_vocabulary(),
            block_vocabulary()
        ),
    )
}

/// The closed families, in canonical order.
fn family_vocabulary() -> String {
    Family::ALL
        .iter()
        .map(|family| family.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

/// Classify a generated block name as a kind in the closed vocabulary.
///
/// Only `<family>.<kind>` is admitted, and the kind must belong to the named
/// family. Anything else — a legacy block name, an unknown family, or a
/// mismatched pairing — is refused before any document is written.
pub fn classify_block(name: &str) -> Result<Kind, ProjectionError> {
    let refusal = || {
        ProjectionError::new(
            LABEL_LEGACY_BLOCK,
            format!(
                "generated block \"{name}\" is outside the closed block vocabulary: {}",
                block_vocabulary()
            ),
        )
    };

    let Some((family_name, kind_name)) = name.split_once('.') else {
        return Err(refusal());
    };
    let Some(family) = Family::parse(family_name) else {
        return Err(refusal());
    };
    let Some(kind) = Kind::parse(kind_name) else {
        return Err(refusal());
    };
    if kind.family() != family {
        return Err(refusal());
    }
    Ok(kind)
}

/// A family-kind omitted from a projection because it has no source records.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Omitted {
    /// Family vocabulary name.
    pub family: &'static str,
    /// Kind vocabulary name.
    pub kind: &'static str,
    /// Why the kind was omitted.
    pub reason: &'static str,
}

/// The result of building one experiment's report: the merged summary plus
/// the family-kinds that produced no block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Built {
    /// Recomputed, unchanged, stale, and source-map bookkeeping.
    pub summary: Summary,
    /// Family-kinds with no source records, in block order.
    pub omitted: Vec<Omitted>,
}

/// Build the experiment report at `path`.
///
/// Validates every generated block already in the document against the closed
/// vocabulary before rendering anything: a legacy or unknown block name is
/// refused without a write. It then renders the specification block from
/// declared intent and the record block from observed execution, replacing or
/// appending each present block and removing a block whose kind no longer has
/// rows. Namespacing keeps reports for different experiments distinct when
/// summaries merge. The source map traces every rendered node and
/// interpretation back to its node or event ids.
pub fn build(
    graph: &Graph,
    experiment_id: &str,
    path: &Path,
    promotion: &PromotionState,
) -> Result<Built, ProjectionError> {
    if experiment_id.trim().is_empty() {
        return Err(ProjectionError::new(
            LABEL_EXPERIMENT,
            "an experiment report requires a non-empty experiment id",
        ));
    }

    let mut document = read_existing(path)?.unwrap_or_default();
    if !document.is_empty() {
        validate_generated_blocks(&document)?;
    }

    let anchors: BTreeSet<String> = [experiment_id.to_string()].into_iter().collect();
    let specification = render_specification(graph, &anchors);
    let record = render_record(graph, &anchors, promotion);

    let mut summary = Summary::default();
    let mut omitted = Vec::new();
    let mut changed = false;

    // Kind order is part of the document contract: the specification always
    // precedes the record. When the specification block is newly declared but
    // the document already holds a record block, drop the record here so the
    // kind loop re-appends it after the specification instead of leaving the
    // order to build history.
    let specification_name = Kind::ExperimentSpecification.block_name();
    let record_name = Kind::ExperimentRecord.block_name();
    if specification.is_some()
        && generated_body(&document, &specification_name)?.is_none()
        && generated_body(&document, &record_name)?.is_some()
    {
        document = remove_generated(&document, &record_name)?;
        changed = true;
    }

    for (kind, rendered) in [
        (Kind::ExperimentSpecification, specification),
        (Kind::ExperimentRecord, record),
    ] {
        let name = kind.block_name();
        let key = format!("{name}:{experiment_id}");
        match rendered {
            Some(rendered) => {
                if generated_body(&document, &name)?.as_deref() == Some(rendered.body.as_str()) {
                    summary.unchanged.push(key.clone());
                } else {
                    document = replace_generated(&document, &name, &rendered.body)?;
                    summary.recomputed.push(key.clone());
                    changed = true;
                }
                summary.sources.insert(key, rendered.sources);
            }
            None => {
                omitted.push(Omitted {
                    family: kind.family().as_str(),
                    kind: kind.as_str(),
                    reason: REASON_NO_RECORDS,
                });
                if generated_body(&document, &name)?.is_some() {
                    document = remove_generated(&document, &name)?;
                    summary.recomputed.push(key);
                    changed = true;
                }
            }
        }
    }

    if changed {
        write(path, &document)?;
    }

    Ok(Built { summary, omitted })
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

/// One non-empty rendered block: its body and the ids backing it.
struct RenderedBlock {
    body: String,
    sources: Vec<String>,
}

/// Declared intent only: the questions investigated and hypotheses predicted.
fn render_specification(graph: &Graph, anchors: &BTreeSet<String>) -> Option<RenderedBlock> {
    let questions = linked(
        graph,
        &[Relationship::Investigates],
        anchors,
        EntityType::Question,
    );
    let hypotheses = linked(
        graph,
        &[Relationship::Predicts],
        anchors,
        EntityType::Hypothesis,
    );
    if questions.is_empty() && hypotheses.is_empty() {
        return None;
    }

    let mut body = String::new();
    push_section(&mut body, "Question", &id_lines(&questions));
    push_section(&mut body, "Hypothesis", &id_lines(&hypotheses));

    let mut sources = BTreeSet::new();
    sources.extend(questions);
    sources.extend(hypotheses);

    Some(RenderedBlock {
        body,
        sources: sources.into_iter().collect(),
    })
}

/// Observed execution only: runs, their observations, derived findings, and
/// the promotion state that informs interpretation.
fn render_record(
    graph: &Graph,
    anchors: &BTreeSet<String>,
    promotion: &PromotionState,
) -> Option<RenderedBlock> {
    let runs = linked(graph, &[Relationship::Executes], anchors, EntityType::Run);
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

    // Emptiness is this experiment's observed execution only. Promotion
    // assertions are ledger-global, so they never conjure a record block for
    // an experiment with no runs, observations, or findings; they only enrich
    // a record that renders.
    if runs.is_empty() && observations.is_empty() && findings.is_empty() {
        return None;
    }

    let mut assertions = promotion.assertions.clone();
    assertions.sort_by(|left, right| left.id.cmp(&right.id));
    let mut interpretation_lines = Vec::with_capacity(assertions.len());
    let mut interpretation_sources = BTreeSet::new();
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
        interpretation_sources.insert(assertion.id.clone());
        interpretation_sources.extend(assertion.event_ids.iter().cloned());
    }

    let mut body = String::new();
    push_section(&mut body, "Runs", &id_lines(&runs));
    push_section(&mut body, "Observations", &id_lines(&observations));
    push_section(&mut body, "Findings", &id_lines(&findings));
    push_section(&mut body, "Interpretations", &interpretation_lines);

    let mut sources = BTreeSet::new();
    for group in [&runs, &observations, &findings] {
        sources.extend(group.iter().cloned());
    }
    sources.extend(interpretation_sources);

    Some(RenderedBlock {
        body,
        sources: sources.into_iter().collect(),
    })
}

/// Append one section; an empty group renders no heading and no placeholder.
fn push_section(body: &mut String, title: &str, lines: &[String]) {
    if lines.is_empty() {
        return;
    }
    if !body.is_empty() {
        body.push('\n');
    }
    body.push_str("## ");
    body.push_str(title);
    body.push_str("\n\n");
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
