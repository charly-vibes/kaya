//! Contract tests for the `orientation.project-overview` projection: which
//! records may render, omission of an empty overview, preservation of authored
//! bytes, validation of legacy block names before writing, and the block name
//! drawn from the closed family/kind vocabulary.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use tempfile::TempDir;

use kaya::model::{EntityType, Graph, GraphBuilder};
use kaya::projection::report::{
    Family, Kind, LABEL_LEGACY_BLOCK, OVERVIEW_BLOCK, Omitted, REASON_NO_RECORDS, build_overview,
    classify_block,
};

const OVERVIEW: &str = "docs/research/overview.md";

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

/// The three admissible groups plus every record the overview must never
/// render: a run, the artifact it captured with its commit sha, an
/// observation, and a decision.
fn overview_graph() -> Graph {
    let mut builder = GraphBuilder::new();
    // Insertion order is deliberately not id order.
    titled(
        &mut builder,
        "p/Project/project-kaya",
        EntityType::Project,
        "Kaya",
    );
    // A project without a title renders its key.
    bare(&mut builder, "p/Project/project-two", EntityType::Project);
    titled(
        &mut builder,
        "p/Question/q-one",
        EntityType::Question,
        "What is this?",
    );
    bare(&mut builder, "p/Question/q-two", EntityType::Question);
    titled(
        &mut builder,
        "p/Finding/f-two",
        EntityType::Finding,
        "Second finding",
    );
    titled(
        &mut builder,
        "p/Finding/f-one",
        EntityType::Finding,
        "First finding",
    );
    titled(
        &mut builder,
        "p/Run/r-one",
        EntityType::Run,
        "ran the suite",
    );
    node(
        &mut builder,
        "p/Artifact/a-one",
        EntityType::Artifact,
        json!({
            "kind": "reports",
            "path": "target/report.json",
            "commit": "deadbeef",
        }),
    );
    titled(
        &mut builder,
        "p/Observation/o-one",
        EntityType::Observation,
        "Latency fell",
    );
    titled(
        &mut builder,
        "p/Decision/d-one",
        EntityType::Decision,
        "Ship the cache",
    );
    builder.build().expect("graph is acyclic and complete")
}

fn document_path(dir: &TempDir) -> PathBuf {
    let path = dir.path().join(OVERVIEW);
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
# Overview notes

Intro prose that must survive.

<!-- kaya:authored -->
This project records what it tried and what it learned.
<!-- kaya:cites p/Finding/f-one -->
<!-- kaya:end authored -->

<!-- wai:begin -->
foreign managed block bytes
<!-- wai:end -->

Tail prose.
";

#[test]
fn overview_renders_project_questions_and_findings_only() {
    let dir = TempDir::new().expect("temp dir");
    let path = document_path(&dir);

    let built = build_overview(&overview_graph(), &path).expect("the project overview builds");

    let after = read(&path);
    assert_eq!(
        block_lines(&after, OVERVIEW_BLOCK),
        lines(&[
            "### Project",
            "- p/Project/project-kaya — Kaya",
            "- p/Project/project-two — project-two",
            "### Questions",
            "- p/Question/q-one — What is this?",
            "- p/Question/q-two — q-two",
            "### Findings",
            "- p/Finding/f-one — First finding",
            "- p/Finding/f-two — Second finding",
        ]),
        "the overview renders the three admissible groups in id order: {after}"
    );

    let body = block_body(&after, OVERVIEW_BLOCK);
    for forbidden in [
        "p/Run/r-one",
        "ran the suite",
        "### Runs",
        "p/Artifact/a-one",
        "target/report.json",
        "deadbeef",
        "### Artifacts",
        "### Captures",
        "p/Observation/o-one",
        "Latency fell",
        "p/Decision/d-one",
        "Ship the cache",
    ] {
        assert!(
            !body.contains(forbidden),
            "a run, artifact, observation, or decision never enters the overview ({forbidden}): \
             {body}"
        );
    }

    assert_eq!(
        built.summary.recomputed,
        vec![OVERVIEW_BLOCK.to_string()],
        "the overview is one block keyed by its block name: {:?}",
        built.summary.recomputed
    );
    assert!(
        built.omitted.is_empty(),
        "every admissible group has a record, so nothing is omitted: {:?}",
        built.omitted
    );
    assert_eq!(
        built.summary.sources.get(OVERVIEW_BLOCK),
        Some(&lines(&[
            "p/Finding/f-one",
            "p/Finding/f-two",
            "p/Project/project-kaya",
            "p/Project/project-two",
            "p/Question/q-one",
            "p/Question/q-two",
        ])),
        "the source map traces the rendered records only: {:?}",
        built.summary.sources
    );
}

#[test]
fn partial_overview_renders_only_the_populated_section() {
    let dir = TempDir::new().expect("temp dir");
    let path = document_path(&dir);
    let mut builder = GraphBuilder::new();
    titled(
        &mut builder,
        "p/Project/project-kaya",
        EntityType::Project,
        "Kaya",
    );
    let graph = builder.build().expect("graph is valid");

    let built = build_overview(&graph, &path).expect("a project-only graph builds");

    let after = read(&path);
    assert_eq!(
        block_lines(&after, OVERVIEW_BLOCK),
        lines(&["### Project", "- p/Project/project-kaya — Kaya"]),
        "only the populated group renders — no Questions, no Findings, no placeholder: {after}"
    );
    let body = block_body(&after, OVERVIEW_BLOCK);
    for absent in ["### Questions", "### Findings", "_(none)_", "(none)"] {
        assert!(
            !body.contains(absent),
            "an empty section renders neither heading nor placeholder ({absent}): {body}"
        );
    }
    assert_eq!(
        built.summary.sources.get(OVERVIEW_BLOCK),
        Some(&lines(&["p/Project/project-kaya"])),
        "the source map traces only the rendered project: {:?}",
        built.summary.sources
    );
    assert!(
        built.omitted.is_empty(),
        "an overview with one populated group is not omitted: {:?}",
        built.omitted
    );
}

#[test]
fn empty_overview_is_omitted_and_removes_the_document() {
    let dir = TempDir::new().expect("temp dir");
    let path = document_path(&dir);
    let empty = GraphBuilder::new()
        .build()
        .expect("an empty graph is valid");
    let expected = vec![Omitted {
        family: "orientation",
        kind: "project-overview",
        reason: REASON_NO_RECORDS,
    }];

    // No file exists and the empty graph creates none.
    let absent = build_overview(&empty, &path).expect("an empty overview builds");
    assert!(!path.exists(), "an empty graph writes no overview document");
    assert_eq!(
        absent.omitted, expected,
        "the empty kind is reported once, naming its family and kind"
    );
    assert!(
        absent.summary.recomputed.is_empty(),
        "nothing is recomputed in an absent document: {:?}",
        absent.summary.recomputed
    );

    // A block left behind by an earlier build is removed; the removal leaves
    // only whitespace, so the document itself is deleted.
    let seeded = format!(
        "{}\n### Project\n\n- p/Project/old — Old project\n{}\n",
        begin(OVERVIEW_BLOCK),
        end(OVERVIEW_BLOCK)
    );
    fs::write(&path, &seeded).expect("seed a stale overview block");

    let removed = build_overview(&empty, &path).expect("the empty overview removes its block");
    assert!(
        !path.exists(),
        "a document left with only whitespace is deleted, never written as zero bytes"
    );
    assert_eq!(removed.omitted, expected, "the omission is still reported");
    assert_eq!(
        removed.summary.recomputed,
        vec![OVERVIEW_BLOCK.to_string()],
        "the removed block is reported under the block name: {:?}",
        removed.summary.recomputed
    );
    assert!(
        removed.summary.unchanged.is_empty(),
        "a removal is not an unchanged block: {:?}",
        removed.summary.unchanged
    );

    // A whitespace-only document is deleted rather than kept as a stub.
    fs::write(&path, "  \n\n").expect("seed a whitespace document");
    let whitespace =
        build_overview(&empty, &path).expect("a whitespace document is treated as absent");
    assert!(
        !path.exists(),
        "a whitespace-only document with no records is deleted"
    );
    assert_eq!(
        whitespace.omitted, expected,
        "the omission is reported on every build"
    );
}

#[test]
fn omitted_overview_keeps_authored_prose_and_settles_across_builds() {
    let dir = TempDir::new().expect("temp dir");
    let path = document_path(&dir);
    let seeded = format!(
        "# Overview notes\n\
         \n\
         Intro prose that must survive.\n\
         \n\
         <!-- kaya:authored -->\n\
         The project records what it tried and what it learned.\n\
         <!-- kaya:cites p/Finding/f-one -->\n\
         <!-- kaya:end authored -->\n\
         \n\
         {}\n\
         ### Findings\n\
         \n\
         - p/Finding/f-one — First finding\n\
         {}\n\
         \n\
         Tail prose.\n",
        begin(OVERVIEW_BLOCK),
        end(OVERVIEW_BLOCK)
    );
    fs::write(&path, &seeded).expect("seed authored prose plus a stale overview block");
    let empty = GraphBuilder::new()
        .build()
        .expect("an empty graph is valid");
    let expected = vec![Omitted {
        family: "orientation",
        kind: "project-overview",
        reason: REASON_NO_RECORDS,
    }];

    // The stale block is removed; the authored bytes around it are not.
    let removed = build_overview(&empty, &path).expect("an empty graph removes its stale block");
    let after = read(&path);
    for survived in [
        "# Overview notes",
        "Intro prose that must survive.",
        "<!-- kaya:authored -->",
        "The project records what it tried and what it learned.",
        "<!-- kaya:cites p/Finding/f-one -->",
        "<!-- kaya:end authored -->",
        "Tail prose.",
    ] {
        assert!(
            after.contains(survived),
            "authored prose survives the omission ({survived}): {after}"
        );
    }
    assert!(
        !after.contains(&begin(OVERVIEW_BLOCK)) && !after.contains(&end(OVERVIEW_BLOCK)),
        "the stale block's begin and end markers are gone: {after}"
    );
    assert!(
        !after.contains("p/Finding/f-one — First finding"),
        "the stale block's rows are gone: {after}"
    );
    assert!(
        !after.contains("<!-- kaya:begin generated:"),
        "no generated block survives the omission: {after}"
    );
    assert_eq!(removed.omitted, expected, "the omission is reported");
    assert_eq!(
        removed.summary.recomputed,
        vec![OVERVIEW_BLOCK.to_string()],
        "the removed block is reported under the vocabulary name: {:?}",
        removed.summary.recomputed
    );
    assert!(
        removed.summary.unchanged.is_empty(),
        "a removal is not an unchanged block: {:?}",
        removed.summary.unchanged
    );
    assert!(
        removed.summary.sources.is_empty(),
        "nothing is rendered, so nothing is traced: {:?}",
        removed.summary.sources
    );

    // The second and third builds change nothing: no rewrite, no report.
    for build_number in [2, 3] {
        let again = build_overview(&empty, &path).expect("an authored document rebuilds");
        assert!(
            again.summary.recomputed.is_empty(),
            "build {build_number} rewrites nothing: {:?}",
            again.summary.recomputed
        );
        assert!(
            again.summary.unchanged.is_empty(),
            "build {build_number} has no block to leave unchanged: {:?}",
            again.summary.unchanged
        );
        assert_eq!(
            again.omitted, expected,
            "build {build_number} still reports the omitted kind"
        );
        assert_eq!(read(&path), after, "build {build_number} is byte-identical");
    }
}

#[test]
fn authored_bytes_survive_and_second_build_is_unchanged() {
    let dir = TempDir::new().expect("temp dir");
    let path = document_path(&dir);
    fs::write(&path, AUTHORED_DOC).expect("seed authored document");

    let built = build_overview(&overview_graph(), &path).expect("the project overview builds");
    assert_eq!(
        built.summary.recomputed,
        vec![OVERVIEW_BLOCK.to_string()],
        "the first build writes the overview block: {:?}",
        built.summary.recomputed
    );

    let after = read(&path);
    assert!(
        after.starts_with("# Overview notes\n\nIntro prose that must survive.\n"),
        "intro prose survives verbatim: {after}"
    );
    assert!(
        after.contains(
            "<!-- kaya:authored -->\nThis project records what it tried and what it learned.\n\
             <!-- kaya:cites p/Finding/f-one -->\n<!-- kaya:end authored -->"
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
        .find(&begin(OVERVIEW_BLOCK))
        .expect("the overview block is appended");
    assert!(
        tail < appended,
        "trailing prose stays ahead of the appended block: {after}"
    );

    let again = build_overview(&overview_graph(), &path).expect("repeat build succeeds");
    assert!(
        again.summary.recomputed.is_empty(),
        "nothing is rewritten when the block already matches: {:?}",
        again.summary.recomputed
    );
    assert_eq!(
        again.summary.unchanged,
        vec![OVERVIEW_BLOCK.to_string()],
        "a byte-identical rebuild reports the block unchanged: {:?}",
        again.summary.unchanged
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
# Overview notes

Authored prose stays put.

<!-- kaya:authored -->
My earlier summary.
<!-- kaya:end authored -->

<!-- kaya:begin generated:explanation -->
### Explanations

- p/Finding/f-one — because
<!-- kaya:end generated:explanation -->
";
    fs::write(&path, legacy_doc).expect("seed legacy document");

    let error = build_overview(&overview_graph(), &path)
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

#[test]
fn overview_block_name_is_the_vocabulary_name() {
    let name = Kind::ProjectOverview.block_name();
    assert_eq!(
        OVERVIEW_BLOCK, name,
        "the build constant is the kind's generated block name"
    );
    assert_eq!(
        name,
        format!(
            "{}.{}",
            Family::Orientation.as_str(),
            Kind::ProjectOverview.as_str()
        ),
        "the block name is exactly the family and kind it declares"
    );
    assert_eq!(name, "orientation.project-overview");
    assert_eq!(Kind::ProjectOverview.family(), Family::Orientation);
    assert_eq!(
        classify_block(&name).ok(),
        Some(Kind::ProjectOverview),
        "the closed vocabulary admits the declared family and kind"
    );
    // The vocabulary admits a family/kind pair other than the overview's too,
    // so the scenario is about admission, not about one hard-coded name.
    assert_eq!(
        classify_block("exploration.experiment-record").ok(),
        Some(Kind::ExperimentRecord),
        "a projection declaring exploration and experiment-record is admitted"
    );
    assert_eq!(
        Kind::ExperimentRecord.block_name(),
        "exploration.experiment-record",
        "the admitted build entry names both the family and the kind"
    );

    // A projection declaring this family and kind builds, and its build entry
    // carries both names.
    let dir = TempDir::new().expect("temp dir");
    let path = document_path(&dir);
    let mut builder = GraphBuilder::new();
    titled(
        &mut builder,
        "p/Project/project-kaya",
        EntityType::Project,
        "Kaya",
    );
    let graph = builder.build().expect("graph is valid");
    let built = build_overview(&graph, &path).expect("the declared kind is admitted");
    assert_eq!(
        built.summary.recomputed,
        vec![name.clone()],
        "the build entry names the block (family and kind): {:?}",
        built.summary.recomputed
    );
    assert!(
        built.summary.sources.contains_key(&name),
        "the source map is keyed by the same block name: {:?}",
        built.summary.sources
    );
    assert!(
        read(&path).contains(&begin(&name)),
        "the written document carries the vocabulary block name"
    );
}
