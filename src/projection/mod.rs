//! Documentation as a compilation: the managed-block grammar.
//!
//! A projection document mixes compiler-owned generated sections with
//! human-maintained authored sections. kaya owns only its own markers:
//! everything between `<!-- kaya:begin generated:<name> -->` and
//! `<!-- kaya:end generated:<name> -->` may be rewritten, everything
//! between `<!-- kaya:authored -->` and `<!-- kaya:end authored -->` is
//! interpreted prose, and every other byte — including foreign tools'
//! managed blocks — is preserved verbatim.
//!
//! [`scan`] parses a document into managed blocks and refuses a broken
//! grammar; [`replace_generated`] rewrites exactly one generated block's
//! body and [`remove_generated`] deletes one, leaving the surrounding bytes
//! untouched.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::Path;

use crate::model::Node;

pub mod index;
pub mod ir;
pub mod report;
pub mod reuse;

/// Prefix of a generated-block begin marker.
pub const BEGIN_PREFIX: &str = "<!-- kaya:begin generated:";

const END_GENERATED_PREFIX: &str = "<!-- kaya:end generated:";
const BEGIN_AUTHORED: &str = "<!-- kaya:authored -->";
const END_AUTHORED: &str = "<!-- kaya:end authored -->";
/// Label for refusals produced by the managed-block grammar.
pub const LABEL_MANAGED_GRAMMAR: &str = "kaya.projection.managed_grammar";

/// A labeled refusal produced by the projection grammar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectionError {
    /// Failure class, for example `kaya.projection.managed_grammar`.
    pub label: String,
    /// Human-readable explanation naming the offending block.
    pub message: String,
}

impl ProjectionError {
    /// Construct a labeled refusal.
    pub fn new(label: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ProjectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.label, self.message)
    }
}

impl std::error::Error for ProjectionError {}

/// Who owns a managed block's body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockKind {
    /// Compiler-owned block; the string is its generated name.
    Generated(String),
    /// Human-maintained block.
    Authored,
}

/// One managed block found in a document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    /// Ownership of [`Self::body`].
    pub kind: BlockKind,
    /// Bytes strictly between the begin and end markers.
    pub body: String,
}

/// An authored block whose cited evidence changed after its acknowledgment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaleBlock {
    /// Name of the stale block as reported in the build summary.
    pub block: String,
    /// Changed evidence ids backing the stale claim.
    pub evidence: Vec<String>,
}

/// The result of rebuilding a projection.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Summary {
    /// Block names whose bytes were newly generated.
    pub recomputed: Vec<String>,
    /// Block names whose bytes already matched and were left alone.
    pub unchanged: Vec<String>,
    /// Authored blocks whose evidence moved underneath them.
    pub stale: Vec<StaleBlock>,
    /// Generated block name to the sorted node ids backing its facts.
    pub sources: BTreeMap<String, Vec<String>>,
}

/// A managed block plus the byte range of its body.
struct Span {
    kind: BlockKind,
    body_start: usize,
    body_end: usize,
}

#[derive(Debug)]
enum Marker<'a> {
    BeginGenerated(&'a str),
    EndGenerated(&'a str),
    BeginAuthored,
    EndAuthored,
}

/// Parse every kaya-managed block in `doc`, in document order.
///
/// Foreign markers are ordinary prose. An unterminated block, a mismatched
/// end marker, or a stray end marker is refused with
/// `kaya.projection.managed_grammar` naming the block.
pub fn scan(doc: &str) -> Result<Vec<Block>, ProjectionError> {
    let spans = scan_spans(doc)?;
    Ok(spans
        .into_iter()
        .map(|span| Block {
            kind: span.kind,
            body: doc[span.body_start..span.body_end].to_owned(),
        })
        .collect())
}

/// Refuse any generated block whose name is outside the closed vocabulary or
/// appears more than once in the document.
///
/// Every projection validates its document with this gate before rendering or
/// writing: a legacy block name and a repeated generated name are both refused
/// with the document left untouched.
pub fn validate_generated_blocks(doc: &str) -> Result<(), ProjectionError> {
    let mut seen = BTreeSet::new();
    for block in scan(doc)? {
        if let BlockKind::Generated(name) = block.kind {
            report::classify_block(&name)?;
            if !seen.insert(name.clone()) {
                return Err(ProjectionError::new(
                    LABEL_MANAGED_GRAMMAR,
                    format!("duplicate generated block \"{name}\""),
                ));
            }
        }
    }
    Ok(())
}

/// Replace the body of generated block `name` with `body`.
///
/// When the block is absent a new generated block is appended at the end of
/// the document, preceded by one blank line. Every byte outside the named
/// block's body is preserved exactly. A document with a broken grammar is
/// refused without producing output.
pub fn replace_generated(doc: &str, name: &str, body: &str) -> Result<String, ProjectionError> {
    if !valid_name(name) {
        return Err(ProjectionError::new(
            LABEL_MANAGED_GRAMMAR,
            format!("invalid generated block name \"{name}\""),
        ));
    }

    let spans = scan_spans(doc)?;
    let mut matching = spans.iter().filter(
        |span| matches!(&span.kind, BlockKind::Generated(existing) if existing.as_str() == name),
    );

    if let Some(span) = matching.next() {
        if matching.next().is_some() {
            return Err(ProjectionError::new(
                LABEL_MANAGED_GRAMMAR,
                format!("duplicate generated block \"{name}\""),
            ));
        }
        let mut out =
            String::with_capacity(doc.len() - (span.body_end - span.body_start) + body.len());
        out.push_str(&doc[..span.body_start]);
        out.push_str(body);
        out.push_str(&doc[span.body_end..]);
        return Ok(out);
    }

    let mut out = String::with_capacity(doc.len() + body.len() + name.len() * 2 + 64);
    out.push_str(doc);
    if !doc.is_empty() {
        if !doc.ends_with('\n') {
            out.push('\n');
        }
        out.push('\n');
    }
    out.push_str(BEGIN_PREFIX);
    out.push_str(name);
    out.push_str(" -->\n");
    out.push_str(body);
    if !body.is_empty() && !body.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(END_GENERATED_PREFIX);
    out.push_str(name);
    out.push_str(" -->\n");
    Ok(out)
}

/// Remove generated block `name` together with its begin and end markers.
///
/// The single blank separator line immediately preceding the begin marker is
/// removed too, so deleting a block leaves no empty section behind. Every
/// other byte is preserved exactly. A document without the named block is
/// returned unchanged; a broken grammar is refused without producing output.
pub fn remove_generated(doc: &str, name: &str) -> Result<String, ProjectionError> {
    let spans = scan_spans(doc)?;
    let mut matching = spans.iter().filter(
        |span| matches!(&span.kind, BlockKind::Generated(existing) if existing.as_str() == name),
    );

    let Some(span) = matching.next() else {
        return Ok(doc.to_owned());
    };
    if matching.next().is_some() {
        return Err(ProjectionError::new(
            LABEL_MANAGED_GRAMMAR,
            format!("duplicate generated block \"{name}\""),
        ));
    }

    // `body_start` begins the line after the begin marker, so the marker's
    // line starts after the newline preceding that line.
    let begin_start = doc[..span.body_start.saturating_sub(1)]
        .rfind('\n')
        .map(|index| index + 1)
        .unwrap_or(0);
    // `body_end` begins the end-marker line; keep its trailing newline with
    // the removed span so the surrounding lines close up cleanly.
    let end_end = doc[span.body_end..]
        .find('\n')
        .map(|index| span.body_end + index + 1)
        .unwrap_or(doc.len());

    // Strip the blank separator line before the begin marker. At byte 1 the
    // document opens with the separator itself, so there is no earlier byte
    // to pair with; the first byte still counts as the blank line.
    let remove_from = if begin_start >= 1
        && doc.as_bytes()[begin_start - 1] == b'\n'
        && (begin_start < 2 || doc.as_bytes()[begin_start - 2] == b'\n')
    {
        begin_start - 1
    } else {
        begin_start
    };

    let mut out = String::with_capacity(doc.len() - (end_end - remove_from));
    out.push_str(&doc[..remove_from]);
    out.push_str(&doc[end_end..]);
    Ok(out)
}

/// Label for projection document read and write failures.
pub(crate) const LABEL_IO: &str = "kaya.projection.io";

/// The body of generated block `name`, when the document carries it.
pub(crate) fn generated_body(
    document: &str,
    name: &str,
) -> Result<Option<String>, ProjectionError> {
    let blocks = scan(document)?;
    Ok(blocks.into_iter().find_map(|block| match block.kind {
        BlockKind::Generated(existing) if existing.as_str() == name => Some(block.body),
        _ => None,
    }))
}

/// Read a projection document, treating absence as an empty document.
pub(crate) fn read_existing(path: &Path) -> Result<Option<String>, ProjectionError> {
    match fs::read_to_string(path) {
        Ok(document) => Ok(Some(document)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(io_error("read", path, error)),
    }
}

/// Write a projection document verbatim.
pub(crate) fn write(path: &Path, document: &str) -> Result<(), ProjectionError> {
    fs::write(path, document).map_err(|error| io_error("write", path, error))
}

/// Render an I/O failure as a labeled projection refusal naming the path.
pub(crate) fn io_error(action: &str, path: &Path, error: io::Error) -> ProjectionError {
    ProjectionError::new(
        LABEL_IO,
        format!("failed to {action} \"{}\": {error}", path.display()),
    )
}

/// A single-line, trimmed string attribute; blank values count as absent.
///
/// A missing attribute, a blank value, and a non-string value all yield
/// `None`, so a projection never renders an empty placeholder for one.
pub(crate) fn attribute(node: &Node, name: &str) -> Option<String> {
    let value = node
        .attributes
        .get(name)
        .and_then(serde_json::Value::as_str)?;
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

/// The key segment of a node id, used when the node carries no title.
pub(crate) fn key(node: &Node) -> String {
    node.id
        .rsplit('/')
        .next()
        .unwrap_or(node.id.as_str())
        .to_owned()
}

/// A node's single-line title, or the key segment of its id when untitled.
pub(crate) fn title_or_key(node: &Node) -> String {
    attribute(node, "title").unwrap_or_else(|| key(node))
}

/// A repo file reference as `path@commit`, pinned to the recorded revision.
///
/// A reference needs both halves: a node missing or blank in `path` or
/// `commit` has no resolvable reference, so it yields `None` and the caller
/// renders no row rather than `path@` or `@commit`.
pub(crate) fn source_reference(node: &Node) -> Option<String> {
    let path = attribute(node, "path")?;
    let commit = attribute(node, "commit")?;
    Some(format!("{path}@{commit}"))
}

fn scan_spans(doc: &str) -> Result<Vec<Span>, ProjectionError> {
    let mut lines: Vec<(usize, usize, &str)> = Vec::new();
    let mut offset = 0;
    for raw in doc.split_inclusive('\n') {
        let content = raw.strip_suffix('\n').unwrap_or(raw);
        lines.push((offset, offset + raw.len(), content));
        offset += raw.len();
    }

    let mut spans = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        let marker = match parse_marker(lines[index].2) {
            Some(marker) => marker,
            None => {
                if looks_like_marker(lines[index].2) {
                    return Err(ProjectionError::new(
                        LABEL_MANAGED_GRAMMAR,
                        format!("malformed kaya marker \"{}\"", lines[index].2.trim()),
                    ));
                }
                index += 1;
                continue;
            }
        };

        match marker {
            Marker::BeginGenerated(name) => {
                let body_start = lines[index].1;
                let mut cursor = index + 1;
                let closed = loop {
                    if cursor >= lines.len() {
                        break None;
                    }
                    match parse_marker(lines[cursor].2) {
                        Some(Marker::EndGenerated(end_name)) => {
                            if end_name == name {
                                break Some((cursor, lines[cursor].0));
                            }
                            return Err(mismatched_end(name, end_name));
                        }
                        Some(Marker::EndAuthored) => {
                            return Err(mismatched_end(name, "authored"));
                        }
                        Some(Marker::BeginGenerated(_)) | Some(Marker::BeginAuthored) => {
                            break None;
                        }
                        None => cursor += 1,
                    }
                };

                let (end_line, body_end) = closed.ok_or_else(|| unterminated(name))?;
                spans.push(Span {
                    kind: BlockKind::Generated(name.to_owned()),
                    body_start,
                    body_end,
                });
                index = end_line + 1;
            }
            Marker::BeginAuthored => {
                let body_start = lines[index].1;
                let mut cursor = index + 1;
                let closed = loop {
                    if cursor >= lines.len() {
                        break None;
                    }
                    match parse_marker(lines[cursor].2) {
                        Some(Marker::EndAuthored) => break Some((cursor, lines[cursor].0)),
                        Some(Marker::EndGenerated(end_name)) => {
                            return Err(mismatched_end("authored", end_name));
                        }
                        Some(Marker::BeginGenerated(_)) | Some(Marker::BeginAuthored) => {
                            break None;
                        }
                        None => cursor += 1,
                    }
                };
                let (end_line, body_end) = closed.ok_or_else(|| unterminated("authored"))?;
                spans.push(Span {
                    kind: BlockKind::Authored,
                    body_start,
                    body_end,
                });
                index = end_line + 1;
            }
            Marker::EndGenerated(name) => {
                return Err(ProjectionError::new(
                    LABEL_MANAGED_GRAMMAR,
                    format!("end marker for generated block \"{name}\" has no begin marker"),
                ));
            }
            Marker::EndAuthored => {
                return Err(ProjectionError::new(
                    LABEL_MANAGED_GRAMMAR,
                    "end marker for authored block has no begin marker",
                ));
            }
        }
    }

    Ok(spans)
}

fn parse_marker(line: &str) -> Option<Marker<'_>> {
    let trimmed = line.trim();
    if trimmed == BEGIN_AUTHORED {
        return Some(Marker::BeginAuthored);
    }
    if trimmed == END_AUTHORED {
        return Some(Marker::EndAuthored);
    }
    if let Some(rest) = trimmed.strip_prefix(BEGIN_PREFIX) {
        if let Some(name) = rest.strip_suffix(" -->")
            && valid_name(name)
        {
            return Some(Marker::BeginGenerated(name));
        }
        return None;
    }
    if let Some(rest) = trimmed.strip_prefix(END_GENERATED_PREFIX) {
        if let Some(name) = rest.strip_suffix(" -->")
            && valid_name(name)
        {
            return Some(Marker::EndGenerated(name));
        }
        return None;
    }
    None
}

fn looks_like_marker(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with("<!-- kaya:begin generated:")
        || trimmed.starts_with(END_GENERATED_PREFIX)
        || trimmed.starts_with("<!-- kaya:authored")
        || trimmed.starts_with("<!-- kaya:end authored")
}

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && !name
            .chars()
            .any(|c| c.is_whitespace() || c == '<' || c == '>')
}

fn block_label(name: &str) -> String {
    if name == "authored" {
        "authored".to_owned()
    } else {
        format!("generated:{name}")
    }
}

fn unterminated(name: &str) -> ProjectionError {
    ProjectionError::new(
        LABEL_MANAGED_GRAMMAR,
        format!("unterminated {} block", block_label(name)),
    )
}

fn mismatched_end(name: &str, end_name: &str) -> ProjectionError {
    ProjectionError::new(
        LABEL_MANAGED_GRAMMAR,
        format!(
            "mismatched end marker \"{end_name}\" while closing {} block",
            block_label(name)
        ),
    )
}
