//! Contract tests for the understanding projection: what the
//! `understanding.findings-and-analysis` and `understanding.decision-record`
//! blocks may render, how evidence is cited, omission of empty kinds, and
//! preservation of authored bytes.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use tempfile::TempDir;

use kaya::model::{EntityType, Graph, GraphBuilder, Relationship};
use kaya::projection::report::{
    DECISION_BLOCK, FINDINGS_BLOCK, Kind, LABEL_LEGACY_BLOCK, Omitted, REASON_NO_RECORDS,
    attribution_stale_row, build_understanding,
};

const UNDERSTANDING: &str = "docs/research/understanding.md";

fn node(builder: &mut GraphBuilder, id: &str, entity: EntityType, attributes: Value) {
    builder
        .entity(id, entity, attributes)
        .expect("node id is namespace-qualified");
}

fn bare(builder: &mut GraphBuilder, id: &str, entity: EntityType) {
    node(builder, id, entity, json!({}));
}

fn titled(builder: &mut GraphBuilder, id: &str, entity: EntityType, title: &str) {
    node(builder, id, entity, json!({ "title": title }));
}

fn edge(builder: &mut GraphBuilder, relationship: Relationship, from: &str, to: &str) {
    builder
        .relationship(relationship, from, to)
        .expect("relationship direction follows the grammar");
}

/// The staleness marker a decision without `cites` edges carries: global
/// attribution is never mistaken for per-decision scoping.
fn stale_marker(decision_id: &str) -> String {
    attribution_stale_row(decision_id)
}

/// Findings, observations, and claims alongside records that the understanding
/// family must never render: a run and the artifact it produced.
fn graph_with_observed_records_only() -> Graph {
    let mut builder = GraphBuilder::new();
    // Insertion order is deliberately not id order.
    titled(
        &mut builder,
        "p/Finding/f2",
        EntityType::Finding,
        "Second finding",
    );
    titled(
        &mut builder,
        "p/Finding/f1",
        EntityType::Finding,
        "First finding",
    );
    titled(
        &mut builder,
        "p/Observation/o1",
        EntityType::Observation,
        "Latency fell",
    );
    titled(
        &mut builder,
        "p/Claim/c1",
        EntityType::Claim,
        "Cache is cold",
    );
    // A claim without a title renders its key.
    bare(&mut builder, "p/Claim/c2", EntityType::Claim);
    titled(&mut builder, "p/Run/r1", EntityType::Run, "run one");
    node(
        &mut builder,
        "p/Artifact/a1",
        EntityType::Artifact,
        json!({ "kind": "reports", "path": "target/report.json" }),
    );
    builder.build().expect("graph is acyclic and complete")
}

/// The decision record's sources: decisions, the findings they influenced, and
/// the source revision backing them.
fn graph_with_decisions_and_evidence() -> Graph {
    let mut builder = GraphBuilder::new();
    titled(
        &mut builder,
        "p/Decision/d2",
        EntityType::Decision,
        "Defer the migration",
    );
    titled(
        &mut builder,
        "p/Decision/d1",
        EntityType::Decision,
        "Ship the cache",
    );
    titled(
        &mut builder,
        "p/Finding/f1",
        EntityType::Finding,
        "Cache hit rate rose",
    );
    // A finding without a title renders its key.
    bare(&mut builder, "p/Finding/f2", EntityType::Finding);
    node(
        &mut builder,
        "p/Source/src2",
        EntityType::Source,
        json!({ "path": "docs/notes.md", "commit": "def456" }),
    );
    node(
        &mut builder,
        "p/Source/src1",
        EntityType::Source,
        json!({ "path": "src/cache.rs", "commit": "abc123" }),
    );

    // One finding influenced by two decisions keeps one evidence row per
    // reference, ordered by id then reference.
    edge(
        &mut builder,
        Relationship::Influences,
        "p/Decision/d1",
        "p/Finding/f1",
    );
    edge(
        &mut builder,
        Relationship::Influences,
        "p/Decision/d2",
        "p/Finding/f1",
    );
    edge(
        &mut builder,
        Relationship::Influences,
        "p/Decision/d1",
        "p/Finding/f2",
    );
    builder.build().expect("graph is acyclic and complete")
}

/// Two decisions whose evidence must be scoped: each cites its own Source,
/// one Source is cited by both decisions, one complete Source is cited by
/// nothing, and one incomplete Source is cited (a cite cannot resurrect a
/// missing `path` or `commit` half).
fn graph_with_scoped_decision_evidence() -> Graph {
    let mut builder = GraphBuilder::new();
    titled(
        &mut builder,
        "p/Decision/d1",
        EntityType::Decision,
        "Ship the cache",
    );
    titled(
        &mut builder,
        "p/Decision/d2",
        EntityType::Decision,
        "Defer the migration",
    );
    titled(
        &mut builder,
        "p/Finding/f1",
        EntityType::Finding,
        "Cache hit rate rose",
    );
    node(
        &mut builder,
        "p/Source/src1",
        EntityType::Source,
        json!({ "path": "src/cache.rs", "commit": "abc123" }),
    );
    node(
        &mut builder,
        "p/Source/src2",
        EntityType::Source,
        json!({ "path": "docs/notes.md", "commit": "def456" }),
    );
    // Cited by both decisions: one evidence row per citing decision.
    node(
        &mut builder,
        "p/Source/src3",
        EntityType::Source,
        json!({ "path": "shared/notes.md", "commit": "789aaa" }),
    );
    // Complete but cited by no decision: no row, no source-map entry.
    node(
        &mut builder,
        "p/Source/src4",
        EntityType::Source,
        json!({ "path": "reports/x.md", "commit": "111bbb" }),
    );
    // Cited but incomplete: still skipped entirely.
    node(
        &mut builder,
        "p/Source/src5",
        EntityType::Source,
        json!({ "path": "src/only-path.rs" }),
    );
    edge(
        &mut builder,
        Relationship::Influences,
        "p/Decision/d1",
        "p/Finding/f1",
    );
    edge(
        &mut builder,
        Relationship::Cites,
        "p/Decision/d1",
        "p/Source/src1",
    );
    edge(
        &mut builder,
        Relationship::Cites,
        "p/Decision/d2",
        "p/Source/src2",
    );
    edge(
        &mut builder,
        Relationship::Cites,
        "p/Decision/d1",
        "p/Source/src3",
    );
    edge(
        &mut builder,
        Relationship::Cites,
        "p/Decision/d2",
        "p/Source/src3",
    );
    edge(
        &mut builder,
        Relationship::Cites,
        "p/Decision/d1",
        "p/Source/src5",
    );
    builder.build().expect("graph is acyclic and complete")
}

/// One decision cites a Source; a second decision cites nothing, so its own
/// evidence stays ledger-global and is marked.
fn graph_with_mixed_decision_evidence() -> Graph {
    let mut builder = GraphBuilder::new();
    titled(
        &mut builder,
        "p/Decision/d1",
        EntityType::Decision,
        "Ship the cache",
    );
    titled(
        &mut builder,
        "p/Decision/d2",
        EntityType::Decision,
        "Defer the migration",
    );
    node(
        &mut builder,
        "p/Source/src1",
        EntityType::Source,
        json!({ "path": "src/cache.rs", "commit": "abc123" }),
    );
    node(
        &mut builder,
        "p/Source/src2",
        EntityType::Source,
        json!({ "path": "docs/notes.md", "commit": "def456" }),
    );
    edge(
        &mut builder,
        Relationship::Cites,
        "p/Decision/d1",
        "p/Source/src1",
    );
    builder.build().expect("graph is acyclic and complete")
}

/// A decision with no Source at all: the record renders no evidence section.
fn graph_with_decisions_only() -> Graph {
    let mut builder = GraphBuilder::new();
    titled(
        &mut builder,
        "p/Decision/d1",
        EntityType::Decision,
        "Ship the cache",
    );
    builder.build().expect("graph is acyclic and complete")
}

/// One decision citing the same Source twice and an incomplete Source: the
/// duplicate renders one row, the incomplete citation renders none.
fn graph_with_duplicate_and_incomplete_citations() -> Graph {
    let mut builder = GraphBuilder::new();
    titled(
        &mut builder,
        "p/Decision/d1",
        EntityType::Decision,
        "Ship the cache",
    );
    node(
        &mut builder,
        "p/Source/src1",
        EntityType::Source,
        json!({ "path": "src/cache.rs", "commit": "abc123" }),
    );
    node(
        &mut builder,
        "p/Source/src2",
        EntityType::Source,
        json!({ "path": "src/only-path.rs" }),
    );
    edge(
        &mut builder,
        Relationship::Cites,
        "p/Decision/d1",
        "p/Source/src1",
    );
    edge(
        &mut builder,
        Relationship::Cites,
        "p/Decision/d1",
        "p/Source/src1",
    );
    edge(
        &mut builder,
        Relationship::Cites,
        "p/Decision/d1",
        "p/Source/src2",
    );
    builder.build().expect("graph is acyclic and complete")
}

/// One finding: the findings block renders and the decision record is empty.
fn graph_with_findings_only() -> Graph {
    let mut builder = GraphBuilder::new();
    titled(
        &mut builder,
        "p/Finding/f1",
        EntityType::Finding,
        "Only finding",
    );
    builder.build().expect("graph is valid")
}

/// Every understanding kind has records, so both blocks render.
fn graph_with_both_kinds() -> Graph {
    let mut builder = GraphBuilder::new();
    titled(
        &mut builder,
        "p/Decision/d1",
        EntityType::Decision,
        "Ship the cache",
    );
    titled(
        &mut builder,
        "p/Finding/f1",
        EntityType::Finding,
        "Cache hit rate rose",
    );
    titled(
        &mut builder,
        "p/Observation/o1",
        EntityType::Observation,
        "Latency fell",
    );
    titled(
        &mut builder,
        "p/Claim/c1",
        EntityType::Claim,
        "Cold caches read slower",
    );
    node(
        &mut builder,
        "p/Source/src1",
        EntityType::Source,
        json!({ "path": "src/cache.rs", "commit": "abc123" }),
    );
    edge(
        &mut builder,
        Relationship::Influences,
        "p/Decision/d1",
        "p/Finding/f1",
    );
    builder.build().expect("graph is acyclic and complete")
}

/// A decision backed by one complete Source and an assortment of Sources that
/// cannot be cited: only path, only commit, blank parts, and non-string parts.
fn graph_with_incomplete_sources() -> Graph {
    let mut builder = GraphBuilder::new();
    titled(
        &mut builder,
        "p/Decision/d1",
        EntityType::Decision,
        "Ship the cache",
    );
    node(
        &mut builder,
        "p/Source/src1",
        EntityType::Source,
        json!({ "path": "src/cache.rs", "commit": "abc123" }),
    );
    // Missing half the reference.
    node(
        &mut builder,
        "p/Source/src2",
        EntityType::Source,
        json!({ "path": "src/only-path.rs" }),
    );
    node(
        &mut builder,
        "p/Source/src3",
        EntityType::Source,
        json!({ "commit": "def456" }),
    );
    // Present but blank.
    node(
        &mut builder,
        "p/Source/src4",
        EntityType::Source,
        json!({ "path": "   ", "commit": "abc123" }),
    );
    node(
        &mut builder,
        "p/Source/src5",
        EntityType::Source,
        json!({ "path": "src/blank-commit.rs", "commit": "  " }),
    );
    // Present but not a string.
    node(
        &mut builder,
        "p/Source/src6",
        EntityType::Source,
        json!({ "path": 7, "commit": "abc123" }),
    );
    node(
        &mut builder,
        "p/Source/src7",
        EntityType::Source,
        json!({ "path": "src/number-commit.rs", "commit": 9 }),
    );
    builder.build().expect("graph is valid")
}

fn document_path(dir: &TempDir) -> PathBuf {
    let path = dir.path().join(UNDERSTANDING);
    fs::create_dir_all(path.parent().expect("document has a parent")).expect("research directory");
    path
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).expect("projection file exists")
}

fn begin(name: &str) -> String {
    format!("<!-- kaya:begin generated:{name} -->")
}

fn end(name: &str) -> String {
    format!("<!-- kaya:end generated:{name} -->")
}

/// Body bytes between a generated block's begin and end markers.
fn block_body(doc: &str, name: &str) -> String {
    let begin = begin(name);
    let end = end(name);
    let start = doc
        .find(&begin)
        .unwrap_or_else(|| panic!("block {name} has a begin marker"))
        + begin.len();
    let stop = doc[start..]
        .find(&end)
        .unwrap_or_else(|| panic!("block {name} has an end marker"))
        + start;
    doc[start..stop].to_string()
}

/// Every non-blank line of a generated block, in document order.
fn block_lines(doc: &str, name: &str) -> Vec<String> {
    block_body(doc, name)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect()
}

fn lines(expected: &[&str]) -> Vec<String> {
    expected.iter().map(|line| (*line).to_owned()).collect()
}

const AUTHORED_DOC: &str = "\
# Understanding notes

Intro prose that must survive.

<!-- kaya:authored -->
The hit rate rose after the cache shipped.
<!-- kaya:cites p/Finding/f1 -->
<!-- kaya:end authored -->

<!-- wai:begin -->
foreign managed block bytes
<!-- wai:end -->

Tail prose.
";

#[test]
fn findings_and_analysis_renders_admissible_records_only() {
    let dir = TempDir::new().expect("temp dir");
    let path = dir.path().join("understanding.md");

    let built = build_understanding(&graph_with_observed_records_only(), &path)
        .expect("understanding builds");

    let after = read(&path);
    assert_eq!(
        block_lines(&after, FINDINGS_BLOCK),
        lines(&[
            "### Findings",
            "- p/Finding/f1 — First finding",
            "- p/Finding/f2 — Second finding",
            "### Observations",
            "- p/Observation/o1 — Latency fell",
            "### Claims",
            "- p/Claim/c1 — Cache is cold",
            "- p/Claim/c2 — c2",
        ]),
        "the block renders admissible records in id order: {after}"
    );

    let body = block_body(&after, FINDINGS_BLOCK);
    for forbidden in [
        "p/Run/r1",
        "run one",
        "### Runs",
        "p/Artifact/a1",
        "target/report.json",
        "### Artifacts",
    ] {
        assert!(
            !body.contains(forbidden),
            "a run or artifact record never enters the findings block ({forbidden}): {body}"
        );
    }

    assert_eq!(
        built.summary.recomputed,
        vec![FINDINGS_BLOCK.to_string()],
        "only the findings block renders: {:?}",
        built.summary.recomputed
    );
    assert_eq!(
        built.summary.sources.get(FINDINGS_BLOCK),
        Some(&lines(&[
            "p/Claim/c1",
            "p/Claim/c2",
            "p/Finding/f1",
            "p/Finding/f2",
            "p/Observation/o1",
        ])),
        "the source map traces the rendered records only: {:?}",
        built.summary.sources
    );
    assert_eq!(
        built.omitted,
        vec![Omitted {
            family: "understanding",
            kind: "decision-record",
            reason: REASON_NO_RECORDS,
        }],
        "the empty decision record is reported, never rendered"
    );
}

#[test]
fn block_constants_are_the_kind_block_names() {
    assert_eq!(
        FINDINGS_BLOCK,
        Kind::FindingsAndAnalysis.block_name(),
        "the findings build constant is the kind's generated block name"
    );
    assert_eq!(
        DECISION_BLOCK,
        Kind::DecisionRecord.block_name(),
        "the decision build constant is the kind's generated block name"
    );
}

#[test]
fn incomplete_sources_never_render_or_enter_the_source_map() {
    let dir = TempDir::new().expect("temp dir");
    let path = document_path(&dir);

    let built =
        build_understanding(&graph_with_incomplete_sources(), &path).expect("understanding builds");

    let after = read(&path);
    assert_eq!(
        block_lines(&after, DECISION_BLOCK),
        lines(&[
            "### Decisions",
            "- p/Decision/d1 — Ship the cache",
            "### Evidence",
            "- p/Source/src1 — src/cache.rs@abc123",
            stale_marker("p/Decision/d1").as_str(),
        ]),
        "only a Source with a path and a commit renders one evidence row: {after}"
    );
    let decision_body = block_body(&after, DECISION_BLOCK);
    for incomplete in ["src2", "src3", "src4", "src5", "src6", "src7"] {
        assert!(
            !decision_body.contains(incomplete),
            "an incomplete source is skipped entirely ({incomplete}): {decision_body}"
        );
    }
    assert!(
        !decision_body.contains("— @") && !decision_body.contains("@\n"),
        "no row renders an empty reference or a bare path: {decision_body}"
    );

    assert_eq!(
        built.summary.sources.get(DECISION_BLOCK),
        Some(&lines(&["p/Decision/d1", "p/Source/src1"])),
        "the source map traces the decision and its one citable Source: {:?}",
        built.summary.sources
    );
    assert!(
        !built.summary.sources.contains_key(FINDINGS_BLOCK),
        "an incomplete Source alone never conjures the findings block: {:?}",
        built.summary.sources
    );
}

#[test]
fn decision_record_renders_decisions_and_evidence_references() {
    let dir = TempDir::new().expect("temp dir");
    let path = document_path(&dir);

    let built = build_understanding(&graph_with_decisions_and_evidence(), &path)
        .expect("understanding builds");

    let after = read(&path);
    assert_eq!(
        block_lines(&after, DECISION_BLOCK),
        lines(&[
            "### Decisions",
            "- p/Decision/d1 — Ship the cache",
            "- p/Decision/d2 — Defer the migration",
            "### Evidence",
            "- p/Finding/f1 — see p/Decision/d1",
            "- p/Finding/f1 — see p/Decision/d2",
            "- p/Finding/f2 — see p/Decision/d1",
            "- p/Source/src1 — src/cache.rs@abc123",
            "- p/Source/src2 — docs/notes.md@def456",
            stale_marker("p/Decision/d1").as_str(),
            stale_marker("p/Decision/d2").as_str(),
        ]),
        "decisions sort by id and evidence by id then reference: {after}"
    );

    let decision_body = block_body(&after, DECISION_BLOCK);
    for forbidden in [
        "### Findings",
        "### Observations",
        "### Claims",
        "Cache hit rate rose",
    ] {
        assert!(
            !decision_body.contains(forbidden),
            "the decision record renders no findings rows ({forbidden}): {decision_body}"
        );
    }

    assert_eq!(
        block_lines(&after, FINDINGS_BLOCK),
        lines(&[
            "### Findings",
            "- p/Finding/f1 — Cache hit rate rose",
            "- p/Finding/f2 — f2",
        ]),
        "the findings block renders its own records only: {after}"
    );
    let findings_body = block_body(&after, FINDINGS_BLOCK);
    for forbidden in [
        "p/Decision/d1",
        "src/cache.rs",
        "docs/notes.md",
        "### Evidence",
    ] {
        assert!(
            !findings_body.contains(forbidden),
            "decision rows and source refs stay out ({forbidden}): {findings_body}"
        );
    }

    let mut recomputed = built.summary.recomputed.clone();
    recomputed.sort();
    assert_eq!(
        recomputed,
        vec![DECISION_BLOCK.to_string(), FINDINGS_BLOCK.to_string()],
        "both kinds have records and both blocks render"
    );
    assert!(
        built.omitted.is_empty(),
        "no kind is omitted when every kind has records: {:?}",
        built.omitted
    );
    assert_eq!(
        built.summary.sources.get(DECISION_BLOCK),
        Some(&lines(&[
            "p/Decision/d1",
            "p/Decision/d2",
            "p/Finding/f1",
            "p/Finding/f2",
            "p/Source/src1",
            "p/Source/src2",
        ])),
        "the decision record's source map traces decisions and evidence: {:?}",
        built.summary.sources
    );
}

#[test]
fn decision_evidence_lists_only_sources_the_decision_cites() {
    let dir = TempDir::new().expect("temp dir");
    let path = document_path(&dir);

    let built = build_understanding(&graph_with_scoped_decision_evidence(), &path)
        .expect("understanding builds");

    let after = read(&path);
    assert_eq!(
        block_lines(&after, DECISION_BLOCK),
        lines(&[
            "### Decisions",
            "- p/Decision/d1 — Ship the cache",
            "- p/Decision/d2 — Defer the migration",
            "### Evidence",
            "- p/Finding/f1 — see p/Decision/d1",
            "- p/Source/src1 — src/cache.rs@abc123 — see p/Decision/d1",
            "- p/Source/src2 — docs/notes.md@def456 — see p/Decision/d2",
            "- p/Source/src3 — shared/notes.md@789aaa — see p/Decision/d1",
            "- p/Source/src3 — shared/notes.md@789aaa — see p/Decision/d2",
        ]),
        "each decision's evidence lists only its own cited sources: {after}"
    );

    let decision_body = block_body(&after, DECISION_BLOCK);
    for forbidden in [
        "src4",
        "src5",
        "reports/x.md",
        "src/only-path.rs",
        "attribution-stale",
    ] {
        assert!(
            !decision_body.contains(forbidden),
            "an uncited source renders no row and no staleness marker appears once cites exist ({forbidden}): {decision_body}"
        );
    }

    assert_eq!(
        built.summary.sources.get(DECISION_BLOCK),
        Some(&lines(&[
            "p/Decision/d1",
            "p/Decision/d2",
            "p/Finding/f1",
            "p/Source/src1",
            "p/Source/src2",
            "p/Source/src3",
        ])),
        "the source map traces decisions and cited sources only: {:?}",
        built.summary.sources
    );
}

#[test]
fn mixed_ledger_scopes_the_cited_decision_and_marks_the_uncited_one() {
    let dir = TempDir::new().expect("temp dir");
    let path = document_path(&dir);

    let built = build_understanding(&graph_with_mixed_decision_evidence(), &path)
        .expect("understanding builds");

    let after = read(&path);
    assert_eq!(
        block_lines(&after, DECISION_BLOCK),
        lines(&[
            "### Decisions",
            "- p/Decision/d1 — Ship the cache",
            "- p/Decision/d2 — Defer the migration",
            "### Evidence",
            "- p/Source/src1 — src/cache.rs@abc123",
            "- p/Source/src1 — src/cache.rs@abc123 — see p/Decision/d1",
            "- p/Source/src2 — docs/notes.md@def456",
            stale_marker("p/Decision/d2").as_str(),
        ]),
        "the cited decision scopes its evidence and the uncited one falls back with a marker: {after}"
    );

    assert_eq!(
        built.summary.sources.get(DECISION_BLOCK),
        Some(&lines(&[
            "p/Decision/d1",
            "p/Decision/d2",
            "p/Source/src1",
            "p/Source/src2",
        ])),
        "the source map traces both decisions and every rendered source: {:?}",
        built.summary.sources
    );
}

#[test]
fn decisions_only_ledger_renders_no_evidence_section() {
    let dir = TempDir::new().expect("temp dir");
    let path = document_path(&dir);

    let built =
        build_understanding(&graph_with_decisions_only(), &path).expect("understanding builds");

    let after = read(&path);
    assert_eq!(
        block_lines(&after, DECISION_BLOCK),
        lines(&["### Decisions", "- p/Decision/d1 — Ship the cache"]),
        "a ledger with no Source renders no evidence section: {after}"
    );
    assert!(
        !block_body(&after, DECISION_BLOCK).contains("attribution-stale"),
        "no evidence means no staleness marker: {after}"
    );
    assert_eq!(
        built.summary.sources.get(DECISION_BLOCK),
        Some(&lines(&["p/Decision/d1"])),
        "only the decision enters the source map: {:?}",
        built.summary.sources
    );
}

#[test]
fn duplicate_citation_renders_one_row_and_incomplete_citation_renders_none() {
    let dir = TempDir::new().expect("temp dir");
    let path = document_path(&dir);

    let built = build_understanding(&graph_with_duplicate_and_incomplete_citations(), &path)
        .expect("understanding builds");

    let after = read(&path);
    assert_eq!(
        block_lines(&after, DECISION_BLOCK),
        lines(&[
            "### Decisions",
            "- p/Decision/d1 — Ship the cache",
            "### Evidence",
            "- p/Source/src1 — src/cache.rs@abc123 — see p/Decision/d1",
        ]),
        "a repeated citation renders one row and an incomplete citation none: {after}"
    );
    let decision_body = block_body(&after, DECISION_BLOCK);
    assert!(
        !decision_body.contains("only-path.rs") && !decision_body.contains("attribution-stale"),
        "a citation cannot resurrect an incomplete Source and cites suppress the marker: {decision_body}"
    );
    assert_eq!(
        built.summary.sources.get(DECISION_BLOCK),
        Some(&lines(&["p/Decision/d1", "p/Source/src1"])),
        "the incomplete Source never enters the source map: {:?}",
        built.summary.sources
    );
}

#[test]
fn source_cited_by_two_decisions_renders_one_row_per_reference() {
    let dir = TempDir::new().expect("temp dir");
    let path = document_path(&dir);

    let built = build_understanding(&graph_with_scoped_decision_evidence(), &path)
        .expect("understanding builds");

    let after = read(&path);
    assert_eq!(
        block_lines(&after, DECISION_BLOCK),
        lines(&[
            "### Decisions",
            "- p/Decision/d1 — Ship the cache",
            "- p/Decision/d2 — Defer the migration",
            "### Evidence",
            "- p/Finding/f1 — see p/Decision/d1",
            "- p/Source/src1 — src/cache.rs@abc123 — see p/Decision/d1",
            "- p/Source/src2 — docs/notes.md@def456 — see p/Decision/d2",
            "- p/Source/src3 — shared/notes.md@789aaa — see p/Decision/d1",
            "- p/Source/src3 — shared/notes.md@789aaa — see p/Decision/d2",
        ]),
        "one row per citing decision for a shared source: {after}"
    );
    assert_eq!(
        built.summary.sources.get(DECISION_BLOCK),
        Some(&lines(&[
            "p/Decision/d1",
            "p/Decision/d2",
            "p/Finding/f1",
            "p/Source/src1",
            "p/Source/src2",
            "p/Source/src3",
        ])),
        "the source map traces decisions and cited sources only: {:?}",
        built.summary.sources
    );
}

#[test]
fn pre_relationship_ledger_falls_back_to_global_attribution_with_staleness_marker() {
    let dir = TempDir::new().expect("temp dir");
    let path = document_path(&dir);

    let built = build_understanding(&graph_with_decisions_and_evidence(), &path)
        .expect("understanding builds");

    let after = read(&path);
    assert_eq!(
        block_lines(&after, DECISION_BLOCK),
        lines(&[
            "### Decisions",
            "- p/Decision/d1 — Ship the cache",
            "- p/Decision/d2 — Defer the migration",
            "### Evidence",
            "- p/Finding/f1 — see p/Decision/d1",
            "- p/Finding/f1 — see p/Decision/d2",
            "- p/Finding/f2 — see p/Decision/d1",
            "- p/Source/src1 — src/cache.rs@abc123",
            "- p/Source/src2 — docs/notes.md@def456",
            stale_marker("p/Decision/d1").as_str(),
            stale_marker("p/Decision/d2").as_str(),
        ]),
        "a pre-relationship ledger keeps global attribution and carries the marker: {after}"
    );

    let decision_body = block_body(&after, DECISION_BLOCK);
    for row in [
        "- p/Source/src1 — src/cache.rs@abc123\n",
        "- p/Source/src2 — docs/notes.md@def456\n",
    ] {
        assert!(
            decision_body.contains(row),
            "fallback rows keep the pre-relationship format with no `see` suffix: {decision_body}"
        );
    }

    assert_eq!(
        built.summary.sources.get(DECISION_BLOCK),
        Some(&lines(&[
            "p/Decision/d1",
            "p/Decision/d2",
            "p/Finding/f1",
            "p/Finding/f2",
            "p/Source/src1",
            "p/Source/src2",
        ])),
        "the fallback source map stays ledger-global: {:?}",
        built.summary.sources
    );
}

#[test]
fn empty_kinds_are_omitted_and_reported() {
    let dir = TempDir::new().expect("temp dir");
    let path = document_path(&dir);

    let built =
        build_understanding(&graph_with_findings_only(), &path).expect("understanding builds");

    let expected = vec![Omitted {
        family: "understanding",
        kind: "decision-record",
        reason: REASON_NO_RECORDS,
    }];
    assert_eq!(
        built.omitted, expected,
        "the empty decision record is reported once"
    );
    assert_eq!(
        built.summary.recomputed,
        vec![FINDINGS_BLOCK.to_string()],
        "an empty kind has no summary key: {:?}",
        built.summary.recomputed
    );

    let after = read(&path);
    assert!(
        after.contains(&begin(FINDINGS_BLOCK)) && after.contains(&end(FINDINGS_BLOCK)),
        "the non-empty kind keeps its block: {after}"
    );
    assert!(
        !after.contains("generated:understanding.decision-record"),
        "an empty kind leaves no block, markers, or placeholder: {after}"
    );
    assert_eq!(
        block_lines(&after, FINDINGS_BLOCK),
        lines(&["### Findings", "- p/Finding/f1 — Only finding"]),
        "the findings block renders alone: {after}"
    );

    // A decision block left behind by an earlier build must be removed.
    let seeded = format!(
        "{after}\n{}\n### Decisions\n\n- p/Decision/d9 — Stale decision\n{}\n",
        begin(DECISION_BLOCK),
        end(DECISION_BLOCK)
    );
    fs::write(&path, &seeded).expect("seed stale decision block");

    let rebuilt = build_understanding(&graph_with_findings_only(), &path)
        .expect("rebuild removes the empty kind");
    assert_eq!(rebuilt.omitted, expected, "the omission is still reported");
    assert_eq!(
        rebuilt.summary.recomputed,
        vec![DECISION_BLOCK.to_string()],
        "the removed block is the only recomputed one: {:?}",
        rebuilt.summary.recomputed
    );
    assert_eq!(
        rebuilt.summary.unchanged,
        vec![FINDINGS_BLOCK.to_string()],
        "the kept block already matched its bytes: {:?}",
        rebuilt.summary.unchanged
    );

    let cleaned = read(&path);
    assert!(
        !cleaned.contains("generated:understanding.decision-record"),
        "the stale decision block is gone: {cleaned}"
    );
    assert!(
        !cleaned.contains("p/Decision/d9"),
        "the stale decision rows are gone: {cleaned}"
    );
    assert_eq!(
        block_lines(&cleaned, FINDINGS_BLOCK),
        lines(&["### Findings", "- p/Finding/f1 — Only finding"]),
        "the findings block survives the removal: {cleaned}"
    );

    // Once the removal has been applied, the document is stable: a third
    // build rewrites nothing and reports the surviving block unchanged.
    let third =
        build_understanding(&graph_with_findings_only(), &path).expect("the third build succeeds");
    assert!(
        third.summary.recomputed.is_empty(),
        "the removal happens once: {:?}",
        third.summary.recomputed
    );
    assert_eq!(
        third.summary.unchanged,
        vec![FINDINGS_BLOCK.to_string()],
        "the surviving block is reported unchanged: {:?}",
        third.summary.unchanged
    );
    assert_eq!(third.omitted, expected, "the omission is still reported");
    assert_eq!(read(&path), cleaned, "the third build writes no bytes");
}

#[test]
fn removing_both_kinds_deletes_the_document() {
    let dir = TempDir::new().expect("temp dir");
    let path = document_path(&dir);

    build_understanding(&graph_with_both_kinds(), &path)
        .expect("seed the document with both kinds");
    let seeded = read(&path);
    assert!(
        seeded.contains(&begin(FINDINGS_BLOCK)) && seeded.contains(&begin(DECISION_BLOCK)),
        "the seed carries both generated blocks: {seeded}"
    );

    let empty = GraphBuilder::new()
        .build()
        .expect("an empty graph is valid");
    let removed = build_understanding(&empty, &path).expect("removing both kinds succeeds");

    assert!(
        !path.exists(),
        "a document left with only whitespace is deleted, never written as zero bytes"
    );
    let mut recomputed = removed.summary.recomputed.clone();
    recomputed.sort();
    assert_eq!(
        recomputed,
        vec![DECISION_BLOCK.to_string(), FINDINGS_BLOCK.to_string()],
        "both removed blocks are reported recomputed: {recomputed:?}"
    );
    assert!(
        removed.summary.unchanged.is_empty(),
        "removals are not unchanged blocks: {:?}",
        removed.summary.unchanged
    );
    let mut omitted = removed.omitted.clone();
    omitted.sort_by_key(|entry| (entry.family, entry.kind));
    assert_eq!(
        omitted,
        vec![
            Omitted {
                family: "understanding",
                kind: "decision-record",
                reason: REASON_NO_RECORDS,
            },
            Omitted {
                family: "understanding",
                kind: "findings-and-analysis",
                reason: REASON_NO_RECORDS,
            },
        ],
        "both empty kinds are reported omitted: {:?}",
        removed.omitted
    );

    // The deleted document is not a failure on the next build: absence reads
    // as an empty document and an empty graph writes nothing.
    let again =
        build_understanding(&empty, &path).expect("a deleted document is treated as absent");
    assert!(
        !path.exists(),
        "an empty build never recreates the document"
    );
    assert!(
        again.summary.recomputed.is_empty(),
        "nothing is recomputed in an absent document: {:?}",
        again.summary.recomputed
    );
    assert_eq!(
        again.omitted, removed.omitted,
        "the omission is reported on every build"
    );
}

#[test]
fn block_order_is_canonical_and_rebuild_is_byte_identical() {
    let dir = TempDir::new().expect("temp dir");
    let path = document_path(&dir);
    // A decision-only document: the kind exists alone, out of canonical order.
    let seeded = format!(
        "{}\n### Decisions\n\n- p/Decision/d9 — Oldest decision\n{}\n",
        begin(DECISION_BLOCK),
        end(DECISION_BLOCK)
    );
    fs::write(&path, &seeded).expect("seed a decision-only document");

    let built = build_understanding(&graph_with_both_kinds(), &path).expect("understanding builds");
    let ordered = read(&path);

    let findings_at = ordered
        .find(&begin(FINDINGS_BLOCK))
        .expect("the findings block renders");
    let decision_at = ordered
        .find(&begin(DECISION_BLOCK))
        .expect("the decision block renders");
    assert!(
        findings_at < decision_at,
        "the findings block precedes the decision block: {ordered}"
    );
    assert!(
        !ordered.contains("p/Decision/d9"),
        "the superseded decision body is replaced: {ordered}"
    );
    assert_eq!(
        block_lines(&ordered, FINDINGS_BLOCK),
        lines(&[
            "### Findings",
            "- p/Finding/f1 — Cache hit rate rose",
            "### Observations",
            "- p/Observation/o1 — Latency fell",
            "### Claims",
            "- p/Claim/c1 — Cold caches read slower",
        ]),
        "the reordered findings block carries its own records: {ordered}"
    );

    let mut recomputed = built.summary.recomputed.clone();
    recomputed.sort();
    assert_eq!(
        recomputed,
        vec![DECISION_BLOCK.to_string(), FINDINGS_BLOCK.to_string()],
        "both blocks are placed in canonical order: {recomputed:?}"
    );

    // The same graph over a fresh path compiles to the same bytes: canonical
    // order is a property of the graph, not of the document it replaced.
    let fresh_dir = TempDir::new().expect("temp dir");
    let fresh_path = document_path(&fresh_dir);
    build_understanding(&graph_with_both_kinds(), &fresh_path).expect("a fresh build succeeds");
    assert_eq!(
        read(&fresh_path),
        ordered,
        "a fresh build of the same graph is byte-identical"
    );

    let again = build_understanding(&graph_with_both_kinds(), &path).expect("a rebuild succeeds");
    let mut unchanged = again.summary.unchanged.clone();
    unchanged.sort();
    assert_eq!(
        unchanged,
        vec![DECISION_BLOCK.to_string(), FINDINGS_BLOCK.to_string()],
        "the canonically ordered document is already current: {unchanged:?}"
    );
    assert!(
        again.summary.recomputed.is_empty(),
        "a rebuild writes nothing: {:?}",
        again.summary.recomputed
    );
}

#[test]
fn authored_bytes_survive_and_second_build_is_unchanged() {
    let dir = TempDir::new().expect("temp dir");
    let path = document_path(&dir);
    fs::write(&path, AUTHORED_DOC).expect("seed authored document");

    let built = build_understanding(&graph_with_both_kinds(), &path).expect("understanding builds");
    let mut recomputed = built.summary.recomputed.clone();
    recomputed.sort();
    assert_eq!(
        recomputed,
        vec![DECISION_BLOCK.to_string(), FINDINGS_BLOCK.to_string()],
        "the first build writes both blocks: {recomputed:?}"
    );

    let after = read(&path);
    assert!(
        after.starts_with("# Understanding notes\n\nIntro prose that must survive.\n"),
        "intro prose survives verbatim: {after}"
    );
    assert!(
        after.contains(
            "<!-- kaya:authored -->\nThe hit rate rose after the cache shipped.\n\
             <!-- kaya:cites p/Finding/f1 -->\n<!-- kaya:end authored -->"
        ),
        "authored prose and its citation survive verbatim: {after}"
    );
    assert!(
        after.contains("<!-- wai:begin -->\nforeign managed block bytes\n<!-- wai:end -->"),
        "foreign managed bytes survive verbatim: {after}"
    );
    let tail = after
        .find("Tail prose.\n")
        .expect("trailing prose survives");
    let appended = after
        .find(&begin(FINDINGS_BLOCK))
        .min(after.find(&begin(DECISION_BLOCK)))
        .expect("both blocks are appended");
    assert!(
        tail < appended,
        "trailing prose stays ahead of the appended blocks: {after}"
    );

    let again =
        build_understanding(&graph_with_both_kinds(), &path).expect("repeat build succeeds");
    let mut unchanged = again.summary.unchanged.clone();
    unchanged.sort();
    assert_eq!(
        unchanged,
        vec![DECISION_BLOCK.to_string(), FINDINGS_BLOCK.to_string()],
        "a byte-identical rebuild reports both blocks unchanged: {unchanged:?}"
    );
    assert!(
        again.summary.recomputed.is_empty(),
        "nothing is rewritten when both blocks match: {:?}",
        again.summary.recomputed
    );
    assert_eq!(
        again.summary.sources, built.summary.sources,
        "recomputing adds no source id and drops none"
    );
    assert_eq!(read(&path), after, "unchanged bodies mean no rewrite");
}

#[test]
fn legacy_block_is_refused_before_writing() {
    let dir = TempDir::new().expect("temp dir");
    let path = document_path(&dir);
    let legacy_doc = "\
# Understanding notes

Authored prose stays put.

<!-- kaya:authored -->
My earlier reading of f1.
<!-- kaya:end authored -->

<!-- kaya:begin generated:explanation -->
### Explanations

- p/Finding/f1 — because
<!-- kaya:end generated:explanation -->
";
    fs::write(&path, legacy_doc).expect("seed legacy document");

    let error = build_understanding(&graph_with_both_kinds(), &path)
        .expect_err("a block outside the closed vocabulary is refused");
    assert_eq!(
        error.label, LABEL_LEGACY_BLOCK,
        "the refusal carries the vocabulary label: {error}"
    );
    assert!(
        error.message.contains("explanation"),
        "the refusal names the offending block: {}",
        error.message
    );
    assert_eq!(read(&path), legacy_doc, "a refused build writes no bytes");
}
