//! Contract tests for the adapter-backed reuse kinds: `technical-reference`
//! from specodelic spec ids and `knowledge-brief` from turu/whisper entry ids.
//!
//! A reference is a row naming an authoritative id, never a payload copy and
//! never a ledger write. These tests pin the inputs selected from the modelled
//! graph, the refusal of malformed attributes, the exact single-line rows, the
//! omission of empty kinds, the canonical document/block order, the removal of
//! a stale block, and the preservation of authored bytes around a rebuild.

use std::fs;
use std::path::{Path, PathBuf};

use kaya::model::{EntityType, Graph, GraphBuilder};
use kaya::projection::report::{Kind, REASON_NO_RECORDS};
use kaya::projection::reuse::{
    HANDOFF_BLOCK, Inputs, KNOWLEDGE_BRIEF_BLOCK, LABEL_REUSE_ATTRIBUTE, REPRODUCTION_BLOCK,
    SourceRef, TECHNICAL_REFERENCE_BLOCK, build, documents, inputs,
};
use serde_json::{Value, json};
use tempfile::TempDir;

const TECHNICAL_REFERENCE_FILE: &str = "technical-reference.md";
const REPRODUCTION_FILE: &str = "reproduction-package.md";
const KNOWLEDGE_BRIEF_FILE: &str = "knowledge-brief.md";
const HANDOFF_FILE: &str = "handoff.md";
const SPEC_ID: &str = "kaya.documents.reuse_reference_only";
const ENTRY_ID: &str = "ea5b862dffa6ed8e778b8d7624e03bff7b6b1f7061c61cc0742c732a5a15ff36";

fn source(id: &str, reference: &str) -> SourceRef {
    SourceRef {
        id: id.to_owned(),
        reference: reference.to_owned(),
    }
}

/// Add one namespace-qualified node carrying `attributes`.
fn node(builder: &mut GraphBuilder, id: &str, entity: EntityType, attributes: Value) {
    builder
        .entity(id, entity, attributes)
        .expect("node id is namespace-qualified");
}

/// A graph with one node, for the refusal and blank-attribute cases.
fn graph_with(id: &str, entity: EntityType, attributes: Value) -> Graph {
    let mut builder = GraphBuilder::new();
    node(&mut builder, id, entity, attributes);
    builder.build().expect("a one-node graph builds")
}

/// A graph holding each reference-bearing node shape.
fn reference_graph() -> Graph {
    let mut builder = GraphBuilder::new();
    node(
        &mut builder,
        "alpha/Source/spec",
        EntityType::Source,
        json!({ "spec": SPEC_ID }),
    );
    node(
        &mut builder,
        "alpha/Source/entry",
        EntityType::Source,
        json!({ "entry": ENTRY_ID }),
    );
    node(
        &mut builder,
        "alpha/Source/revision",
        EntityType::Source,
        json!({ "path": "src/main.rs", "commit": "9f2c1a" }),
    );
    node(
        &mut builder,
        "alpha/Source/blank",
        EntityType::Source,
        json!({ "spec": "   ", "entry": "", "path": "  ", "commit": "  " }),
    );
    builder.build().expect("the reference graph builds")
}

/// The directory the reuse documents are projected under.
fn out_dir(dir: &TempDir) -> PathBuf {
    dir.path().join("docs").join("research")
}

fn read(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("{} is readable: {error}", path.display()))
}

/// The sorted file names directly under `out`.
fn file_names(out: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(out)
        .unwrap_or_else(|error| panic!("{} is readable: {error}", out.display()))
        .map(|entry| {
            entry
                .expect("directory entry is readable")
                .file_name()
                .into_string()
                .expect("file name is UTF-8")
        })
        .collect();
    names.sort();
    names
}

/// The exact bytes of one generated block, markers excluded.
fn body<'a>(document: &'a str, block_name: &str) -> &'a str {
    let begin = format!("<!-- kaya:begin generated:{block_name} -->");
    let end = format!("<!-- kaya:end generated:{block_name} -->");
    let mut start = document
        .find(&begin)
        .unwrap_or_else(|| panic!("document lacks the {block_name} begin marker:\n{document}"))
        + begin.len();
    if document[start..].starts_with('\n') {
        start += 1;
    }
    let length = document[start..]
        .find(&end)
        .unwrap_or_else(|| panic!("document lacks the {block_name} end marker:\n{document}"));
    &document[start..start + length]
}

#[track_caller]
fn assert_block(document: &str, block_name: &str, expected: &[&str]) {
    assert_eq!(
        body(document, block_name),
        format!("{}\n", expected.join("\n")),
        "the {block_name} block renders exactly these rows"
    );
}

fn built_inputs(graph: &Graph) -> Inputs {
    inputs(graph).expect("well-formed attributes project")
}

#[test]
fn inputs_select_spec_and_entry_references() {
    let selected = built_inputs(&reference_graph());

    // A specodelic spec id and a turu entry id each select their own kind; a
    // blank attribute selects nothing, and path+commit still selects a
    // revision for the reproduction package.
    assert_eq!(
        selected.specs,
        vec![source("alpha/Source/spec", SPEC_ID)],
        "only the non-blank spec attribute becomes a technical-reference row"
    );
    assert_eq!(
        selected.entries,
        vec![source("alpha/Source/entry", ENTRY_ID)],
        "only the non-blank entry attribute becomes a knowledge-brief row"
    );
    assert_eq!(
        selected.revisions,
        vec![source("alpha/Source/revision", "src/main.rs@9f2c1a")],
        "the revision selection is unchanged"
    );

    // A present non-string spec is a labeled refusal naming node and attribute.
    let error = inputs(&graph_with(
        "alpha/Source/bad",
        EntityType::Source,
        json!({ "spec": 7 }),
    ))
    .expect_err("a non-string spec is refused");
    assert_eq!(error.label, LABEL_REUSE_ATTRIBUTE);
    assert!(
        error.message.contains("alpha/Source/bad") && error.message.contains("\"spec\""),
        "the refusal names the node and attribute: {}",
        error.message
    );

    // The same refusal applies to `entry`.
    let error = inputs(&graph_with(
        "alpha/Source/bad",
        EntityType::Source,
        json!({ "entry": true }),
    ))
    .expect_err("a non-string entry is refused");
    assert_eq!(error.label, LABEL_REUSE_ATTRIBUTE);
    assert!(error.message.contains("\"entry\""), "{}", error.message);

    // The three selections are independent: one source may carry all of them.
    let combined = built_inputs(&graph_with(
        "alpha/Source/all",
        EntityType::Source,
        json!({
            "spec": "spec-x",
            "entry": "entry-x",
            "path": "src/lib.rs",
            "commit": "abc123",
        }),
    ));
    assert_eq!(combined.specs, vec![source("alpha/Source/all", "spec-x")]);
    assert_eq!(
        combined.entries,
        vec![source("alpha/Source/all", "entry-x")]
    );
    assert_eq!(
        combined.revisions,
        vec![source("alpha/Source/all", "src/lib.rs@abc123")],
        "a source carrying every reference contributes one row to each kind"
    );
}

#[test]
fn technical_reference_renders_spec_ids() {
    let dir = TempDir::new().expect("temp dir");
    let out = out_dir(&dir);
    fs::create_dir_all(&out).expect("output directory");

    let built = build(&built_inputs(&reference_graph()), &out).expect("references build");

    let document = read(&out.join(TECHNICAL_REFERENCE_FILE));
    assert_block(
        &document,
        TECHNICAL_REFERENCE_BLOCK,
        &["### Specs", &format!("- alpha/Source/spec — {SPEC_ID}")],
    );

    assert_eq!(
        file_names(&out),
        vec![
            KNOWLEDGE_BRIEF_FILE.to_owned(),
            REPRODUCTION_FILE.to_owned(),
            TECHNICAL_REFERENCE_FILE.to_owned(),
        ],
        "a document per non-empty kind, and none for the empty kinds"
    );
    assert_eq!(
        built.summary.recomputed,
        vec![
            TECHNICAL_REFERENCE_BLOCK.to_owned(),
            REPRODUCTION_BLOCK.to_owned(),
            KNOWLEDGE_BRIEF_BLOCK.to_owned(),
        ],
        "blocks are recomputed in canonical kind order"
    );
    let omitted: Vec<&str> = built.omitted.iter().map(|entry| entry.kind).collect();
    assert_eq!(
        omitted,
        vec![Kind::Handoff.as_str()],
        "each empty kind is reported exactly once"
    );
    assert!(
        built
            .omitted
            .iter()
            .all(|entry| entry.reason == REASON_NO_RECORDS),
        "empty kinds carry the no-records reason"
    );
}

#[test]
fn knowledge_brief_names_every_entry_id() {
    let dir = TempDir::new().expect("temp dir");
    let out = out_dir(&dir);
    fs::create_dir_all(&out).expect("output directory");
    let mut builder = GraphBuilder::new();
    node(
        &mut builder,
        "alpha/Source/b",
        EntityType::Source,
        json!({ "entry": "bbbb" }),
    );
    node(
        &mut builder,
        "alpha/Source/a",
        EntityType::Source,
        json!({ "entry": "aaaa" }),
    );
    node(
        &mut builder,
        "alpha/Artifact/not-a-source",
        EntityType::Artifact,
        json!({ "entry": "cccc" }),
    );

    let built = build(&built_inputs(&builder.build().expect("graph builds")), &out)
        .expect("knowledge briefs build");

    let document = read(&out.join(KNOWLEDGE_BRIEF_FILE));
    assert_block(
        &document,
        KNOWLEDGE_BRIEF_BLOCK,
        &[
            "### Entries",
            "- alpha/Source/a — aaaa",
            "- alpha/Source/b — bbbb",
        ],
    );
    for line in body(&document, KNOWLEDGE_BRIEF_BLOCK)
        .lines()
        .filter(|line| !line.starts_with("### "))
    {
        assert!(
            line.contains(" — ") && line.starts_with("- alpha/Source/"),
            "every rendered item names its node and entry id: {line}"
        );
    }
    assert_eq!(
        built.summary.recomputed,
        vec![KNOWLEDGE_BRIEF_BLOCK.to_owned()]
    );
    assert!(
        !out.join(TECHNICAL_REFERENCE_FILE).exists(),
        "an empty technical-reference writes no file"
    );
}

#[test]
fn ledger_gains_no_entity_and_only_markdown_is_written() {
    let dir = TempDir::new().expect("temp dir");
    let out = out_dir(&dir);
    fs::create_dir_all(&out).expect("output directory");
    let ledger = dir.path().join(".kaya").join("ledger.jsonl");
    fs::create_dir_all(ledger.parent().expect("ledger directory")).expect("ledger directory");
    let seed = "{\"id\":\"ev-1\",\"schema\":\"kaya.event/v0\"}\n";
    fs::write(&ledger, seed).expect("seed the ledger");

    let selected = built_inputs(&reference_graph());
    let before = selected.clone();
    build(&selected, &out).expect("build");

    assert_eq!(
        selected, before,
        "building mutates no input reference and creates no entity"
    );
    assert_eq!(
        fs::read_to_string(&ledger).expect("ledger readable"),
        seed,
        "the ledger is byte-identical after the build"
    );
    assert!(
        file_names(&out).iter().all(|name| name.ends_with(".md")),
        "the projection writes markdown only: {:?}",
        file_names(&out)
    );
}

#[test]
fn empty_kinds_are_omitted_once_and_write_no_file() {
    let dir = TempDir::new().expect("temp dir");
    let out = out_dir(&dir);
    let built = build(&Inputs::default(), &out).expect("an empty projection builds");

    let omitted: Vec<&str> = built.omitted.iter().map(|entry| entry.kind).collect();
    assert_eq!(
        omitted,
        vec![
            Kind::TechnicalReference.as_str(),
            Kind::ReproductionPackage.as_str(),
            Kind::KnowledgeBrief.as_str(),
            Kind::Handoff.as_str(),
        ],
        "every reuse kind is reported once, in canonical order"
    );
    assert!(
        built
            .omitted
            .iter()
            .all(|entry| entry.reason == REASON_NO_RECORDS),
        "each omission carries its reason"
    );
    assert!(
        !out.exists(),
        "an empty projection writes no document and creates no directory"
    );
}

#[test]
fn stale_adapter_blocks_are_removed() {
    for (file, block) in [
        (TECHNICAL_REFERENCE_FILE, TECHNICAL_REFERENCE_BLOCK),
        (KNOWLEDGE_BRIEF_FILE, KNOWLEDGE_BRIEF_BLOCK),
    ] {
        let dir = TempDir::new().expect("temp dir");
        let out = out_dir(&dir);
        fs::create_dir_all(&out).expect("output directory");
        let path = out.join(file);
        fs::write(
            &path,
            format!("<!-- kaya:begin generated:{block} -->\n- old — old\n<!-- kaya:end generated:{block} -->\n"),
        )
        .expect("stale document");

        let built = build(&Inputs::default(), &out).expect("empty projection builds");

        assert!(!path.exists(), "{file} left blank is deleted");
        assert!(
            built.summary.recomputed.contains(&block.to_owned()),
            "the {block} removal is reported as a recomputation"
        );
    }
}

#[test]
fn block_names_are_vocabulary_names() {
    let dir = TempDir::new().expect("temp dir");
    let out = out_dir(&dir);
    let expected = [
        (
            TECHNICAL_REFERENCE_BLOCK,
            TECHNICAL_REFERENCE_FILE,
            Kind::TechnicalReference,
        ),
        (
            REPRODUCTION_BLOCK,
            REPRODUCTION_FILE,
            Kind::ReproductionPackage,
        ),
        (
            KNOWLEDGE_BRIEF_BLOCK,
            KNOWLEDGE_BRIEF_FILE,
            Kind::KnowledgeBrief,
        ),
        (HANDOFF_BLOCK, HANDOFF_FILE, Kind::Handoff),
    ];

    let pairs = documents(&out);
    assert_eq!(pairs.len(), expected.len(), "one document per reuse kind");

    for ((block, path), (expected_block, file, kind)) in pairs.iter().zip(expected) {
        assert_eq!(
            *block, expected_block,
            "documents follow canonical kind order"
        );
        assert_eq!(
            path,
            &out.join(file),
            "the document path is the kind's file"
        );
        assert_eq!(
            kind.block_name(),
            *block,
            "the block name is the vocabulary name"
        );
        assert_eq!(
            Kind::parse(block.rsplit('.').next().expect("a suffix")),
            Some(kind),
            "the block name parses back to its kind"
        );
    }
}

#[test]
fn authored_bytes_survive_around_new_blocks() {
    let cases = [
        (
            TECHNICAL_REFERENCE_FILE,
            TECHNICAL_REFERENCE_BLOCK,
            "### Specs",
            format!("- alpha/Source/spec — {SPEC_ID}"),
        ),
        (
            KNOWLEDGE_BRIEF_FILE,
            KNOWLEDGE_BRIEF_BLOCK,
            "### Entries",
            format!("- alpha/Source/entry — {ENTRY_ID}"),
        ),
    ];

    for (file, block, heading, row) in cases {
        let dir = TempDir::new().expect("temp dir");
        let out = out_dir(&dir);
        fs::create_dir_all(&out).expect("output directory");
        let path = out.join(file);
        let authored = concat!(
            "# Technical reference\n\n",
            "<!-- kaya:authored -->\nPinned by hand.\n<!-- kaya:end authored -->\n\n",
            "<!-- other-tool:begin -->\nforeign\n<!-- other-tool:end -->\n",
        );
        fs::write(&path, authored).expect("authored document");

        build(&built_inputs(&reference_graph()), &out).expect("build");

        let document = read(&path);
        assert!(
            document.starts_with(authored),
            "authored and foreign bytes survive in {file}: {document}"
        );
        assert_block(&document, block, &[heading, &row]);
    }
}
