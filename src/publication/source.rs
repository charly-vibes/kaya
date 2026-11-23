//! The compiled-document source of every publication.
//!
//! Purpose: read compiled documents into the kind-keyed source map the
//! composition and the export both project from. Responsibilities: read the
//! research tree in a fixed order, keep only generated blocks whose names are
//! in the document vocabulary, refuse a repeated block inside one file, let
//! the first populated block win across files, and extract each body's
//! `kaya/...` event ids. Rationale: one canonical source map means a fact is
//! read once and every consumer — pages, export, verification — sees the same
//! bytes and the same attribution.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::projection::{BlockKind, scan};

use super::error::PublicationError;

/// The closed document vocabulary an admission row may name.
const DOCUMENT_KINDS: [&str; 11] = [
    "orientation.project-overview",
    "orientation.research-map",
    "exploration.exploration-brief",
    "exploration.experiment-specification",
    "exploration.experiment-record",
    "understanding.findings-and-analysis",
    "understanding.decision-record",
    "reuse-handoff.technical-reference",
    "reuse-handoff.reproduction-package",
    "reuse-handoff.knowledge-brief",
    "reuse-handoff.handoff",
];

/// One compiled document: its kind, its authored/generated body, and the event
/// ids the body's node ids resolve through.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentRef {
    /// Generated block name, `<family>.<kind>`.
    pub kind: String,
    /// The block body exactly as compiled.
    pub body: String,
    /// Sorted, deduplicated `kaya/...` node ids named in the body.
    pub event_ids: Vec<String>,
}

/// The document source keyed by generated block name (the document kind).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Documents(pub BTreeMap<String, DocumentRef>);

/// Read the compiled documents under `research_dir`.
///
/// Files are read in a fixed order — `index.md`, `overview.md`,
/// `understanding.md`, `reproduction-package.md`, `handoff.md`, then the
/// experiment dossiers sorted by name — and only generated blocks whose
/// names are in the document vocabulary are kept. A repeated block name
/// inside one file is refused, because the file's grammar then admits two
/// answers; across files the first populated block wins. Whitespace-only
/// blocks are skipped, so an empty compilation never shadows a populated one.
/// The block body's `kaya/...` node ids become its event attribution. No
/// ledger access is needed.
pub fn read_documents(research_dir: &Path) -> Result<Documents, PublicationError> {
    let mut documents = BTreeMap::new();
    let mut files: Vec<PathBuf> = [
        "index.md",
        "overview.md",
        "understanding.md",
        "reproduction-package.md",
        "handoff.md",
    ]
    .iter()
    .map(|name| research_dir.join(name))
    .collect();
    files.extend(experiment_files(research_dir)?);
    for path in files {
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(PublicationError::io("read", &path, error)),
        };
        collect_documents(&mut documents, &path, &text)?;
    }
    Ok(Documents(documents))
}

/// The experiment dossiers under `research_dir/experiments`, sorted by path.
///
/// A missing experiments directory contributes no files rather than an error:
/// early research trees legitimately have none yet.
fn experiment_files(research_dir: &Path) -> Result<Vec<PathBuf>, PublicationError> {
    let experiments = research_dir.join("experiments");
    let entries = match fs::read_dir(&experiments) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(PublicationError::io("read", &experiments, error)),
    };
    let mut paths: Vec<PathBuf> = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| PublicationError::io("read", &experiments, error))?;
        let path = entry.path();
        if path.extension().is_some_and(|extension| extension == "md") {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

/// Read compiled documents from an explicit sequence of paths and texts.
///
/// The caller owns the read order, so `kaya check` feeds the documents it
/// expects the projection to emit in exactly the order `read_documents` uses.
/// The parsing rules are shared with [`read_documents`]: only generated blocks
/// in the document vocabulary are kept, a repeated block inside one file is
/// refused, and the first populated block wins across files.
pub fn read_documents_from(
    files: impl IntoIterator<Item = (PathBuf, String)>,
) -> Result<Documents, PublicationError> {
    let mut documents = BTreeMap::new();
    for (path, text) in files {
        collect_documents(&mut documents, &path, &text)?;
    }
    Ok(Documents(documents))
}

/// Render a projection scan failure as a publication refusal naming the file.
fn parse_error(path: &Path, error: crate::projection::ProjectionError) -> PublicationError {
    PublicationError::new(
        super::vocabulary::LABEL_IO,
        format!(
            "failed to parse \"{}\": {}: {}",
            path.display(),
            error.label,
            error.message
        ),
    )
}

/// Collect one compiled document's generated blocks into `documents`.
fn collect_documents(
    documents: &mut BTreeMap<String, DocumentRef>,
    path: &Path,
    text: &str,
) -> Result<(), PublicationError> {
    let blocks = scan(text).map_err(|error| parse_error(path, error))?;
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for block in blocks {
        let BlockKind::Generated(name) = block.kind else {
            continue;
        };
        if !seen.insert(name.clone()) {
            return Err(PublicationError::duplicate_block(path, &name));
        }
        // An empty block is no more a document than a missing one: it must
        // neither back a product nor shadow a populated block later in the
        // read order. The vocabulary and first-wins guards keep the map to
        // known kinds, one rendering per kind.
        if !DOCUMENT_KINDS.contains(&name.as_str())
            || block.body.trim().is_empty()
            || documents.contains_key(&name)
        {
            continue;
        }
        let event_ids = node_ids(&block.body);
        documents.insert(
            name.clone(),
            DocumentRef {
                kind: name,
                body: block.body,
                event_ids,
            },
        );
    }
    Ok(())
}

/// The `kaya/...` node ids named in a block body, sorted and deduplicated.
///
/// `@` stays inside a token because a `Source` reference carries its commit
/// suffix: `kaya/Source/docs-src-index.md@43b606f` is one node id, not the
/// path plus a trailing fragment.
fn node_ids(body: &str) -> Vec<String> {
    let ids: BTreeSet<String> = body
        .split(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '/' | '-' | '_' | '.' | '@')))
        .filter(|token| token.starts_with("kaya/"))
        .map(str::to_owned)
        .collect();
    ids.into_iter().collect()
}
