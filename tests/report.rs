//! Contract tests for the experiment report projection.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use tempfile::TempDir;

use kaya::model::{EntityType, Graph, GraphBuilder, Relationship};
use kaya::projection::report::{build, stale_authored};
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

    let summary = build(&graph(), "p/Experiment/exp1", &path, &promotion()).expect("report builds");
    assert_eq!(summary.recomputed, vec!["experiment-report".to_string()]);

    let after = read(&path);
    assert!(
        after.contains(AUTHORED_DOC),
        "authored and foreign bytes survive"
    );
    assert!(after.contains("<!-- kaya:begin generated:experiment-report -->"));
    assert!(after.contains("<!-- kaya:end generated:experiment-report -->"));
    assert!(after.contains("## Question"));
    assert!(after.contains("- p/Question/q1"));
    assert!(after.contains("- p/Hypothesis/h1"));
    assert!(
        after.find("- p/Run/r1").expect("r1 rendered")
            < after.find("- p/Run/r2").expect("r2 rendered"),
        "runs are sorted"
    );
}

#[test]
fn generated_sections_are_deterministic() {
    let left = TempDir::new().expect("temp dir");
    let right = TempDir::new().expect("temp dir");
    let left_path = report_path(&left);
    let right_path = report_path(&right);

    let left_summary =
        build(&graph(), "p/Experiment/exp1", &left_path, &promotion()).expect("first build");
    let right_summary =
        build(&graph(), "p/Experiment/exp1", &right_path, &promotion()).expect("second build");

    assert_eq!(read(&left_path), read(&right_path));
    assert_eq!(left_summary.sources, right_summary.sources);
    assert_eq!(left_summary.recomputed, right_summary.recomputed);
}

#[test]
fn second_build_with_unchanged_inputs_keeps_bytes() {
    let dir = TempDir::new().expect("temp dir");
    let path = report_path(&dir);

    build(&graph(), "p/Experiment/exp1", &path, &promotion()).expect("first build");
    let before = read(&path);

    let summary = build(&graph(), "p/Experiment/exp1", &path, &promotion()).expect("second build");
    assert_eq!(summary.unchanged, vec!["experiment-report".to_string()]);
    assert!(summary.recomputed.is_empty());
    assert_eq!(read(&path), before, "unchanged body means no rewrite");
}

#[test]
fn changed_graph_recomputes_the_report() {
    let dir = TempDir::new().expect("temp dir");
    let path = report_path(&dir);

    build(&graph(), "p/Experiment/exp1", &path, &promotion()).expect("first build");
    assert!(!read(&path).contains("p/Run/r3"));

    let summary = build(
        &graph_with_extra_run(),
        "p/Experiment/exp1",
        &path,
        &promotion(),
    )
    .expect("rebuild after a run lands");
    assert_eq!(summary.recomputed, vec!["experiment-report".to_string()]);
    assert!(summary.unchanged.is_empty());
    assert!(read(&path).contains("p/Run/r3"));
}

#[test]
fn source_map_names_every_rendered_fact() {
    let dir = TempDir::new().expect("temp dir");
    let path = report_path(&dir);

    let summary = build(&graph(), "p/Experiment/exp1", &path, &promotion()).expect("report builds");
    let sources = summary
        .sources
        .get("experiment-report")
        .expect("report has a source map");

    for id in [
        "p/Question/q1",
        "p/Hypothesis/h1",
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
            sources.iter().any(|source| source == id),
            "source map names {id}: {sources:?}"
        );
    }
    let mut sorted = sources.clone();
    sorted.sort();
    assert_eq!(sources, &sorted, "source ids are deterministic");
}

#[test]
fn interpretations_show_status_and_machine_flag() {
    let dir = TempDir::new().expect("temp dir");
    let path = report_path(&dir);

    build(&graph(), "p/Experiment/exp1", &path, &promotion()).expect("report builds");
    let after = read(&path);

    assert!(after.contains("## Interpretations"));
    assert!(after.contains("`challenged` counts contradicted H-001"));
    assert!(after.contains("`proposed` the effect is real"));
    assert!(after.contains("`established` H-002 survives"));
    assert!(after.contains("machine-proposed: yes"));
    assert!(after.contains("machine-proposed: no"));
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
