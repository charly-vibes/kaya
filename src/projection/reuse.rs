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

use std::collections::BTreeSet;
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
    let [
        (_, technical_path),
        (_, reproduction_path),
        (_, knowledge_path),
        (_, handoff_path),
    ] = documents(out);

    // Every existing target is read and validated before any document is
    // written, so a legacy block leaves all four documents byte-identical.
    let mut technical_document = read_validated(&technical_path)?;
    let mut reproduction_document = read_validated(&reproduction_path)?;
    let mut knowledge_document = read_validated(&knowledge_path)?;
    let mut handoff_document = read_validated(&handoff_path)?;

    let mut summary = Summary::default();
    let mut omitted = Vec::new();

    apply(
        Kind::TechnicalReference,
        &technical_path,
        &mut technical_document,
        render_technical_reference(inputs),
        &mut summary,
        &mut omitted,
    )?;
    apply(
        Kind::ReproductionPackage,
        &reproduction_path,
        &mut reproduction_document,
        render_reproduction(inputs),
        &mut summary,
        &mut omitted,
    )?;
    apply(
        Kind::KnowledgeBrief,
        &knowledge_path,
        &mut knowledge_document,
        render_knowledge_brief(inputs),
        &mut summary,
        &mut omitted,
    )?;
    apply(
        Kind::Handoff,
        &handoff_path,
        &mut handoff_document,
        render_handoff(inputs),
        &mut summary,
        &mut omitted,
    )?;

    Ok(Built { summary, omitted })
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
    render_flat(&inputs.specs, "Specs")
}

/// Render the knowledge-brief block from turu/whisper entry ids.
fn render_knowledge_brief(inputs: &Inputs) -> Option<Rendered> {
    render_flat(&inputs.entries, "Entries")
}

/// Render the handoff block from handoff artifact references.
fn render_handoff(inputs: &Inputs) -> Option<Rendered> {
    render_flat(&inputs.handoffs, "Handoffs")
}

/// Render one single-section reference kind: the sorted rows under `title`
/// and the distinct node ids backing them. No rows renders nothing.
fn render_flat(references: &[SourceRef], title: &str) -> Option<Rendered> {
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

/// Apply one kind's rendered rows (or its omission) to its target document.
fn apply(
    kind: Kind,
    path: &Path,
    document: &mut Option<String>,
    rendered: Option<Rendered>,
    summary: &mut Summary,
    omitted: &mut Vec<Omitted>,
) -> Result<(), ProjectionError> {
    let block = kind.block_name();
    match rendered {
        Some(rendered) => {
            let existing = match document {
                Some(document) => generated_body(document, &block)?,
                None => None,
            };
            if existing.as_deref() == Some(rendered.body.as_str()) {
                summary.unchanged.push(block.clone());
            } else {
                let base = document.take().unwrap_or_default();
                let next = replace_generated(&base, &block, &rendered.body)?;
                write(path, &next)?;
                *document = Some(next);
                summary.recomputed.push(block.clone());
            }
            summary.sources.insert(block, rendered.sources);
        }
        None => {
            omitted.push(Omitted {
                family: kind.family().as_str(),
                kind: kind.as_str(),
                reason: REASON_NO_RECORDS,
            });
            if let Some(existing) = document.as_ref()
                && generated_body(existing, &block)?.is_some()
            {
                let next = remove_generated(existing, &block)?;
                if next.trim().is_empty() {
                    fs::remove_file(path).map_err(|error| io_error("remove", path, error))?;
                    *document = None;
                } else {
                    write(path, &next)?;
                    *document = Some(next);
                }
                summary.recomputed.push(block);
            }
        }
    }
    Ok(())
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
