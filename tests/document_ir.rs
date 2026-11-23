//! Integration tests for the document IR materialization.
//!
//! Contract (kaya-projection, add-document-ir-commit-gate): the compiler
//! materializes a deterministic document IR between the ledger-model state and
//! every rendered projection. Each record carries a stable id, a content hash,
//! and its event ids; records cover generated documents (family and kind),
//! authored blocks, and composed records; edges link a record to the event
//! ids, cited ids, source artifacts, and contained records it depends on; the
//! emitted source map for a rendered block is exactly that record's
//! provenance; authored blocks are referenced (document, block, region hash,
//! cited ids) and never copied; and nothing outside the document sources and
//! the ledger feeds the IR, so build caches and stale sidecars cannot change
//! it. Structural consumers read the IR; rendered markdown is never reparsed
//! to recover records or dependencies.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;

use assert_cmd::Command as AssertCommand;
use sha2::{Digest, Sha256};
use tempfile::TempDir;

use kaya::model::{EntityType, Graph, GraphBuilder, Relationship};
use kaya::projection::ir::{self, RecordKind, RenderedBlock, RenderedDocument, SourceDocument};
use kaya::projection::report;
use kaya::promotion::{Assertion, PromotionState, Status};

const AUTHORED_BEGIN: &str = "<!-- kaya:authored -->";
const AUTHORED_END: &str = "<!-- kaya:end authored -->";
const RECORD_BLOCK: &str = "exploration.experiment-record";

fn node(builder: &mut GraphBuilder, id: &str, entity: EntityType) {
    builder
        .entity(id, entity, serde_json::json!({}))
        .expect("node id is namespace-qualified");
}

fn edge(builder: &mut GraphBuilder, relationship: Relationship, from: &str, to: &str) {
    builder
        .relationship(relationship, from, to)
        .expect("relationship direction follows the grammar");
}

/// One experiment with declared intent and one executed, observed run.
fn graph() -> Graph {
    let mut builder = GraphBuilder::new();
    node(&mut builder, "p/Experiment/exp1", EntityType::Experiment);
    node(&mut builder, "p/Question/q1", EntityType::Question);
    node(&mut builder, "p/Hypothesis/h1", EntityType::Hypothesis);
    node(&mut builder, "p/Run/r1", EntityType::Run);
    node(&mut builder, "p/Observation/o1", EntityType::Observation);
    node(&mut builder, "p/Finding/f1", EntityType::Finding);
    edge(
        &mut builder,
        Relationship::Investigates,
        "p/Question/q1",
        "p/Experiment/exp1",
    );
    edge(
        &mut builder,
        Relationship::Predicts,
        "p/Hypothesis/h1",
        "p/Experiment/exp1",
    );
    edge(
        &mut builder,
        Relationship::Executes,
        "p/Run/r1",
        "p/Experiment/exp1",
    );
    edge(
        &mut builder,
        Relationship::Observes,
        "p/Observation/o1",
        "p/Run/r1",
    );
    builder.build().expect("graph is acyclic and complete")
}

/// A promotion state with one human assertion whose evidence ids are known.
fn promotion() -> PromotionState {
    PromotionState {
        assertions: vec![Assertion {
            id: "assertion-1".to_string(),
            text: "the effect is real".to_string(),
            status: Status::Proposed,
            machine_proposed: false,
            event_ids: vec!["assertion-1-ev".to_string()],
            author: "alice".to_string(),
        }],
    }
}

fn sha256(bytes: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// A document source holding one authored block with the given prose.
fn authored_document(prose: &str) -> String {
    format!("# Research notes\n\n{AUTHORED_BEGIN}\n{prose}\n{AUTHORED_END}\n")
}

/// The bytes strictly between the authored markers of `document`.
fn authored_region(document: &str) -> String {
    let start = document
        .find(AUTHORED_BEGIN)
        .expect("the fixture has an authored marker")
        + AUTHORED_BEGIN.len()
        + 1;
    let end = document
        .find(AUTHORED_END)
        .expect("the fixture closes the authored block");
    document[start..end].to_string()
}

fn source(path: &str, bytes: &str) -> SourceDocument {
    SourceDocument {
        path: path.to_string(),
        bytes: bytes.to_string(),
    }
}

fn rendered(path: &str, block: &str, body: &str, provenance: &[&str]) -> RenderedDocument {
    RenderedDocument {
        path: path.to_string(),
        blocks: vec![RenderedBlock {
            name: block.to_string(),
            body: body.to_string(),
            provenance: provenance.iter().map(|id| id.to_string()).collect(),
        }],
    }
}

/// Generated records in the IR, by block name.
fn generated_records(ir: &ir::DocumentIr) -> Vec<(&String, &ir::Record)> {
    ir.records
        .iter()
        .filter(|(_, record)| matches!(record.kind, RecordKind::GeneratedDocument { .. }))
        .collect()
}

#[test]
fn ir_materializes_deterministically() {
    let sources = vec![source(
        "docs/research/notes.md",
        &authored_document("Human interpretation."),
    )];
    let rendered = vec![rendered(
        "docs/research/notes.md",
        RECORD_BLOCK,
        "- run r1 observed",
        &["p/Run/r1", "p/Observation/o1"],
    )];
    let composed = BTreeMap::new();

    let first = ir::materialize(&sources, &rendered, &composed);
    let second = ir::materialize(&sources, &rendered, &composed);
    assert_eq!(first, second, "identical state yields an identical IR");
}

#[test]
fn generated_provenance_equals_the_source_map() {
    let dir = tempfile::tempdir().expect("temp workspace");
    let path = dir.path().join("report.md");
    fs::write(&path, authored_document("Human interpretation.")).expect("seed the authored report");

    let built = report::build(&graph(), "p/Experiment/exp1", &path, &promotion())
        .expect("the report builds");

    assert!(
        !generated_records(&built.ir).is_empty(),
        "the report carries generated records"
    );
    for (id, record) in generated_records(&built.ir) {
        let block = id.rsplit('#').next().expect("the id names its block");
        // The emitted source map keys project documents by bare block name
        // and experiment reports by `<block>:<experiment id>`; the record's
        // provenance is exactly the union of the block's entries.
        let mut expected: Vec<&String> = built
            .summary
            .sources
            .iter()
            .filter(|(key, _)| {
                key.as_str() == block
                    || key
                        .strip_prefix(block)
                        .is_some_and(|rest| rest.starts_with(':'))
            })
            .flat_map(|(_, ids)| ids.iter())
            .collect();
        expected.sort();
        expected.dedup();
        let expected: Vec<String> = expected.into_iter().cloned().collect();
        assert_eq!(
            record.provenance, expected,
            "record {id} provenance is exactly the emitted source map entry"
        );
    }
}

#[test]
fn provenance_names_assertion_event_ids() {
    let dir = tempfile::tempdir().expect("temp workspace");
    let path = dir.path().join("report.md");
    fs::write(&path, authored_document("Human interpretation.")).expect("seed the authored report");

    let built = report::build(&graph(), "p/Experiment/exp1", &path, &promotion())
        .expect("the report builds");

    let mut all: Vec<&String> = built
        .ir
        .records
        .values()
        .flat_map(|record| record.provenance.iter())
        .collect();
    all.sort();
    all.dedup();
    assert!(
        all.contains(&&"assertion-1-ev".to_string()),
        "the record provenance names the assertion's event id: {all:?}"
    );
}

#[test]
fn authored_record_references_not_copies() {
    let prose = "Human interpretation of the record.\n<!-- kaya:cites evt-aaa evt-bbb -->";
    let dir = tempfile::tempdir().expect("temp workspace");
    let path = dir.path().join("report.md");
    let document = authored_document(prose);
    fs::write(&path, &document).expect("seed the authored report");

    let built = report::build(&graph(), "p/Experiment/exp1", &path, &promotion())
        .expect("the report builds");

    let authored: Vec<&ir::Record> = built
        .ir
        .records
        .values()
        .filter(|record| matches!(record.kind, RecordKind::AuthoredBlock))
        .collect();
    assert_eq!(authored.len(), 1, "exactly one authored record");
    let record = authored[0];
    assert_eq!(
        record.cites,
        vec!["evt-aaa".to_string(), "evt-bbb".to_string()],
        "the authored record declares its cited evidence"
    );
    assert_eq!(
        record.content_hash,
        sha256(&authored_region(&document)),
        "the authored record hashes its region bytes"
    );

    let serialized = serde_json::to_string(&record).expect("records serialize");
    assert!(
        !serialized.contains("Human interpretation"),
        "the authored record stores no authored bytes: {serialized}"
    );
}

#[test]
fn authored_hash_tracks_region_bytes() {
    let first_prose = "First interpretation.";
    let second_prose = "Second interpretation.";
    let dir = tempfile::tempdir().expect("temp workspace");
    let path = dir.path().join("report.md");

    fs::write(&path, authored_document(first_prose)).expect("seed the authored report");
    let first = report::build(&graph(), "p/Experiment/exp1", &path, &promotion())
        .expect("the report builds");
    fs::write(&path, authored_document(second_prose)).expect("change the authored prose");
    let second = report::build(&graph(), "p/Experiment/exp1", &path, &promotion())
        .expect("the report rebuilds");

    let hash = |built: &report::Built| {
        built
            .ir
            .records
            .values()
            .find(|record| matches!(record.kind, RecordKind::AuthoredBlock))
            .map(|record| record.content_hash.clone())
            .expect("an authored record exists")
    };
    assert_ne!(
        hash(&first),
        hash(&second),
        "changing the authored region changes its hash"
    );
}

#[test]
fn cache_outside_sources_leaves_ir_identical() {
    let fixture = CliFixture::new();
    let first = fixture.ir();
    fs::write(fixture.root().join(".kaya/ir-cache.json"), "{\"junk\":1}")
        .expect("plant a fake build cache");
    let public = fixture.root().join("docs/public/junk-page.md");
    fs::create_dir_all(public.parent().expect("the copy has a parent"))
        .expect("create the rendered-copy directory");
    fs::write(&public, "a rendered copy outside the document sources")
        .expect("plant a rendered copy");
    let second = fixture.ir();
    assert_eq!(
        first, second,
        "caches and copies outside the sources cannot change the IR"
    );
}

#[test]
fn composed_record_holds_document_records() {
    let sources = vec![source(
        "docs/research/overview.md",
        &authored_document("Overview prose."),
    )];
    let rendered = vec![rendered(
        "docs/research/overview.md",
        "orientation.project-overview",
        "- project p",
        &["p/Question/q1"],
    )];
    let mut composed = BTreeMap::new();
    composed.insert(
        "publication#overview".to_string(),
        vec!["docs/research/overview.md#orientation.project-overview".to_string()],
    );

    let materialized = ir::materialize(&sources, &rendered, &composed);
    let page = materialized
        .records
        .get("publication#overview")
        .expect("the composed page record exists");
    assert!(
        matches!(page.kind, RecordKind::Composed),
        "the page record is composed"
    );
    assert_eq!(
        page.contains,
        vec!["docs/research/overview.md#orientation.project-overview".to_string()],
        "the composed record holds its containment edges"
    );
    assert!(
        materialized
            .records
            .contains_key("docs/research/overview.md#orientation.project-overview"),
        "the contained document record exists"
    );
}

/// A committed record workspace driving the built binary.
struct CliFixture {
    dir: TempDir,
}

impl CliFixture {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("temp workspace");
        let root = dir.path();
        Self::git(root, &["init", "--quiet"]);
        Self::git(root, &["config", "user.name", "kaya document ir"]);
        Self::git(root, &["config", "user.email", "ir@example.invalid"]);
        Self::run_ok(root, &["init"]);
        let seed = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(".kaya/seeds/add-core-research-record.jsonl");
        Self::run_ok(
            root,
            &[
                "import",
                "model",
                "--file",
                seed.to_str().expect("seed path is UTF-8"),
            ],
        );
        // `build` verifies the ledger against git HEAD, so the seed commits
        // before the first build.
        Self::git(root, &["add", "--all"]);
        Self::git(
            root,
            &["commit", "--quiet", "--message", "record the model seed"],
        );
        let index = root.join("docs/research/index.md");
        fs::create_dir_all(index.parent().expect("the index has a parent"))
            .expect("create the research directory");
        fs::write(
            &index,
            authored_document("Human interpretation of the record."),
        )
        .expect("seed the authored index");
        Self::run_ok(root, &["build"]);
        Self::git(root, &["add", "--all"]);
        Self::git(
            root,
            &["commit", "--quiet", "--message", "commit the record"],
        );
        Self { dir }
    }

    fn root(&self) -> &Path {
        self.dir.path()
    }

    fn ir(&self) -> String {
        let output = AssertCommand::cargo_bin("kaya")
            .expect("kaya binary is built")
            .current_dir(self.root())
            .args(["ir"])
            .output()
            .expect("kaya ir runs");
        assert!(
            output.status.success(),
            "kaya ir exits zero: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).expect("kaya ir prints UTF-8")
    }

    fn run_ok(root: &Path, args: &[&str]) {
        let output = AssertCommand::cargo_bin("kaya")
            .expect("kaya binary is built")
            .current_dir(root)
            .args(args)
            .output()
            .expect("kaya runs");
        assert!(
            output.status.success(),
            "kaya {args:?} exits zero: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn git(root: &Path, args: &[&str]) {
        let status = StdCommand::new("git")
            .current_dir(root)
            .args(args)
            .status()
            .expect("git runs");
        assert!(status.success(), "git {args:?} succeeds");
    }
}
