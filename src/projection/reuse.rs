//! The reuse-handoff reference projections.
//!
//! Purpose: compile technical-reference, reproduction-package, knowledge-brief
//! and handoff documents that point at authoritative sources instead of
//! copying their payloads.
//! Responsibilities: render deterministic reference rows from graph inputs,
//! validate existing documents against the closed block vocabulary before any
//! write, replace only the family-owned generated blocks, omit empty kinds
//! without creating a file or block, and assemble those inputs from the
//! modelled graph.
//! Rationale: a reuse document is an index of resolvable references — the
//! ledger stays the only record, every row names a source that can be opened
//! without kaya, and authored bytes survive regeneration untouched.
//!
//! A node whose required attribute is absent, or blank after trim, supplies
//! no reference: absence is not a reference. An attribute that is present but
//! not a string is a labeled refusal naming the node and attribute, never a
//! silent skip. Rendered rows are always single-line — `\n` and `\r` collapse
//! to a space and the reference is trimmed — and a kind is empty solely when
//! the ledger yields no renderable row for it, never because a document is
//! absent.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::model::{EntityType, Graph};
use crate::projection::report::{Kind, Omitted, REASON_NO_RECORDS};
use crate::projection::{
    ProjectionError, Summary, generated_body, io_error, read_existing, remove_generated,
    replace_generated, validate_generated_blocks, write,
};

/// Generated block carrying the technical-reference spec ids.
pub const TECHNICAL_REFERENCE_BLOCK: &str = "reuse-handoff.technical-reference";
/// Generated block carrying the reproduction-package references.
pub const REPRODUCTION_BLOCK: &str = "reuse-handoff.reproduction-package";
/// Generated block carrying the knowledge-brief entry ids.
pub const KNOWLEDGE_BRIEF_BLOCK: &str = "reuse-handoff.knowledge-brief";
/// Generated block carrying the handoff references.
pub const HANDOFF_BLOCK: &str = "reuse-handoff.handoff";

/// Label refusing a reuse attribute that cannot become a reference.
pub const LABEL_REUSE_ATTRIBUTE: &str = "kaya.projection.reuse_attribute";

/// File receiving the technical-reference block, under the output directory.
const TECHNICAL_REFERENCE_FILE: &str = "technical-reference.md";
/// File receiving the reproduction-package block, under the output directory.
const REPRODUCTION_FILE: &str = "reproduction-package.md";
/// File receiving the knowledge-brief block, under the output directory.
const KNOWLEDGE_BRIEF_FILE: &str = "knowledge-brief.md";
/// File receiving the handoff block, under the output directory.
const HANDOFF_FILE: &str = "handoff.md";

/// One row: a kaya node id and a source reference resolvable without kaya.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRef {
    /// Id of the kaya node the row names.
    pub id: String,
    /// Reference a reader can resolve without kaya (a path, command, or
    /// `path@commit`).
    pub reference: String,
}

/// Inputs assembled from the modelled graph by the caller.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Inputs {
    /// Specodelic spec ids, rendered as the id itself.
    pub specs: Vec<SourceRef>,
    /// Turu/whisper entry ids, rendered as the id itself.
    pub entries: Vec<SourceRef>,
    /// Source-node references, rendered as `path@commit`.
    pub revisions: Vec<SourceRef>,
    /// Run command references, one row per command.
    pub commands: Vec<SourceRef>,
    /// Handoff artifact references, rendered as their artifact path.
    pub handoffs: Vec<SourceRef>,
}

/// The result of building the reuse documents: the merged summary plus the
/// family-kinds that produced no block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Built {
    /// Recomputed, unchanged, and source-map bookkeeping.
    pub summary: Summary,
    /// Family-kinds with no source records, in block order.
    pub omitted: Vec<Omitted>,
    /// The document IR materialized from the planned documents.
    pub ir: crate::projection::ir::DocumentIr,
}

/// One reuse document's planned bytes, before the write phase.
///
/// `document` is `None` when the projection leaves no file at `path`: a kind
/// with no rows never creates one, and a removal that leaves only whitespace
/// deletes the existing document. That makes the plan the whole answer for
/// `kaya check`, which compares these bytes without writing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedDocument {
    /// Document path under the output directory.
    pub path: PathBuf,
    /// The document the projection leaves behind, or `None` for absence.
    pub document: Option<String>,
    /// Recomputed, unchanged, and source-map bookkeeping for this document.
    pub summary: Summary,
    /// Family-kinds with no source records, in block order.
    pub omitted: Vec<Omitted>,
}

/// The four reuse documents under `out`, in canonical kind order, paired with
/// their generated block names.
pub fn documents(out: &Path) -> [(&'static str, PathBuf); 4] {
    [
        (
            TECHNICAL_REFERENCE_BLOCK,
            out.join(TECHNICAL_REFERENCE_FILE),
        ),
        (REPRODUCTION_BLOCK, out.join(REPRODUCTION_FILE)),
        (KNOWLEDGE_BRIEF_BLOCK, out.join(KNOWLEDGE_BRIEF_FILE)),
        (HANDOFF_BLOCK, out.join(HANDOFF_FILE)),
    ]
}

/// Write the reuse documents under `out`; return what was written/omitted.
///
/// Every existing target document is read and validated against the closed
/// block vocabulary before any document is written, so a legacy or duplicate
/// generated block leaves all bytes untouched. A kind with rows replaces or
/// appends its block only when the body changed; a kind with no rows writes no
/// file, reports one omission, and removes a stale block from an existing
/// document. A removal that leaves only whitespace deletes the document
/// instead of writing an empty one.
pub fn build(inputs: &Inputs, out: &Path) -> Result<Built, ProjectionError> {
    let mut existing: BTreeMap<PathBuf, Option<String>> = BTreeMap::new();
    for (_, path) in documents(out) {
        let document = read_validated(&path)?;
        existing.insert(path, document);
    }

    let mut summary = Summary::default();
    let mut omitted = Vec::new();
    let mut ir = crate::projection::ir::DocumentIr::default();
    for planned in plan(inputs, out, &existing)? {
        let previous = existing.get(&planned.path).cloned().flatten();
        match &planned.document {
            None => {
                if previous.is_some() {
                    fs::remove_file(&planned.path)
                        .map_err(|error| io_error("remove", &planned.path, error))?;
                }
            }
            Some(document) => {
                if previous.as_deref() != Some(document.as_str()) {
                    write(&planned.path, document)?;
                }
            }
        }
        let mut planned_summary = planned.summary;
        let planned_ir = crate::projection::ir::for_document(
            &planned.path,
            planned.document.as_deref().unwrap_or(""),
            &mut planned_summary,
        );
        ir.records.extend(planned_ir.records);
        merge(&mut summary, planned_summary);
        omitted.extend(planned.omitted);
    }

    Ok(Built {
        summary,
        omitted,
        ir,
    })
}

/// The bytes each reuse document would hold, without touching the disk.
///
/// Every existing target in `existing` is validated against the closed block
/// vocabulary before any document is planned, so a legacy or duplicate
/// generated block refuses the whole family. A kind with rows replaces or
/// appends its block only when the planned bytes differ from the existing
/// document; a kind with no rows plans no file and removes a stale block,
/// leaving `None` when the document is emptied. Planning is the whole
/// projection, so `kaya check` compares these bytes without a second renderer.
pub fn plan(
    inputs: &Inputs,
    out: &Path,
    existing: &BTreeMap<PathBuf, Option<String>>,
) -> Result<Vec<PlannedDocument>, ProjectionError> {
    let targets = documents(out);
    for (_, path) in &targets {
        if let Some(Some(document)) = existing.get(path) {
            validate_generated_blocks(document)?;
        }
    }

    let [
        (_, technical),
        (_, reproduction),
        (_, knowledge),
        (_, handoff),
    ] = targets;
    let mut planned = Vec::with_capacity(4);
    for (kind, path, rendered) in [
        (
            Kind::TechnicalReference,
            technical,
            render_technical_reference(inputs),
        ),
        (
            Kind::ReproductionPackage,
            reproduction,
            render_reproduction(inputs),
        ),
        (
            Kind::KnowledgeBrief,
            knowledge,
            render_knowledge_brief(inputs),
        ),
        (Kind::Handoff, handoff, render_handoff(inputs)),
    ] {
        let previous = existing.get(&path).cloned().flatten();
        planned.push(plan_document(kind, path, previous, rendered)?);
    }
    Ok(planned)
}

/// Merge one document's summary into the family summary.
fn merge(into: &mut Summary, from: Summary) {
    into.recomputed.extend(from.recomputed);
    into.unchanged.extend(from.unchanged);
    for (block, ids) in from.sources {
        into.sources.entry(block).or_default().extend(ids);
    }
}

/// Read an existing target document and validate its managed blocks.
fn read_validated(path: &Path) -> Result<Option<String>, ProjectionError> {
    let document = read_existing(path)?;
    if let Some(document) = &document {
        validate_generated_blocks(document)?;
    }
    Ok(document)
}

/// Assemble reuse inputs from the modelled graph.
///
/// Sources contribute a specodelic spec id from `spec`, a turu/whisper entry
/// id from `entry`, and a revision row when both `path` and `commit` are
/// present and non-blank; the three selections are independent, so one source
/// may carry several. Runs contribute one command row per `command` string or
/// `commands` array entry; artifacts whose `kind` is `handoffs` contribute one
/// row per path. An absent or blank attribute skips that selection, while a
/// present non-string attribute — or a `commands` array holding a non-string
/// entry — is refused with [`LABEL_REUSE_ATTRIBUTE`] naming the node and
/// attribute. Graph iteration order keeps the result deterministic.
pub fn inputs(graph: &Graph) -> Result<Inputs, ProjectionError> {
    let mut inputs = Inputs::default();
    for node in graph.nodes() {
        match node.entity {
            EntityType::Source => {
                if let Some(spec) = non_blank(string_attribute(&node.id, &node.attributes, "spec")?)
                {
                    inputs.specs.push(SourceRef {
                        id: node.id.clone(),
                        reference: spec.to_owned(),
                    });
                }
                if let Some(entry) =
                    non_blank(string_attribute(&node.id, &node.attributes, "entry")?)
                {
                    inputs.entries.push(SourceRef {
                        id: node.id.clone(),
                        reference: entry.to_owned(),
                    });
                }
                let Some(path) = non_blank(string_attribute(&node.id, &node.attributes, "path")?)
                else {
                    continue;
                };
                let Some(commit) =
                    non_blank(string_attribute(&node.id, &node.attributes, "commit")?)
                else {
                    continue;
                };
                inputs.revisions.push(SourceRef {
                    id: node.id.clone(),
                    reference: format!("{path}@{commit}"),
                });
            }
            EntityType::Run => {
                for command in command_refs(&node.id, &node.attributes)? {
                    inputs.commands.push(SourceRef {
                        id: node.id.clone(),
                        reference: command.to_owned(),
                    });
                }
            }
            EntityType::Artifact => {
                if node
                    .attributes
                    .get("kind")
                    .and_then(serde_json::Value::as_str)
                    != Some("handoffs")
                {
                    continue;
                }
                let Some(path) = non_blank(string_attribute(&node.id, &node.attributes, "path")?)
                else {
                    continue;
                };
                inputs.handoffs.push(SourceRef {
                    id: node.id.clone(),
                    reference: path.to_owned(),
                });
            }
            _ => {}
        }
    }
    Ok(inputs)
}

/// The run's command references: `command`, `commands`, or both.
fn command_refs<'a>(
    node_id: &str,
    attributes: &'a serde_json::Value,
) -> Result<Vec<&'a str>, ProjectionError> {
    let mut commands = Vec::new();
    if let Some(command) = non_blank(string_attribute(node_id, attributes, "command")?) {
        commands.push(command);
    }
    match attributes.get("commands") {
        None => {}
        Some(serde_json::Value::Array(entries)) => {
            for entry in entries {
                let serde_json::Value::String(command) = entry else {
                    return Err(attribute_refusal(
                        node_id,
                        "commands",
                        "an array of strings",
                    ));
                };
                if !command.trim().is_empty() {
                    commands.push(command.as_str());
                }
            }
        }
        Some(_) => {
            return Err(attribute_refusal(
                node_id,
                "commands",
                "an array of strings",
            ));
        }
    }
    Ok(commands)
}

/// One non-empty rendered block: its body and the ids backing it.
struct Rendered {
    body: String,
    sources: Vec<String>,
}

/// Render the reproduction-package block from revisions and commands.
///
/// Sections appear only when non-empty; a kind with no rows renders nothing.
fn render_reproduction(inputs: &Inputs) -> Option<Rendered> {
    crate::projection::bump_render_calls();
    let revisions = rows(&inputs.revisions);
    let commands = rows(&inputs.commands);
    if revisions.is_empty() && commands.is_empty() {
        return None;
    }

    let mut body = String::new();
    push_section(&mut body, "Sources", &revisions);
    push_section(&mut body, "Commands", &commands);

    let mut sources = BTreeSet::new();
    for row in inputs.revisions.iter().chain(inputs.commands.iter()) {
        sources.insert(row.id.clone());
    }

    Some(Rendered {
        body,
        sources: sources.into_iter().collect(),
    })
}

/// Render the technical-reference block from specodelic spec ids.
fn render_technical_reference(inputs: &Inputs) -> Option<Rendered> {
    crate::projection::bump_render_calls();
    render_flat(&inputs.specs, "Specs")
}

/// Render the knowledge-brief block from turu/whisper entry ids.
fn render_knowledge_brief(inputs: &Inputs) -> Option<Rendered> {
    crate::projection::bump_render_calls();
    render_flat(&inputs.entries, "Entries")
}

/// Render the handoff block from handoff artifact references.
fn render_handoff(inputs: &Inputs) -> Option<Rendered> {
    crate::projection::bump_render_calls();
    render_flat(&inputs.handoffs, "Handoffs")
}

/// Render one single-section reference kind: the sorted rows under `title`
/// and the distinct node ids backing them. No rows renders nothing.
fn render_flat(references: &[SourceRef], title: &str) -> Option<Rendered> {
    crate::projection::bump_render_calls();
    let rows = rows(references);
    if rows.is_empty() {
        return None;
    }

    let mut body = String::new();
    push_section(&mut body, title, &rows);

    // A blank reference renders no row, so its node must not be attributed.
    let sources: BTreeSet<String> = references
        .iter()
        .filter(|row| !row.reference.trim().is_empty())
        .map(|row| row.id.clone())
        .collect();

    Some(Rendered {
        body,
        sources: sources.into_iter().collect(),
    })
}

/// Plan one kind's rendered rows (or its omission) for its target document.
///
/// A rendered block is reused when the existing bytes already carry it, and
/// otherwise replaces or appends it; an unrendered kind reports one omission
/// and removes its stale block, leaving `None` when that empties the document.
fn plan_document(
    kind: Kind,
    path: PathBuf,
    existing: Option<String>,
    rendered: Option<Rendered>,
) -> Result<PlannedDocument, ProjectionError> {
    let block = kind.block_name();
    let mut summary = Summary::default();
    let mut omitted = Vec::new();
    let document = match rendered {
        Some(rendered) => {
            let current = match &existing {
                Some(document) => generated_body(document, &block)?,
                None => None,
            };
            let document = if current.as_deref() == Some(rendered.body.as_str()) {
                summary.unchanged.push(block.clone());
                existing
            } else {
                let next =
                    replace_generated(existing.as_deref().unwrap_or(""), &block, &rendered.body)?;
                summary.recomputed.push(block.clone());
                Some(next)
            };
            summary.sources.insert(block, rendered.sources);
            document
        }
        None => {
            omitted.push(Omitted {
                family: kind.family().as_str(),
                kind: kind.as_str(),
                reason: REASON_NO_RECORDS,
            });
            match existing {
                Some(document) if generated_body(&document, &block)?.is_some() => {
                    let next = remove_generated(&document, &block)?;
                    summary.recomputed.push(block);
                    if next.trim().is_empty() {
                        None
                    } else {
                        Some(next)
                    }
                }
                other => other,
            }
        }
    };

    Ok(PlannedDocument {
        path,
        document,
        summary,
        omitted,
    })
}

/// Sorted, de-duplicated single-line reference rows: by id, then reference.
fn rows(inputs: &[SourceRef]) -> Vec<String> {
    let mut sorted: Vec<(String, String)> = inputs
        .iter()
        .map(|row| {
            (
                single_line(&row.id),
                single_line(&row.reference).trim().to_owned(),
            )
        })
        .filter(|(_, reference)| !reference.is_empty())
        .collect();
    sorted.sort();
    sorted.dedup();
    sorted
        .into_iter()
        .map(|(id, reference)| format!("- {id} — {reference}"))
        .collect()
}

/// Collapse embedded line breaks into spaces so a row stays one line.
fn single_line(value: &str) -> String {
    value.replace(['\n', '\r'], " ")
}

/// Append one section; an empty group renders no heading and no placeholder.
fn push_section(body: &mut String, title: &str, lines: &[String]) {
    if lines.is_empty() {
        return;
    }
    if !body.is_empty() {
        body.push('\n');
    }
    body.push_str("### ");
    body.push_str(title);
    body.push('\n');
    for line in lines {
        body.push_str(line);
        body.push('\n');
    }
}

/// A present attribute that must be a JSON string, or the labeled refusal.
fn string_attribute<'a>(
    node_id: &str,
    attributes: &'a serde_json::Value,
    key: &str,
) -> Result<Option<&'a str>, ProjectionError> {
    match attributes.get(key) {
        None => Ok(None),
        Some(serde_json::Value::String(value)) => Ok(Some(value)),
        Some(_) => Err(attribute_refusal(node_id, key, "a string")),
    }
}

/// Drop an absent or blank attribute value: neither is a reference.
fn non_blank(value: Option<&str>) -> Option<&str> {
    value.filter(|value| !value.trim().is_empty())
}

fn attribute_refusal(node_id: &str, key: &str, expectation: &str) -> ProjectionError {
    ProjectionError::new(
        LABEL_REUSE_ATTRIBUTE,
        format!("node \"{node_id}\" attribute \"{key}\" must be {expectation}"),
    )
}
