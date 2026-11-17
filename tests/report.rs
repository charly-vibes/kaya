//! Contract tests for the experiment report projection.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use tempfile::TempDir;

use kaya::model::{EntityType, Graph, GraphBuilder, Relationship};
use kaya::projection::report::{
    Built, LABEL_LEGACY_BLOCK, Omitted, REASON_NO_RECORDS, RECORD_BLOCK, SPECIFICATION_BLOCK,
    build, stale_authored,
};
use kaya::promotion::{Assertion, PromotionState, Status};

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

fn graph() -> Graph {
    let mut builder = GraphBuilder::new();
    node(&mut builder, "p/Experiment/exp1", EntityType::Experiment);
    node(&mut builder, "p/Question/q1", EntityType::Question);
    node(&mut builder, "p/Hypothesis/h1", EntityType::Hypothesis);
    node(&mut builder, "p/Run/r1", EntityType::Run);
    node(&mut builder, "p/Run/r2", EntityType::Run);
    node(&mut builder, "p/Observation/o1", EntityType::Observation);
    node(&mut builder, "p/Observation/o2", EntityType::Observation);
    node(&mut builder, "p/Finding/f1", EntityType::Finding);
    node(&mut builder, "p/Finding/f2", EntityType::Finding);

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
        "p/Run/r2",
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
        "p/Observation/o2",
        "p/Run/r2",
    );
    edge(
        &mut builder,
        Relationship::Observes,
        "p/Observation/o1",
        "p/Run/r1",
    );
    edge(
        &mut builder,
        Relationship::Derives,
        "p/Finding/f2",
        "p/Observation/o2",
    );
    edge(
        &mut builder,
        Relationship::Derives,
        "p/Finding/f1",
        "p/Observation/o1",
    );
    builder.build().expect("graph is acyclic and complete")
}

fn graph_with_extra_run() -> Graph {
    let mut builder = GraphBuilder::new();
    node(&mut builder, "p/Experiment/exp1", EntityType::Experiment);
    node(&mut builder, "p/Run/r1", EntityType::Run);
    node(&mut builder, "p/Run/r2", EntityType::Run);
    node(&mut builder, "p/Run/r3", EntityType::Run);
    edge(
        &mut builder,
        Relationship::Executes,
        "p/Run/r1",
        "p/Experiment/exp1",
    );
    edge(
        &mut builder,
        Relationship::Executes,
        "p/Run/r2",
        "p/Experiment/exp1",
    );
    edge(
        &mut builder,
        Relationship::Executes,
        "p/Run/r3",
        "p/Experiment/exp1",
    );
    builder.build().expect("graph is valid")
}

fn assertion(id: &str, text: &str, status: Status, machine: bool) -> Assertion {
    Assertion {
        id: id.to_string(),
        text: text.to_string(),
        status,
        machine_proposed: machine,
        event_ids: vec![format!("{id}-ev")],
        author: if machine {
            "analysis-agent".to_string()
        } else {
            "alice".to_string()
        },
    }
}

fn promotion() -> PromotionState {
    PromotionState {
        assertions: vec![
            assertion(
                "assertion-1",
                "counts contradicted H-001",
                Status::Challenged,
                false,
            ),
            assertion("assertion-2", "the effect is real", Status::Proposed, true),
            assertion("assertion-3", "H-002 survives", Status::Established, false),
        ],
    }
}

fn report_path(dir: &TempDir) -> std::path::PathBuf {
    dir.path().join("report.md")
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).expect("report file exists")
}

const EXPERIMENT: &str = "p/Experiment/exp1";

/// Summary name for one block of the fixture experiment.
fn name(block: &str) -> String {
    format!("{block}:{EXPERIMENT}")
}

/// Ascending order, so assertions never pin emission order.
fn sorted(mut names: Vec<String>) -> Vec<String> {
    names.sort();
    names
}

/// The body of one generated block, markers excluded.
fn block(document: &str, block_name: &str) -> String {
    let begin = format!("<!-- kaya:begin generated:{block_name} -->");
    let end = format!("<!-- kaya:end generated:{block_name} -->");
    let start = document
        .find(begin.as_str())
        .unwrap_or_else(|| panic!("document lacks the {block_name} begin marker"));
    let body_start = start + begin.len();
    let length = document[body_start..]
        .find(end.as_str())
        .unwrap_or_else(|| panic!("document lacks the {block_name} end marker"));
    document[body_start..body_start + length].to_string()
}

/// A build that rendered both kinds, so nothing was omitted.
fn both_blocks(built: &Built) -> &kaya::projection::Summary {
    assert!(
        built.omitted.is_empty(),
        "both kinds rendered, so nothing is omitted: {:?}",
        built.omitted
    );
    &built.summary
}

/// One experiment that declares intent and has no observed records.
fn intent_only_graph() -> Graph {
    let mut builder = GraphBuilder::new();
    node(&mut builder, EXPERIMENT, EntityType::Experiment);
    node(&mut builder, "p/Question/q1", EntityType::Question);
    node(&mut builder, "p/Hypothesis/h1", EntityType::Hypothesis);
    edge(
        &mut builder,
        Relationship::Investigates,
        "p/Question/q1",
        EXPERIMENT,
    );
    edge(
        &mut builder,
        Relationship::Predicts,
        "p/Hypothesis/h1",
        EXPERIMENT,
    );
    builder.build().expect("graph is acyclic and complete")
}

/// One experiment with a single executed run and no declared intent.
fn record_only_graph() -> Graph {
    let mut builder = GraphBuilder::new();
    node(&mut builder, EXPERIMENT, EntityType::Experiment);
    node(&mut builder, "p/Run/r1", EntityType::Run);
    edge(&mut builder, Relationship::Executes, "p/Run/r1", EXPERIMENT);
    builder.build().expect("graph is acyclic and complete")
}

/// The record-only experiment plus a predicted hypothesis.
fn record_and_hypothesis_graph() -> Graph {
    let mut builder = GraphBuilder::new();
    node(&mut builder, EXPERIMENT, EntityType::Experiment);
    node(&mut builder, "p/Run/r1", EntityType::Run);
    node(&mut builder, "p/Hypothesis/h1", EntityType::Hypothesis);
    edge(&mut builder, Relationship::Executes, "p/Run/r1", EXPERIMENT);
    edge(
        &mut builder,
        Relationship::Predicts,
        "p/Hypothesis/h1",
        EXPERIMENT,
    );
    builder.build().expect("graph is acyclic and complete")
}

const AUTHORED_DOC: &str = "\
# Research notes

Intro prose that must survive.

<!-- kaya:authored -->
The observed counts contradict my earlier reading of H-001.
<!-- kaya:cites ev-old -->
<!-- kaya:end authored -->

<!-- wai:begin -->
foreign managed block bytes
<!-- wai:end -->

Tail prose.
";

#[test]
fn authored_bytes_and_foreign_blocks_survive_rebuild() {
    let dir = TempDir::new().expect("temp dir");
    let path = report_path(&dir);
    fs::write(&path, AUTHORED_DOC).expect("seed report");

    let built = build(&graph(), EXPERIMENT, &path, &promotion()).expect("report builds");
    assert_eq!(
        sorted(built.summary.recomputed.clone()),
        sorted(vec![name(SPECIFICATION_BLOCK), name(RECORD_BLOCK)])
    );
    assert!(built.omitted.is_empty(), "both kinds rendered");

    let after = read(&path);
    assert!(
        after.contains(AUTHORED_DOC),
        "authored and foreign bytes survive"
    );

    let specification = block(&after, SPECIFICATION_BLOCK);
    let record = block(&after, RECORD_BLOCK);
    assert!(specification.contains("## Question"));
    assert!(specification.contains("- p/Question/q1"));
    assert!(specification.contains("- p/Hypothesis/h1"));
    assert!(record.contains("## Runs"));
    assert!(
        record.find("- p/Run/r1").expect("r1 rendered")
            < record.find("- p/Run/r2").expect("r2 rendered"),
        "runs are sorted"
    );
    assert!(
        after
            .find(SPECIFICATION_BLOCK)
            .expect("specification block")
            < after.find(RECORD_BLOCK).expect("record block"),
        "blocks are written in kind order"
    );
}

#[test]
fn generated_sections_are_deterministic() {
    let left = TempDir::new().expect("temp dir");
    let right = TempDir::new().expect("temp dir");
    let left_path = report_path(&left);
    let right_path = report_path(&right);

    let left_built = build(&graph(), EXPERIMENT, &left_path, &promotion()).expect("first build");
    let left_summary = both_blocks(&left_built);
    let right_built = build(&graph(), EXPERIMENT, &right_path, &promotion()).expect("second build");
    let right_summary = both_blocks(&right_built);

    assert_eq!(read(&left_path), read(&right_path));
    assert_eq!(left_summary.sources, right_summary.sources);
    assert_eq!(left_summary.recomputed, right_summary.recomputed);
}

#[test]
fn second_build_with_unchanged_inputs_keeps_bytes() {
    let dir = TempDir::new().expect("temp dir");
    let path = report_path(&dir);

    build(&graph(), EXPERIMENT, &path, &promotion()).expect("first build");
    let before = read(&path);

    let second = build(&graph(), EXPERIMENT, &path, &promotion()).expect("second build");
    let summary = both_blocks(&second);
    assert_eq!(
        sorted(summary.unchanged.clone()),
        sorted(vec![name(SPECIFICATION_BLOCK), name(RECORD_BLOCK)])
    );
    assert!(summary.recomputed.is_empty());
    assert_eq!(read(&path), before, "unchanged body means no rewrite");
}

#[test]
fn changed_graph_recomputes_the_report() {
    let dir = TempDir::new().expect("temp dir");
    let path = report_path(&dir);

    build(&graph(), EXPERIMENT, &path, &promotion()).expect("first build");
    assert!(!read(&path).contains("p/Run/r3"));

    let rebuilt = build(&graph_with_extra_run(), EXPERIMENT, &path, &promotion())
        .expect("rebuild after a run lands");
    // The new graph declares no intent, so the specification block becomes
    // empty and is removed; removing an obsolete block counts as recomputed.
    assert_eq!(
        sorted(rebuilt.summary.recomputed.clone()),
        sorted(vec![name(SPECIFICATION_BLOCK), name(RECORD_BLOCK)]),
        "the changed record block and the removed specification block are recomputed"
    );
    assert!(
        rebuilt.summary.unchanged.is_empty(),
        "nothing survives unchanged: {:?}",
        rebuilt.summary.unchanged
    );

    let after = read(&path);
    assert!(after.contains("p/Run/r3"));
    assert!(
        !after.contains(SPECIFICATION_BLOCK),
        "an obsolete block is removed with its markers: {after}"
    );
    assert!(
        !after.contains("p/Question/q1"),
        "removed intent rows leave no residue: {after}"
    );
}

#[test]
fn source_map_names_every_rendered_fact() {
    let dir = TempDir::new().expect("temp dir");
    let path = report_path(&dir);

    let built = build(&graph(), EXPERIMENT, &path, &promotion()).expect("report builds");
    let sources = &built.summary.sources;

    let specification = sources
        .get(&name(SPECIFICATION_BLOCK))
        .expect("the specification block has a source map");
    assert_eq!(
        sorted(specification.to_vec()),
        vec!["p/Hypothesis/h1".to_string(), "p/Question/q1".to_string()],
        "the specification block names exactly the intent rows it rendered"
    );

    let record = sources
        .get(&name(RECORD_BLOCK))
        .expect("the record block has a source map");
    for id in [
        "p/Run/r1",
        "p/Run/r2",
        "p/Observation/o1",
        "p/Observation/o2",
        "p/Finding/f1",
        "p/Finding/f2",
        "assertion-1",
        "assertion-1-ev",
        "assertion-2",
        "assertion-2-ev",
        "assertion-3",
        "assertion-3-ev",
    ] {
        assert!(
            record.iter().any(|source| source == id),
            "record source map names {id}: {record:?}"
        );
    }
    for id in ["p/Question/q1", "p/Hypothesis/h1"] {
        assert!(
            !record.iter().any(|source| source == id),
            "the record block never sources intent {id}: {record:?}"
        );
    }
    assert_eq!(
        record,
        &sorted(record.to_vec()),
        "source ids are deterministic"
    );
}

#[test]
fn interpretations_show_status_and_machine_flag() {
    let dir = TempDir::new().expect("temp dir");
    let path = report_path(&dir);

    build(&graph(), EXPERIMENT, &path, &promotion()).expect("report builds");
    let after = read(&path);
    let record = block(&after, RECORD_BLOCK);

    assert!(record.contains("## Interpretations"));
    assert!(record.contains("`challenged` counts contradicted H-001"));
    assert!(record.contains("`proposed` the effect is real"));
    assert!(record.contains("`established` H-002 survives"));
    assert!(record.contains("machine-proposed: yes"));
    assert!(record.contains("machine-proposed: no"));
    assert!(
        !block(&after, SPECIFICATION_BLOCK).contains("## Interpretations"),
        "interpretations are observed records, never intent"
    );
}

#[test]
fn intent_rows_land_in_the_specification_block_and_records_in_the_record_block() {
    let dir = TempDir::new().expect("temp dir");
    let path = report_path(&dir);

    build(&graph(), EXPERIMENT, &path, &promotion()).expect("report builds");
    let after = read(&path);
    let specification = block(&after, SPECIFICATION_BLOCK);
    let record = block(&after, RECORD_BLOCK);

    for heading in ["## Question", "## Hypothesis"] {
        assert!(
            specification.contains(heading),
            "the specification block renders {heading}: {specification}"
        );
    }
    for row in ["- p/Question/q1", "- p/Hypothesis/h1"] {
        assert!(
            specification.contains(row),
            "the specification block renders intent {row}: {specification}"
        );
    }
    for observed in [
        "## Runs",
        "## Observations",
        "## Findings",
        "## Interpretations",
        "- p/Run/r1",
        "- p/Observation/o1",
        "- p/Finding/f1",
        "`challenged` counts contradicted H-001",
    ] {
        assert!(
            !specification.contains(observed),
            "observed content never reaches the specification block: {observed}"
        );
    }

    for heading in [
        "## Runs",
        "## Observations",
        "## Findings",
        "## Interpretations",
    ] {
        assert!(
            record.contains(heading),
            "the record block renders {heading}: {record}"
        );
    }
    for row in [
        "- p/Run/r1",
        "- p/Observation/o1",
        "- p/Finding/f1",
        "`challenged` counts contradicted H-001",
    ] {
        assert!(
            record.contains(row),
            "the record block renders observed {row}: {record}"
        );
    }
    for intent in [
        "## Question",
        "## Hypothesis",
        "- p/Question/q1",
        "- p/Hypothesis/h1",
    ] {
        assert!(
            !record.contains(intent),
            "intent never reaches the record block: {intent}"
        );
    }

    for block_name in [SPECIFICATION_BLOCK, RECORD_BLOCK] {
        let begin = format!("<!-- kaya:begin generated:{block_name} -->");
        assert_eq!(
            after.matches(begin.as_str()).count(),
            1,
            "the document holds one {block_name} block"
        );
    }
}

#[test]
fn record_kind_without_runs_is_omitted_and_reported() {
    let dir = TempDir::new().expect("temp dir");
    let path = report_path(&dir);
    let no_promotion = PromotionState {
        assertions: Vec::new(),
    };

    let built = build(&intent_only_graph(), EXPERIMENT, &path, &no_promotion)
        .expect("a specification without runs still builds");
    assert_eq!(
        built.omitted,
        vec![Omitted {
            family: "exploration",
            kind: "experiment-record",
            reason: REASON_NO_RECORDS,
        }],
        "the empty record kind is reported once, naming its family and reason"
    );
    assert_eq!(
        sorted(built.summary.recomputed.clone()),
        vec![name(SPECIFICATION_BLOCK)],
        "only the specification block is written"
    );
    assert!(
        !built.summary.sources.contains_key(&name(RECORD_BLOCK)),
        "an omitted kind has no source-map entry: {:?}",
        built.summary.sources
    );

    let after = read(&path);
    assert!(after.contains("## Question") && after.contains("## Hypothesis"));
    assert!(
        !after.contains(RECORD_BLOCK),
        "an empty kind emits no block and no markers: {after}"
    );
    for heading in [
        "## Runs",
        "## Observations",
        "## Findings",
        "## Interpretations",
    ] {
        assert!(
            !after.contains(heading),
            "an empty section is omitted, not titled: {heading}"
        );
    }
    assert!(
        !after.contains("- _(none)_"),
        "an empty kind leaves no placeholder row: {after}"
    );
}

#[test]
fn legacy_block_name_is_refused_rather_than_rewritten() {
    let dir = TempDir::new().expect("temp dir");
    let path = report_path(&dir);
    let doc = "\
# Notes

<!-- kaya:begin generated:experiment-report -->
## Runs

- p/Run/r1
<!-- kaya:end generated:experiment-report -->
";
    fs::write(&path, doc).expect("seed legacy report");

    let error = build(&graph(), EXPERIMENT, &path, &promotion())
        .expect_err("a block name outside the vocabulary is refused");
    assert_eq!(
        error.label, LABEL_LEGACY_BLOCK,
        "the refusal carries the legacy-block label"
    );
    assert!(
        error.message.contains("experiment-report"),
        "the refusal names the block: {}",
        error.message
    );
    assert_eq!(read(&path), doc, "a refused build writes nothing");
}

#[test]
fn stale_authored_names_block_and_evidence_without_touching_bytes() {
    let doc = "\
# Notes

<!-- kaya:authored -->
First block keeps its own counsel.
<!-- kaya:end authored -->

<!-- kaya:authored -->
Second block rests on evidence.
<!-- kaya:cites ev-2 ev-9 -->
<!-- kaya:end authored -->
";
    let original = doc.to_string();
    let changed = vec![
        "ev-2".to_string(),
        "ev-9".to_string(),
        "ev-elsewhere".to_string(),
    ];

    let stale = stale_authored(doc, &changed, &BTreeMap::new());
    assert_eq!(stale.len(), 1);
    assert_eq!(stale[0].block, "authored-2");
    assert_eq!(
        stale[0].evidence,
        vec!["ev-2".to_string(), "ev-9".to_string()]
    );
    assert_eq!(doc, original, "stale detection never modifies bytes");
}

#[test]
fn acknowledged_evidence_is_not_stale() {
    let doc = "\
<!-- kaya:authored -->
Evidence reviewed.
<!-- kaya:cites ev-2 ev-9 -->
<!-- kaya:end authored -->
";
    let changed = vec!["ev-2".to_string(), "ev-9".to_string()];
    let mut acknowledged = BTreeMap::new();
    acknowledged.insert(
        "authored-1".to_string(),
        vec!["ev-2".to_string(), "ev-9".to_string()],
    );
    assert!(stale_authored(doc, &changed, &acknowledged).is_empty());
}

#[test]
fn partial_acknowledgment_still_flags_the_block() {
    let doc = "\
<!-- kaya:authored -->
Evidence reviewed.
<!-- kaya:cites ev-2 ev-9 -->
<!-- kaya:end authored -->
";
    let changed = vec!["ev-2".to_string(), "ev-9".to_string()];
    let mut acknowledged = BTreeMap::new();
    acknowledged.insert("authored-1".to_string(), vec!["ev-2".to_string()]);

    let stale = stale_authored(doc, &changed, &acknowledged);
    assert_eq!(stale.len(), 1);
    assert_eq!(
        stale[0].evidence,
        vec!["ev-2".to_string(), "ev-9".to_string()]
    );
}

#[test]
fn generated_block_citations_are_not_authored() {
    let doc = "\
<!-- kaya:begin generated:experiment-report -->
## Runs

- p/Run/r1
<!-- kaya:cites ev-2 -->
<!-- kaya:end generated:experiment-report -->
";
    let stale = stale_authored(doc, &["ev-2".to_string()], &BTreeMap::new());
    assert!(stale.is_empty(), "only authored blocks are flagged");
}

#[test]
fn block_order_is_specification_then_record_regardless_of_build_history() {
    let dir = TempDir::new().expect("temp dir");
    let path = report_path(&dir);
    let no_promotion = PromotionState {
        assertions: Vec::new(),
    };

    let record_only = record_only_graph();
    let first =
        build(&record_only, EXPERIMENT, &path, &no_promotion).expect("a record-only report builds");
    assert_eq!(
        first.omitted,
        vec![Omitted {
            family: "exploration",
            kind: "experiment-specification",
            reason: REASON_NO_RECORDS,
        }],
        "the fresh report has no specification block yet"
    );
    assert!(
        !read(&path).contains(SPECIFICATION_BLOCK),
        "the record-only report writes only the record block: {}",
        read(&path)
    );

    let with_hypothesis = record_and_hypothesis_graph();
    build(&with_hypothesis, EXPERIMENT, &path, &no_promotion)
        .expect("rebuild adds the specification block");

    let after = read(&path);
    let specification_begin = format!("<!-- kaya:begin generated:{SPECIFICATION_BLOCK} -->");
    let record_begin = format!("<!-- kaya:begin generated:{RECORD_BLOCK} -->");
    let specification_at = after
        .find(specification_begin.as_str())
        .unwrap_or_else(|| panic!("the rebuild writes the specification block: {after}"));
    let record_at = after
        .find(record_begin.as_str())
        .unwrap_or_else(|| panic!("the rebuild keeps the record block: {after}"));
    assert!(
        specification_at < record_at,
        "the specification block precedes the record block even though the record was written first: {after}"
    );
    assert_eq!(
        after.matches(specification_begin.as_str()).count(),
        1,
        "the specification block appears once"
    );
    assert_eq!(
        after.matches(record_begin.as_str()).count(),
        1,
        "the record block appears once"
    );

    let third = build(&with_hypothesis, EXPERIMENT, &path, &no_promotion)
        .expect("a third identical build succeeds");
    assert_eq!(
        sorted(third.summary.unchanged.clone()),
        sorted(vec![name(SPECIFICATION_BLOCK), name(RECORD_BLOCK)]),
        "both blocks are unchanged: {:?}",
        third.summary
    );
    assert!(
        third.summary.recomputed.is_empty(),
        "nothing left to recompute: {:?}",
        third.summary.recomputed
    );
    assert_eq!(read(&path), after, "unchanged body means no rewrite");
}
