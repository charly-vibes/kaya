//! The persistent build state that drives incremental regeneration.
//!
//! Responsibilities: record, per compiled document, the content fingerprint
//! of its last successful build plus that build's summary contribution, and
//! answer the skip predicate `cmd_build` consults before invoking a
//! projection. Rationale: a projection whose inputs (ledger digest, generator
//! version, on-disk bytes) are all unchanged produces identical bytes, so
//! re-running its render constructors is wasted work; the state file lets the
//! build restore the document's contribution to the summary without
//! rendering, keeping every envelope truthful. The file is a cache: a missing
//! or unreadable state simply means every document rebuilds.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::ledger::sha256_hex;
use crate::projection::{Summary, report};

/// Bumped whenever a code change could alter rendered bytes beyond the crate
/// version, so stale state cannot skip a document the new code would render.
pub const GENERATOR: &str = concat!(env!("CARGO_PKG_VERSION"), "+incremental-1");

/// The persisted state of the last successful build.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildState {
    /// Schema version of this state file.
    version: u32,
    /// Generator identity: crate version plus a constant bump string.
    generator: String,
    /// Digest of the ledger bytes the build compiled.
    pub ledger_digest: String,
    /// Output-relative document key to its recorded contribution.
    pub documents: BTreeMap<String, DocumentState>,
}

/// One document's contribution to the last successful build.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentState {
    /// Fingerprint of the document bytes (covering absence) after the build.
    pub content_hash: String,
    /// Generated block names this document reports, restored as `unchanged`.
    pub blocks: Vec<String>,
    /// The document's block-to-source-ids map, restored verbatim.
    pub sources: BTreeMap<String, Vec<String>>,
    /// Family-kinds the document omitted, restored verbatim.
    pub omitted: Vec<OmittedEntry>,
}

/// A serializable mirror of [`report::Omitted`] with owned strings.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct OmittedEntry {
    /// Family vocabulary name.
    pub family: String,
    /// Kind vocabulary name.
    pub kind: String,
    /// Why the kind was omitted.
    pub reason: String,
}

impl From<&report::Omitted> for OmittedEntry {
    fn from(omitted: &report::Omitted) -> Self {
        Self {
            family: omitted.family.to_owned(),
            kind: omitted.kind.to_owned(),
            reason: omitted.reason.to_owned(),
        }
    }
}

/// Fingerprint one document (or document family): the digest of its paths and
/// bytes, with absence folded in, so a deleted document never matches.
pub fn fingerprint(documents: &[(String, PathBuf)]) -> String {
    let mut bytes = Vec::new();
    for (name, path) in documents {
        bytes.extend_from_slice(name.as_bytes());
        bytes.push(0);
        match fs::read(path) {
            Ok(content) => bytes.extend_from_slice(&content),
            Err(_) => bytes.extend_from_slice(b"<absent>"),
        }
        bytes.push(0);
    }
    sha256_hex(&bytes)
}

/// Record one document's contribution from a build result and final bytes.
pub fn record(
    documents: &mut BTreeMap<String, DocumentState>,
    key: &str,
    paths: &[(String, PathBuf)],
    summary: &Summary,
    omitted: &[OmittedEntry],
) {
    let mut blocks: Vec<String> = summary.recomputed.clone();
    blocks.extend(summary.unchanged.iter().cloned());
    blocks.sort();
    blocks.dedup();
    documents.insert(
        key.to_owned(),
        DocumentState {
            content_hash: fingerprint(paths),
            blocks,
            sources: summary.sources.clone(),
            omitted: omitted.to_vec(),
        },
    );
}

impl BuildState {
    /// The empty state: no document is skippable.
    pub fn empty(ledger_digest: &str) -> Self {
        Self {
            version: 1,
            generator: GENERATOR.to_owned(),
            ledger_digest: ledger_digest.to_owned(),
            documents: BTreeMap::new(),
        }
    }

    /// Attach the recorded document contributions.
    pub fn with_documents(mut self, documents: BTreeMap<String, DocumentState>) -> Self {
        self.documents = documents;
        self
    }

    /// The loaded state, when it matches this build's generator and ledger.
    pub fn current(saved: Option<Self>, ledger_digest: &str) -> Option<Self> {
        saved.filter(|state| state.generator == GENERATOR && state.ledger_digest == ledger_digest)
    }

    /// The skip predicate: the recorded fingerprint still equals the current
    /// on-disk bytes for `paths`.
    pub fn skippable(&self, key: &str, paths: &[(String, PathBuf)]) -> Option<&DocumentState> {
        let entry = self.documents.get(key)?;
        (entry.content_hash == fingerprint(paths)).then_some(entry)
    }

    /// Load the state file; absence or a corrupt file means nothing to skip.
    pub fn load(path: &Path) -> Option<Self> {
        let bytes = fs::read(path).ok()?;
        serde_json::from_slice(&bytes).ok()
    }

    /// Store the state atomically: write a sibling temp file, then rename.
    ///
    /// A failure between write and rename removes the temp file, so a crash
    /// never leaves an orphaned `build-state.json.tmp` sibling.
    pub fn store(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let bytes = serde_json::to_vec_pretty(self)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string()))?;
        let temp = path.with_extension("json.tmp");
        if let Err(error) = fs::write(&temp, bytes).and_then(|()| fs::rename(&temp, path)) {
            let _ = fs::remove_file(&temp);
            return Err(error);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn sample(digest: &str) -> BuildState {
        let mut documents = BTreeMap::new();
        documents.insert(
            "index.md".to_owned(),
            DocumentState {
                content_hash: "abc".to_owned(),
                blocks: vec!["orientation.research-map".to_owned()],
                sources: BTreeMap::new(),
                omitted: Vec::new(),
            },
        );
        BuildState {
            version: 1,
            generator: GENERATOR.to_owned(),
            ledger_digest: digest.to_owned(),
            documents,
        }
    }

    #[test]
    fn load_store_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("build-state.json");
        let state = sample("digest-1");
        state.store(&path).unwrap();
        assert_eq!(BuildState::load(&path), Some(state));
    }

    #[test]
    fn missing_file_loads_as_none() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("absent").join("build-state.json");
        assert_eq!(BuildState::load(&path), None);
    }

    #[test]
    fn corrupt_file_loads_as_none() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("build-state.json");
        fs::write(&path, b"not json").unwrap();
        assert_eq!(BuildState::load(&path), None);
    }

    #[test]
    fn digest_mismatch_is_not_current() {
        let state = sample("digest-1");
        assert!(BuildState::current(Some(state.clone()), "digest-1").is_some());
        assert!(BuildState::current(Some(state), "digest-2").is_none());
        assert!(BuildState::current(None, "digest-1").is_none());
    }

    #[test]
    fn fingerprint_covers_absence_and_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("doc.md");
        let single = vec![("doc.md".to_owned(), path.clone())];
        let absent = fingerprint(&single);
        fs::write(&path, b"bytes").unwrap();
        assert_ne!(fingerprint(&single), absent);
        assert_eq!(fingerprint(&single), fingerprint(&single));
    }

    #[test]
    fn skippable_requires_matching_fingerprint() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("doc.md");
        fs::write(&path, b"bytes").unwrap();
        let paths = vec![("doc.md".to_owned(), path.clone())];
        let mut state = sample("digest-1");
        record(
            &mut state.documents,
            "doc.md",
            &paths,
            &Summary {
                unchanged: vec!["b".to_owned()],
                ..Summary::default()
            },
            &[],
        );
        assert!(state.skippable("doc.md", &paths).is_some());
        fs::write(&path, b"changed").unwrap();
        assert!(state.skippable("doc.md", &paths).is_none());
        assert!(state.skippable("other.md", &paths).is_none());
    }
}
