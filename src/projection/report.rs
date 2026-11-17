//! The generated experiment report projection.
//!
//! Purpose: compile the research graph's understanding and experiment slices
//! into deterministic Markdown blocks that stay in sync with the ledger.
//! Responsibilities: classify document blocks against the closed family/kind
//! vocabulary, split declared intent (`experiment-specification`) from
//! observed execution (`experiment-record`), render the project overview from
//! Project, Question, and Finding records, render findings and decision
//! records with their evidence, replace only those generated blocks, report
//! empty kinds as omitted, and flag authored blocks whose cited evidence
//! moved.
//! Rationale: the report is a compilation artifact — same inputs, same
//! bytes; human prose survives regeneration untouched; interpretation is
//! shown with its promotion status so a proposal never reads as a verdict.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use crate::model::{EntityType, Graph, Relationship};
use crate::projection::{
    ProjectionError, StaleBlock, Summary, generated_body, io_error, read_existing,
    remove_generated, replace_generated, source_reference, title_or_key, validate_generated_blocks,
    write,
};
use crate::promotion::PromotionState;

/// Generated block carrying an experiment's declared intent.
pub const SPECIFICATION_BLOCK: &str = "exploration.experiment-specification";
/// Generated block carrying an experiment's observed execution.
pub const RECORD_BLOCK: &str = "exploration.experiment-record";
/// The project overview block name, as a test-facing alias.
///
/// `build_overview` derives the name from `Kind::ProjectOverview.block_name()`;
/// this constant exists so tests and callers can name the block without
/// constructing a `Kind`.
pub const OVERVIEW_BLOCK: &str = "orientation.project-overview";
/// Generated block carrying findings, observations, and claims.
pub const FINDINGS_BLOCK: &str = "understanding.findings-and-analysis";
/// Generated block carrying decisions and their evidence references.
pub const DECISION_BLOCK: &str = "understanding.decision-record";
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

/// Build the project-level understanding document at `path`.
///
/// Validates every generated block already in the document against the closed
/// vocabulary before rendering anything: a legacy or unknown block name is
/// refused without a write. It then renders the findings block from Finding,
/// Observation, and Claim records and the decision block from Decision records
/// plus their evidence, replacing or appending each present block and removing
/// a block whose kind no longer has records. Kind order is canonical — findings
/// precede the decision record even when that means dropping and re-appending
/// the decision block. A removal that leaves only whitespace deletes the
/// document, and a document with no records and no existing file stays absent.
/// The summary is keyed by block name — one document covers the project, not
/// one per experiment — and the source map traces every rendered id.
pub fn build_understanding(graph: &Graph, path: &Path) -> Result<Built, ProjectionError> {
    let mut summary = Summary::default();
    let mut omitted = Vec::new();
    commit_project_document(
        read_project_document(path)?,
        path,
        vec![
            (Kind::FindingsAndAnalysis, render_findings(graph)),
            (Kind::DecisionRecord, render_decision(graph)),
        ],
        &mut summary,
        &mut omitted,
    )?;
    Ok(Built { summary, omitted })
}

/// Build the project overview at `path`.
///
/// Validates every generated block already in the document against the closed
/// vocabulary before rendering anything: a legacy or unknown block name is
/// refused without a write. It then renders one block from the Project,
/// Question, and Finding records — each in its own section, sections with no
/// record omitted — replacing or appending the block. Runs, artifacts, and
/// every other family stay out of the overview by construction.
///
/// When none of the three groups has a record the kind is omitted once, an
/// existing block is removed, and a document left with only whitespace is
/// deleted rather than written empty — absence is the whole-project
/// representation of "nothing to orient the reader". The summary is keyed by
/// the block name and the source map traces every rendered id.
pub fn build_overview(graph: &Graph, path: &Path) -> Result<Built, ProjectionError> {
    let mut summary = Summary::default();
    let mut omitted = Vec::new();
    commit_project_document(
        read_project_document(path)?,
        path,
        vec![(Kind::ProjectOverview, render_overview(graph))],
        &mut summary,
        &mut omitted,
    )?;
    Ok(Built { summary, omitted })
}

/// Read a project-level document, refusing a block outside the closed
/// vocabulary before a single byte can be rewritten.
fn read_project_document(path: &Path) -> Result<String, ProjectionError> {
    let document = read_existing(path)?.unwrap_or_default();
    if !document.is_empty() {
        validate_generated_blocks(&document)?;
    }
    Ok(document)
}

/// Apply the project-level commit policy: one document, one write, one delete.
///
/// `blocks` names the document's kinds in canonical order. A rendered block
/// replaces or appends its body and records its ids under the block name; an
/// unrendered kind is reported omitted and its obsolete block removed. Kind
/// order is part of the document contract, so a block that is newly declared
/// while a later block already exists drops that later block first: canonical
/// order then never depends on build history. `summary` and `omitted`
/// accumulate across kinds, so the whole document is committed once.
///
/// A document left holding only whitespace is deleted rather than written
/// empty — absence already maps to success, so a concurrent removal is not a
/// failure. A document whose blocks all match is not rewritten at all.
fn commit_project_document(
    mut document: String,
    path: &Path,
    blocks: Vec<(Kind, Option<RenderedBlock>)>,
    summary: &mut Summary,
    omitted: &mut Vec<Omitted>,
) -> Result<(), ProjectionError> {
    let mut changed = false;

    for (index, (kind, rendered)) in blocks.iter().enumerate() {
        if rendered.is_none() || generated_body(&document, &kind.block_name())?.is_some() {
            continue;
        }
        for (later, _) in &blocks[index + 1..] {
            let later_name = later.block_name();
            if generated_body(&document, &later_name)?.is_some() {
                document = remove_generated(&document, &later_name)?;
                changed = true;
            }
        }
    }

    for (kind, rendered) in blocks {
        let name = kind.block_name();
        match rendered {
            Some(rendered) => {
                if generated_body(&document, &name)?.as_deref() == Some(rendered.body.as_str()) {
                    summary.unchanged.push(name.clone());
                } else {
                    document = replace_generated(&document, &name, &rendered.body)?;
                    summary.recomputed.push(name.clone());
                    changed = true;
                }
                summary.sources.insert(name, rendered.sources);
            }
            None => {
                omitted.push(Omitted {
                    family: kind.family().as_str(),
                    kind: kind.as_str(),
                    reason: REASON_NO_RECORDS,
                });
                if generated_body(&document, &name)?.is_some() {
                    document = remove_generated(&document, &name)?;
                    summary.recomputed.push(name);
                    changed = true;
                }
            }
        }
    }

    if document.trim().is_empty() {
        match fs::remove_file(path) {
            Ok(()) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(io_error("remove", path, error)),
        }
    } else if changed {
        write(path, &document)?;
    }

    Ok(())
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

/// `- <id> — <title-or-key>` rows for every node of `entities`, plus their ids.
///
/// `graph.nodes()` is id-ordered, so the rows are id-sorted; sections keep a
/// fixed order of their own, chosen by the caller. The ids come back as one
/// set so the source map is traced without walking the graph twice.
fn entity_rows(graph: &Graph, entities: &[EntityType]) -> (Vec<String>, BTreeSet<String>) {
    let mut rows = Vec::new();
    let mut ids = BTreeSet::new();
    for node in graph.nodes() {
        if !entities.contains(&node.entity) {
            continue;
        }
        rows.push(format!("- {} — {}", node.id, title_or_key(node)));
        ids.insert(node.id.clone());
    }
    (rows, ids)
}

/// The orientation slice only: Project, Question, and Finding records, each in
/// its own section, id-sorted. Runs, artifacts, observations, decisions, and
/// every other entity never enter this block.
fn render_overview(graph: &Graph) -> Option<RenderedBlock> {
    let (projects, project_ids) = entity_rows(graph, &[EntityType::Project]);
    let (questions, question_ids) = entity_rows(graph, &[EntityType::Question]);
    let (findings, finding_ids) = entity_rows(graph, &[EntityType::Finding]);

    if projects.is_empty() && questions.is_empty() && findings.is_empty() {
        return None;
    }

    let mut body = String::new();
    push_heading(&mut body, 3, "Project", &projects);
    push_heading(&mut body, 3, "Questions", &questions);
    push_heading(&mut body, 3, "Findings", &findings);

    let mut sources = project_ids;
    sources.extend(question_ids);
    sources.extend(finding_ids);

    Some(RenderedBlock {
        body,
        sources: sources.into_iter().collect(),
    })
}

/// Findings and their analysis only: Finding, Observation, and Claim records,
/// each in its own section. Runs and artifacts never enter this block.
fn render_findings(graph: &Graph) -> Option<RenderedBlock> {
    let (findings, finding_ids) = entity_rows(graph, &[EntityType::Finding]);
    let (observations, observation_ids) = entity_rows(graph, &[EntityType::Observation]);
    let (claims, claim_ids) = entity_rows(graph, &[EntityType::Claim]);

    if findings.is_empty() && observations.is_empty() && claims.is_empty() {
        return None;
    }

    let mut body = String::new();
    push_heading(&mut body, 3, "Findings", &findings);
    push_heading(&mut body, 3, "Observations", &observations);
    push_heading(&mut body, 3, "Claims", &claims);

    let mut sources = finding_ids;
    sources.extend(observation_ids);
    sources.extend(claim_ids);

    Some(RenderedBlock {
        body,
        sources: sources.into_iter().collect(),
    })
}

/// Decision records plus their evidence: the decisions themselves, the source
/// revisions backing them, and the findings that influenced them.
///
/// Emptiness is the decision records only. Sources are ledger-global repo
/// pins, so on their own they never conjure a decision record; as with
/// promotion assertions in the experiment report, they only enrich a record
/// that renders. A Source node needs both `path` and `commit` to become a
/// reference: a missing or blank half skips the node entirely, with no row and
/// no source-map entry.
fn render_decision(graph: &Graph) -> Option<RenderedBlock> {
    let (decisions, mut sources) = entity_rows(graph, &[EntityType::Decision]);
    if decisions.is_empty() {
        return None;
    }

    // Evidence sorts by id then reference, so one finding influenced by two
    // decisions keeps one row per reference. A Source missing or blank in
    // `path` or `commit` is not a reference: it renders no row and enters no
    // source map.
    let mut evidence: Vec<(String, String, String)> = Vec::new();
    for node in graph
        .nodes()
        .filter(|node| node.entity == EntityType::Source)
    {
        let Some(reference) = source_reference(node) else {
            continue;
        };
        evidence.push((
            node.id.clone(),
            reference.clone(),
            format!("- {} — {reference}", node.id),
        ));
        sources.insert(node.id.clone());
    }
    for edge in graph.edges() {
        if edge.relationship != Relationship::Influences {
            continue;
        }
        let (Some(from), Some(to)) = (graph.node(&edge.from), graph.node(&edge.to)) else {
            continue;
        };
        let (finding, decision) = match (from.entity, to.entity) {
            (EntityType::Decision, EntityType::Finding) => (to, from),
            (EntityType::Finding, EntityType::Decision) => (from, to),
            _ => continue,
        };
        evidence.push((
            finding.id.clone(),
            decision.id.clone(),
            format!("- {} — see {}", finding.id, decision.id),
        ));
        sources.insert(finding.id.clone());
        sources.insert(decision.id.clone());
    }
    evidence.sort();
    evidence.dedup();

    let evidence_lines: Vec<String> = evidence.into_iter().map(|(_, _, line)| line).collect();

    let mut body = String::new();
    push_heading(&mut body, 3, "Decisions", &decisions);
    push_heading(&mut body, 3, "Evidence", &evidence_lines);

    Some(RenderedBlock {
        body,
        sources: sources.into_iter().collect(),
    })
}

/// Append one section; an empty group renders no heading and no placeholder.
fn push_section(body: &mut String, title: &str, lines: &[String]) {
    push_heading(body, 2, title, lines);
}

/// Append one section at `level` heading depth; an empty group renders nothing.
fn push_heading(body: &mut String, level: usize, title: &str, lines: &[String]) {
    if lines.is_empty() {
        return;
    }
    if !body.is_empty() {
        body.push('\n');
    }
    for _ in 0..level {
        body.push('#');
    }
    body.push(' ');
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
