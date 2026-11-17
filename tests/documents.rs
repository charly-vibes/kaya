//! Contract tests for the document taxonomy: closed family/kind vocabulary,
//! family-scoped block names, the intent/record split, and silent omission of
//! empty kinds.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use tempfile::TempDir;

use kaya::model::{EntityType, Graph, GraphBuilder, Relationship};
use kaya::projection::remove_generated;
use kaya::projection::report::{
    Family, Kind, LABEL_LEGACY_BLOCK, LABEL_UNKNOWN_KIND, Omitted, REASON_NO_RECORDS, RECORD_BLOCK,
    SPECIFICATION_BLOCK, block_vocabulary, build, classify_block, refuse_kind,
};
use kaya::promotion::PromotionState;

const EXPERIMENT: &str = "p/Experiment/exp1";

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

/// One experiment carrying both declared intent and observed execution.
fn graph_with_intent_and_run() -> Graph {
    let mut builder = GraphBuilder::new();
    node(&mut builder, EXPERIMENT, EntityType::Experiment);
    node(&mut builder, "p/Question/q1", EntityType::Question);
    node(&mut builder, "p/Hypothesis/h1", EntityType::Hypothesis);
    node(&mut builder, "p/Run/r1", EntityType::Run);
    node(&mut builder, "p/Observation/o1", EntityType::Observation);
    node(&mut builder, "p/Finding/f1", EntityType::Finding);

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
    edge(&mut builder, Relationship::Executes, "p/Run/r1", EXPERIMENT);
    edge(
        &mut builder,
        Relationship::Observes,
        "p/Observation/o1",
        "p/Run/r1",
    );
    edge(
        &mut builder,
        Relationship::Derives,
        "p/Finding/f1",
        "p/Observation/o1",
    );
    builder.build().expect("graph is acyclic and complete")
}

/// Intent only: a question and a hypothesis, but no run events.
fn graph_without_runs() -> Graph {
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
    builder.build().expect("graph is valid")
}

/// A question and a bare run: the record has a run group and nothing else.
fn graph_with_intent_and_a_bare_run() -> Graph {
    let mut builder = GraphBuilder::new();
    node(&mut builder, EXPERIMENT, EntityType::Experiment);
    node(&mut builder, "p/Question/q1", EntityType::Question);
    node(&mut builder, "p/Run/r1", EntityType::Run);

    edge(
        &mut builder,
        Relationship::Investigates,
        "p/Question/q1",
        EXPERIMENT,
    );
    edge(&mut builder, Relationship::Executes, "p/Run/r1", EXPERIMENT);
    builder.build().expect("graph is valid")
}

/// Records only: one executed run and no declared intent at all.
fn graph_with_run_only() -> Graph {
    let mut builder = GraphBuilder::new();
    node(&mut builder, EXPERIMENT, EntityType::Experiment);
    node(&mut builder, "p/Run/r1", EntityType::Run);

    edge(&mut builder, Relationship::Executes, "p/Run/r1", EXPERIMENT);
    builder.build().expect("graph is valid")
}

fn promotion() -> PromotionState {
    PromotionState::default()
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

fn occurrences(haystack: &str, needle: &str) -> usize {
    haystack.matches(needle).count()
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
fn unknown_kind_is_refused_with_a_label_and_vocabulary() {
    assert_eq!(Kind::parse("experiment-report"), None);
    assert_eq!(Family::parse("archive"), None);

    let kind_error = refuse_kind("experiment-report");
    assert_eq!(kind_error.label, LABEL_UNKNOWN_KIND);
    assert!(
        kind_error.message.contains("\"experiment-report\""),
        "the refusal quotes the offending value: {}",
        kind_error.message
    );

    let family_error = refuse_kind("archive");
    assert_eq!(family_error.label, LABEL_UNKNOWN_KIND);
    assert!(
        family_error.message.contains("\"archive\""),
        "the refusal quotes the offending value: {}",
        family_error.message
    );

    let block_error = classify_block("exploration.experiment-report")
        .expect_err("a block outside the vocabulary is refused");
    assert_eq!(block_error.label, LABEL_LEGACY_BLOCK);
    assert!(
        block_error
            .message
            .contains("\"exploration.experiment-report\""),
        "the refusal quotes the offending block: {}",
        block_error.message
    );

    let vocabulary = block_vocabulary();
    for kind in Kind::ALL {
        let block = kind.block_name();
        assert!(
            vocabulary.contains(&block),
            "vocabulary names {block}: {vocabulary}"
        );
        for error in [&kind_error, &family_error, &block_error] {
            assert!(
                error.message.contains(&block),
                "refusal lists {block}: {}",
                error.message
            );
        }
    }
}

#[test]
fn legacy_block_name_is_refused_before_writing() {
    let dir = TempDir::new().expect("temp dir");
    let path = dir.path().join("dossier.md");
    let legacy_doc = "\
# Research notes

Authored prose stays put.

<!-- kaya:authored -->
My earlier reading of H-001.
<!-- kaya:end authored -->

<!-- wai:begin -->
foreign managed block bytes
<!-- wai:end -->

<!-- kaya:begin generated:experiment-report -->
## Runs

- p/Run/r1
<!-- kaya:end generated:experiment-report -->
";
    fs::write(&path, legacy_doc).expect("seed legacy document");

    let error = build(
        &graph_with_intent_and_run(),
        EXPERIMENT,
        &path,
        &promotion(),
    )
    .expect_err("a legacy block name is refused");

    assert_eq!(error.label, LABEL_LEGACY_BLOCK);
    assert!(
        error.message.contains("experiment-report"),
        "the refusal names the block: {}",
        error.message
    );
    assert_eq!(read(&path), legacy_doc, "refusal writes no bytes");
}

#[test]
fn intent_and_record_blocks_do_not_mix() {
    let dir = TempDir::new().expect("temp dir");
    let path = dir.path().join("dossier.md");

    build(
        &graph_with_intent_and_run(),
        EXPERIMENT,
        &path,
        &promotion(),
    )
    .expect("dossier builds");
    let after = read(&path);

    let specification = block_body(&after, SPECIFICATION_BLOCK);
    let record = block_body(&after, RECORD_BLOCK);

    for intent in ["p/Question/q1", "p/Hypothesis/h1"] {
        assert!(
            specification.contains(intent),
            "specification carries intent row {intent}"
        );
        assert!(
            !record.contains(intent),
            "record never carries intent row {intent}"
        );
    }
    for observed in ["p/Run/r1", "p/Observation/o1", "p/Finding/f1"] {
        assert!(
            record.contains(observed),
            "record carries observed row {observed}"
        );
        assert!(
            !specification.contains(observed),
            "specification never carries observed row {observed}"
        );
    }

    assert_eq!(
        occurrences(&after, &begin(SPECIFICATION_BLOCK)),
        1,
        "specification block appears once"
    );
    assert_eq!(
        occurrences(&after, &end(SPECIFICATION_BLOCK)),
        1,
        "specification block is closed once"
    );
    assert_eq!(
        occurrences(&after, &begin(RECORD_BLOCK)),
        1,
        "record block appears once"
    );
    assert_eq!(
        occurrences(&after, &end(RECORD_BLOCK)),
        1,
        "record block is closed once"
    );
}

#[test]
fn empty_kind_leaves_no_section_and_reports_omission() {
    let dir = TempDir::new().expect("temp dir");
    let path = dir.path().join("dossier.md");
    fs::write(&path, AUTHORED_DOC).expect("seed authored document");

    let expected = vec![Omitted {
        family: "exploration",
        kind: "experiment-record",
        reason: REASON_NO_RECORDS,
    }];
    let built = build(&graph_without_runs(), EXPERIMENT, &path, &promotion())
        .expect("intent-only dossier builds");
    assert_eq!(built.omitted, expected);

    let after = read(&path);
    assert!(
        !after.contains(&begin(RECORD_BLOCK)),
        "no record block is written: {after}"
    );
    assert!(
        !after.contains("generated:exploration.experiment-record"),
        "no record markers anywhere: {after}"
    );
    assert!(after.contains(AUTHORED_DOC), "authored bytes survive");

    // A record block left behind by an earlier build must be removed.
    let seeded = format!(
        "{after}\n{}\n## Runs\n\n- p/Run/r9\n{}\n",
        begin(RECORD_BLOCK),
        end(RECORD_BLOCK)
    );
    fs::write(&path, seeded).expect("seed stale record block");

    let rebuilt = build(&graph_without_runs(), EXPERIMENT, &path, &promotion())
        .expect("rebuild removes the empty kind");
    assert_eq!(rebuilt.omitted, expected);
    assert!(
        !rebuilt.summary.recomputed.is_empty(),
        "removing the record block counts as a recomputation: {:?}",
        rebuilt.summary.recomputed
    );

    let cleaned = read(&path);
    assert!(
        !cleaned.contains(&begin(RECORD_BLOCK)),
        "stale record block is gone: {cleaned}"
    );
    assert!(
        !cleaned.contains("p/Run/r9"),
        "stale record rows are gone: {cleaned}"
    );
    assert!(cleaned.contains(AUTHORED_DOC), "authored bytes survive");

    let repeat =
        build(&graph_without_runs(), EXPERIMENT, &path, &promotion()).expect("repeat build");
    assert!(
        repeat.summary.recomputed.is_empty(),
        "nothing left to recompute: {:?}",
        repeat.summary.recomputed
    );
    assert!(
        repeat
            .summary
            .unchanged
            .contains(&format!("{SPECIFICATION_BLOCK}:{EXPERIMENT}")),
        "repeat build reports the document unchanged: {:?}",
        repeat.summary.unchanged
    );
    assert_eq!(read(&path), cleaned, "unchanged body means no rewrite");
}

#[test]
fn specification_without_runs_renders_and_record_is_omitted() {
    let dir = TempDir::new().expect("temp dir");
    let path = dir.path().join("dossier.md");

    let built = build(&graph_without_runs(), EXPERIMENT, &path, &promotion())
        .expect("intent-only dossier builds");
    assert_eq!(
        built.omitted,
        vec![Omitted {
            family: "exploration",
            kind: "experiment-record",
            reason: REASON_NO_RECORDS,
        }]
    );

    let after = read(&path);
    let specification = block_body(&after, SPECIFICATION_BLOCK);
    assert!(
        specification.contains("p/Hypothesis/h1"),
        "specification renders the declared hypothesis: {specification}"
    );
    assert!(
        specification.contains("p/Question/q1"),
        "specification renders the declared question: {specification}"
    );
    assert!(
        !after.contains("generated:exploration.experiment-record"),
        "the record block is omitted: {after}"
    );
    assert!(
        !after.contains("_(none)_"),
        "no placeholder row is emitted: {after}"
    );
}

#[test]
fn every_kind_names_its_block_family_dot_kind() {
    let mut names = BTreeSet::new();
    for kind in Kind::ALL {
        let name = kind.block_name();
        assert_eq!(
            name,
            format!("{}.{}", kind.family().as_str(), kind.as_str()),
            "block name is <family>.<kind>"
        );
        assert_eq!(
            Kind::parse(kind.as_str()),
            Some(kind),
            "kind as_str round-trips"
        );
        assert!(
            kind.family().kinds().contains(&kind),
            "{} lists {name}",
            kind.family().as_str()
        );
        assert!(names.insert(name), "block names are distinct");
    }
    assert_eq!(names.len(), 11, "eleven distinct block names");

    for family in Family::ALL {
        for kind in family.kinds() {
            assert_eq!(
                kind.family(),
                family,
                "{} belongs to {}",
                kind.as_str(),
                family.as_str()
            );
        }
    }

    let error =
        classify_block("orientation.finding").expect_err("a family-mismatched block is refused");
    assert_eq!(error.label, LABEL_LEGACY_BLOCK);
    assert!(
        error.message.contains("orientation.finding"),
        "the refusal names the block: {}",
        error.message
    );
    assert_eq!(Kind::parse("finding"), None);
}

#[test]
fn record_block_omits_empty_groups() {
    let dir = TempDir::new().expect("temp dir");
    let path = dir.path().join("dossier.md");

    build(
        &graph_with_intent_and_a_bare_run(),
        EXPERIMENT,
        &path,
        &promotion(),
    )
    .expect("dossier builds");

    let after = read(&path);
    assert!(
        after.contains(&begin(RECORD_BLOCK)),
        "a run makes the record non-empty: {after}"
    );

    let record = block_body(&after, RECORD_BLOCK);
    assert!(
        record.contains("## Runs"),
        "the populated group keeps its heading: {record}"
    );
    assert!(
        record.contains("- p/Run/r1"),
        "the run row is rendered: {record}"
    );
    for empty_heading in ["## Observations", "## Findings", "## Interpretations"] {
        assert!(
            !record.contains(empty_heading),
            "an empty group renders no heading: {empty_heading} in {record}"
        );
    }
    assert!(
        !record.contains("_(none)_"),
        "an empty group renders no placeholder: {record}"
    );
}

#[test]
fn specification_kind_is_omitted_for_a_record_only_experiment() {
    let dir = TempDir::new().expect("temp dir");
    let path = dir.path().join("dossier.md");

    let built = build(&graph_with_run_only(), EXPERIMENT, &path, &promotion())
        .expect("record-only dossier builds");
    assert_eq!(
        built.omitted,
        vec![Omitted {
            family: "exploration",
            kind: "experiment-specification",
            reason: REASON_NO_RECORDS,
        }],
        "the empty specification kind is reported once"
    );

    let after = read(&path);
    assert!(
        !after.contains(&begin(SPECIFICATION_BLOCK)),
        "no specification block is written: {after}"
    );
    assert!(
        !after.contains("generated:exploration.experiment-specification"),
        "no specification markers anywhere: {after}"
    );
    assert!(
        after.contains(&begin(RECORD_BLOCK)),
        "the record block is written: {after}"
    );
    assert!(
        !built
            .summary
            .sources
            .contains_key(&format!("{SPECIFICATION_BLOCK}:{EXPERIMENT}")),
        "an omitted kind has no source-map entry: {:?}",
        built.summary.sources
    );
}

#[test]
fn classify_block_admits_every_vocabulary_name() {
    let mut expected_names = Vec::new();
    for kind in Kind::ALL {
        let name = kind.block_name();
        assert_eq!(
            classify_block(&name),
            Ok(kind),
            "every vocabulary block classifies back to its kind: {name}"
        );
        expected_names.push(name);
    }
    assert_eq!(
        block_vocabulary(),
        expected_names.join(", "),
        "the vocabulary is the canonical kind list in block order"
    );
    assert_eq!(
        block_vocabulary(),
        "orientation.project-overview, orientation.research-map, \
         exploration.exploration-brief, exploration.experiment-specification, \
         exploration.experiment-record, understanding.findings-and-analysis, \
         understanding.decision-record, reuse-handoff.technical-reference, \
         reuse-handoff.reproduction-package, reuse-handoff.knowledge-brief, \
         reuse-handoff.handoff",
        "the exact vocabulary string is pinned"
    );

    for family in Family::ALL {
        assert_eq!(
            Family::parse(family.as_str()),
            Some(family),
            "family {:?} round-trips through its vocabulary name",
            family.as_str()
        );
    }

    let error =
        classify_block("orientation.finding").expect_err("a family-mismatched block is refused");
    assert_eq!(
        error.label, LABEL_LEGACY_BLOCK,
        "a kind under the wrong family is a legacy block"
    );
}

#[test]
fn duplicate_generated_block_is_refused() {
    let dir = TempDir::new().expect("temp dir");
    let path = dir.path().join("dossier.md");
    // These bytes equal what the fixture graph renders, so a build that only
    // compares bodies would call both blocks unchanged instead of refusing.
    let record_block = format!(
        "{}\n## Runs\n\n- p/Run/r1\n\n## Observations\n\n- p/Observation/o1\n\n## Findings\n\n- p/Finding/f1\n{}",
        begin(RECORD_BLOCK),
        end(RECORD_BLOCK)
    );
    let duplicate = format!(
        "# Notes\n\n<!-- kaya:authored -->\nTwo identical blocks were pasted here.\n<!-- kaya:end authored -->\n\n{record_block}\n\n{record_block}"
    );
    fs::write(&path, &duplicate).expect("seed duplicate blocks");

    let error = build(
        &graph_with_intent_and_run(),
        EXPERIMENT,
        &path,
        &promotion(),
    )
    .expect_err("a duplicated generated block is refused");

    assert_eq!(
        error.label, "kaya.projection.managed_grammar",
        "a duplicate block breaks the managed grammar"
    );
    assert!(
        error.message.contains(RECORD_BLOCK),
        "the refusal names the duplicated block: {}",
        error.message
    );
    assert_eq!(read(&path), duplicate, "refusal writes no bytes");
}

#[test]
fn legacy_block_after_a_valid_block_is_refused() {
    let dir = TempDir::new().expect("temp dir");
    let path = dir.path().join("dossier.md");
    let specification_block = format!(
        "{}\n## Question\n\n- p/Question/q1\n{}",
        begin(SPECIFICATION_BLOCK),
        end(SPECIFICATION_BLOCK)
    );
    let legacy_block = format!(
        "{}\n## Runs\n\n- p/Run/r1\n{}",
        begin("experiment-report"),
        end("experiment-report")
    );
    let doc = format!("# Notes\n\n{specification_block}\n\n{legacy_block}\n");
    fs::write(&path, &doc).expect("seed a valid block followed by a legacy block");

    let error = build(
        &graph_with_intent_and_run(),
        EXPERIMENT,
        &path,
        &promotion(),
    )
    .expect_err("a legacy block is refused wherever it sits");

    assert_eq!(
        error.label, LABEL_LEGACY_BLOCK,
        "the refusal carries the legacy-block label"
    );
    assert!(
        error.message.contains("experiment-report"),
        "the refusal names the legacy block: {}",
        error.message
    );
    assert_eq!(read(&path), doc, "refusal writes no bytes");
}

#[test]
fn remove_generated_preserves_bytes_around_a_mid_document_block() {
    let document = concat!(
        "Authored prefix prose.\n",
        "\n",
        "<!-- kaya:begin generated:exploration.experiment-record -->\n",
        "## Runs\n",
        "\n",
        "- p/Run/r1\n",
        "<!-- kaya:end generated:exploration.experiment-record -->\n",
        "\n",
        "<!-- kaya:begin generated:exploration.experiment-specification -->\n",
        "## Hypothesis\n",
        "\n",
        "- p/Hypothesis/h1\n",
        "<!-- kaya:end generated:exploration.experiment-specification -->\n",
        "\n",
        "Trailing prose.\n",
    );
    let expected = concat!(
        "Authored prefix prose.\n",
        "\n",
        "<!-- kaya:begin generated:exploration.experiment-specification -->\n",
        "## Hypothesis\n",
        "\n",
        "- p/Hypothesis/h1\n",
        "<!-- kaya:end generated:exploration.experiment-specification -->\n",
        "\n",
        "Trailing prose.\n",
    );

    let removed =
        remove_generated(document, RECORD_BLOCK).expect("the document grammar is well formed");
    assert_eq!(
        removed, expected,
        "the block and its one separator line are gone; every other byte is identical"
    );
}
