//! The document IR: the typed artifact between the ledger-model state and
//! every rendered projection.
//!
//! The compiler materializes the IR from grammar-validated document sources
//! and its own rendering results, then writes rendered bytes from it and
//! derives the emitted source map from record provenance, so structural
//! consumers never reparse rendered markdown to recover records or
//! dependencies. Records carry a stable id, a content hash, and their
//! provenance; records cover generated documents (family and kind), authored
//! blocks — referenced by document, block, region hash, and cited ids, never
//! copied — and composed records holding containment edges to the records
//! they contain. Materialization is deterministic: identical inputs yield an
//! identical IR, and nothing outside the document sources and the caller's
//! rendering results feeds it, so build caches and stale sidecars cannot
//! change it.

use std::collections::BTreeMap;
use std::path::Path;

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::doctest;

use super::report;
use super::{BlockKind, Summary, scan};

/// A grammar-validated document source, by its stable path and bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceDocument {
    /// The stable path the record ids are qualified with.
    pub path: String,
    /// The document bytes.
    pub bytes: String,
}

/// One rendered generated block: its name, bytes, and attributed provenance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RenderedBlock {
    /// The generated block name (`<family>.<kind>`).
    pub name: String,
    /// The rendered block bytes.
    pub body: String,
    /// The event or node ids backing the block's facts, as the walk
    /// attributed them.
    pub provenance: Vec<String>,
}

/// The rendered generated blocks of one document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RenderedDocument {
    /// The stable path the record ids are qualified with.
    pub path: String,
    /// Every generated block the projection renders for this document.
    pub blocks: Vec<RenderedBlock>,
}

/// What a record stands for in the document IR.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum RecordKind {
    /// A compiler-generated document block, named `<family>.<kind>`.
    GeneratedDocument {
        /// The closed family of the block.
        family: String,
        /// The closed kind of the block.
        kind: String,
    },
    /// A human-maintained block, referenced — never copied.
    AuthoredBlock,
    /// A declared doctest execution contract, extracted from authored prose.
    Doctest {
        /// The controlled runner that executes the contract.
        runner: String,
    },
    /// A composed record (publication page or collection) over other records.
    Composed,
}

/// One record of the document IR.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Record {
    /// `<document path>#<block>` for document records; the caller's page id
    /// for composed records.
    pub id: String,
    /// What the record stands for.
    pub kind: RecordKind,
    /// SHA-256 of the block's bytes (or of the canonical containment list).
    pub content_hash: String,
    /// The sorted, deduplicated event or node ids backing the record's facts.
    pub provenance: Vec<String>,
    /// The sorted, deduplicated ids an authored block cites.
    pub cites: Vec<String>,
    /// Source artifact references the record depends on.
    pub artifacts: Vec<String>,
    /// The sorted, deduplicated record ids a composed record contains.
    pub contains: Vec<String>,
}

/// The document IR: every record, by stable id.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct DocumentIr {
    /// All records, ordered by id.
    pub records: BTreeMap<String, Record>,
}

/// Sort and deduplicate a provenance or citation list.
fn canonical(mut ids: Vec<String>) -> Vec<String> {
    ids.sort();
    ids.dedup();
    ids
}

/// The provenance a summary attributes to one generated block.
///
/// Project-document summaries key the source map by bare block name; the
/// experiment report namespaces its keys as `<block>:<experiment id>` so
/// reports for different experiments stay distinct when summaries merge. A
/// block's provenance is the union of its bare entry and every entry its
/// name namespaces.
pub fn provenance_of(summary: &Summary, block: &str) -> Vec<String> {
    let mut ids = Vec::new();
    let namespaced = format!("{block}:");
    for (key, values) in &summary.sources {
        if key == block || key.starts_with(&namespaced) {
            ids.extend(values.iter().cloned());
        }
    }
    canonical(ids)
}

/// SHA-256 of `bytes`, hex-encoded.
fn hash(bytes: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Materialize the document IR.
///
/// `sources` must be grammar-validated documents (the compiler validates them
/// before planning; an invalid document cannot reach this function). Only
/// authored blocks are taken from `sources`; generated records come from
/// `rendered` — the compiler's own walk output — so the IR carries exactly
/// what the projections rendered, with provenance canonicalized to sorted,
/// deduplicated ids. `composed` maps a caller-chosen page id to the record
/// ids the page contains.
pub fn materialize(
    sources: &[SourceDocument],
    rendered: &[RenderedDocument],
    composed: &BTreeMap<String, Vec<String>>,
) -> DocumentIr {
    let mut records = BTreeMap::new();

    for source in sources {
        let blocks = scan(&source.bytes).unwrap_or_else(|error| {
            panic!(
                "materialize requires grammar-validated sources; {} broke the grammar: {error}",
                source.path
            )
        });
        let mut authored_count = 0usize;
        for block in blocks {
            let Some(name) = authored_name(&block.kind, &mut authored_count) else {
                continue;
            };
            let id = format!("{}#{name}", source.path);
            let cites = canonical(
                report::citation_markers(&block.body)
                    .into_iter()
                    .flat_map(|(_, ids)| ids)
                    .collect(),
            );
            records.insert(
                id.clone(),
                Record {
                    id,
                    kind: RecordKind::AuthoredBlock,
                    content_hash: hash(&block.body),
                    provenance: Vec::new(),
                    cites,
                    artifacts: Vec::new(),
                    contains: Vec::new(),
                },
            );
        }
        for contract in doctest::extract(&source.bytes).unwrap_or_else(|error| {
            panic!(
                "materialize requires grammar-valid doctest declarations; {} refused: {error}",
                source.path
            )
        }) {
            let id = format!("{}#{}", source.path, contract.label);
            records.insert(
                id.clone(),
                Record {
                    id,
                    kind: RecordKind::Doctest {
                        runner: contract.runner.as_str().to_owned(),
                    },
                    content_hash: doctest::contract_hash(&contract.body),
                    provenance: Vec::new(),
                    cites: Vec::new(),
                    artifacts: Vec::new(),
                    contains: Vec::new(),
                },
            );
        }
    }

    for document in rendered {
        for block in &document.blocks {
            let kind = report::classify_block(&block.name).unwrap_or_else(|error| {
                panic!(
                    "materialize requires closed-vocabulary block names; {} named {}: {error}",
                    document.path, block.name
                )
            });
            let id = format!("{}#{}", document.path, block.name);
            let record = Record {
                id: id.clone(),
                kind: RecordKind::GeneratedDocument {
                    family: kind.family().as_str().to_string(),
                    kind: kind.as_str().to_string(),
                },
                content_hash: hash(&block.body),
                provenance: canonical(block.provenance.clone()),
                cites: Vec::new(),
                artifacts: Vec::new(),
                contains: Vec::new(),
            };
            records.insert(id, record);
        }
    }

    for (page, contains) in composed {
        let contains = canonical(contains.clone());
        let content_hash = hash(&contains.join("\n"));
        records.insert(
            page.clone(),
            Record {
                id: page.clone(),
                kind: RecordKind::Composed,
                content_hash,
                provenance: Vec::new(),
                cites: Vec::new(),
                artifacts: Vec::new(),
                contains,
            },
        );
    }

    DocumentIr { records }
}

/// Materialize the document IR for one planned document and canonicalize the
/// summary's source map to the record provenance.
///
/// The projection's planned bytes carry the generated blocks (re-scanned
/// here) and the authored blocks (referenced, never copied); the summary's
/// source map is sorted and deduplicated in place so the emitted map equals
/// the record provenance.
pub(crate) fn for_document(path: &Path, document: &str, summary: &mut Summary) -> DocumentIr {
    for ids in summary.sources.values_mut() {
        ids.sort();
        ids.dedup();
    }
    let path_string = path.to_string_lossy().to_string();
    let blocks = scan(document)
        .unwrap_or_else(|error| {
            panic!(
                "for_document requires grammar-validated documents; {} broke the grammar: {error}",
                path_string
            )
        })
        .into_iter()
        .filter_map(|block| match block.kind {
            BlockKind::Generated(name) => {
                let provenance = provenance_of(summary, &name);
                Some(RenderedBlock {
                    name,
                    body: block.body,
                    provenance,
                })
            }
            BlockKind::Authored => None,
        })
        .collect();
    materialize(
        &[SourceDocument {
            path: path_string.clone(),
            bytes: document.to_string(),
        }],
        &[RenderedDocument {
            path: path_string,
            blocks,
        }],
        &BTreeMap::new(),
    )
}

/// The stable name of an authored block: `authored`, then `authored-2`, …
///
/// Generated blocks return `None`; they are recorded from `rendered`.
fn authored_name(kind: &BlockKind, count: &mut usize) -> Option<String> {
    if matches!(kind, BlockKind::Generated(_)) {
        return None;
    }
    *count += 1;
    Some(if *count == 1 {
        "authored".to_string()
    } else {
        format!("authored-{count}")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const AUTHORED_BEGIN: &str = "<!-- kaya:authored -->";
    const AUTHORED_END: &str = "<!-- kaya:end authored -->";

    fn authored_document(prose: &str) -> String {
        format!("# Notes\n\n{AUTHORED_BEGIN}\n{prose}\n{AUTHORED_END}\n")
    }

    #[test]
    fn authored_cites_are_canonical() {
        let sources = vec![SourceDocument {
            path: "docs/research/index.md".to_string(),
            bytes: authored_document("Prose.\n<!-- kaya:cites b a b -->"),
        }];
        let materialized = materialize(&sources, &[], &BTreeMap::new());
        let record = materialized
            .records
            .get("docs/research/index.md#authored")
            .expect("the authored record exists");
        assert_eq!(record.cites, vec!["a".to_string(), "b".to_string()]);
        assert!(record.provenance.is_empty());
    }

    #[test]
    fn composed_hash_follows_containment() {
        let mut composed = BTreeMap::new();
        composed.insert(
            "publication#overview".to_string(),
            vec!["docs/research/overview.md#orientation.project-overview".to_string()],
        );
        let materialized = materialize(&[], &[], &composed);
        let page = materialized
            .records
            .get("publication#overview")
            .expect("the composed record exists");
        assert_eq!(page.contains.len(), 1);
        assert_ne!(page.content_hash, hash(""));
    }

    #[test]
    fn declared_doctest_is_a_first_class_record() {
        let sources = vec![SourceDocument {
            path: "docs/research/index.md".to_string(),
            bytes: authored_document("Prose.\n\n```doctest shell\n$ echo hello\nhello\n```\n"),
        }];
        let materialized = materialize(&sources, &[], &BTreeMap::new());
        let record = materialized
            .records
            .get("docs/research/index.md#doctest-1")
            .expect("the doctest record exists");
        assert_eq!(
            record.kind,
            RecordKind::Doctest {
                runner: "shell".to_string()
            }
        );
        assert_eq!(
            record.content_hash,
            crate::doctest::contract_hash("$ echo hello\nhello\n")
        );
        assert!(
            materialized
                .records
                .contains_key("docs/research/index.md#authored"),
            "the authored record keeps its own record beside the doctest"
        );
    }

    #[test]
    fn doctest_label_is_stable() {
        let sources = vec![SourceDocument {
            path: "docs/research/index.md".to_string(),
            bytes: authored_document(
                "```doctest shell #build-check\n$ echo hello\nhello\n```\n",
            ),
        }];
        let materialized = materialize(&sources, &[], &BTreeMap::new());
        assert!(
            materialized
                .records
                .contains_key("docs/research/index.md#build-check"),
            "the declared label names the record: {:?}",
            materialized.records.keys().collect::<Vec<_>>()
        );
    }

    #[test]
    fn undeclared_code_stays_prose() {
        let sources = vec![SourceDocument {
            path: "docs/research/index.md".to_string(),
            bytes: authored_document("```bash\n$ echo hello\nhello\n```\n"),
        }];
        let materialized = materialize(&sources, &[], &BTreeMap::new());
        assert!(
            !materialized
                .records
                .keys()
                .any(|id| id.contains("doctest")),
            "no doctest record for an undeclared fence: {:?}",
            materialized.records.keys().collect::<Vec<_>>()
        );
        assert!(
            materialized
                .records
                .contains_key("docs/research/index.md#authored"),
            "the fence stays inside the authored record"
        );
    }
}
