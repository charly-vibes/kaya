//! Contract tests for the reuse-handoff reference projections.
//!
//! A reuse document is assembled from references to authoritative sources —
//! never from payload copies and never from a ledger write. These tests pin
//! the inputs selected from the modelled graph, the refusal of malformed
//! attributes, the exact single-line rendered rows, the deduplication of
//! identical rows, the family-scoped block names, the omission of empty kinds
//! (including deleting a document whose last block was removed), and the
//! refusal of a legacy generated block before a single byte is written.

use std::fs;
use std::path::{Path, PathBuf};

use kaya::model::{EntityType, Graph, GraphBuilder};
use kaya::projection::report::{Kind, LABEL_LEGACY_BLOCK, Omitted, REASON_NO_RECORDS};
use kaya::projection::reuse::{
    HANDOFF_BLOCK, Inputs, LABEL_REUSE_ATTRIBUTE, REPRODUCTION_BLOCK, SourceRef, build, inputs,
};
use serde_json::{Value, json};
use tempfile::TempDir;

const REPRODUCTION_FILE: &str = "reproduction-package.md";
const HANDOFF_FILE: &str = "handoff.md";
const LABEL_REUSE_ATTRIBUTE_LITERAL: &str = "kaya.projection.reuse_attribute";

/// Two ledger events seeded before a build; a projection must add none.
const LEDGER_SEED: &str = concat!(
    "{\"id\":\"ev-1\",\"schema\":\"kaya.event/v0\",\"type\":\"artifact.captured\"}\n",
    "{\"id\":\"ev-2\",\"schema\":\"kaya.event/v0\",\"type\":\"artifact.captured\"}\n",
);

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

/// A graph holding every reference-bearing node shape.
fn reference_graph() -> Graph {
    let mut builder = GraphBuilder::new();
    node(
        &mut builder,
        "alpha/Source/one",
        EntityType::Source,
        json!({ "path": "src/main.rs", "commit": "9f2c1a" }),
    );
    node(
        &mut builder,
        "alpha/Source/no-commit",
        EntityType::Source,
        json!({ "path": "docs/x.md" }),
    );
    node(
        &mut builder,
        "alpha/Run/two",
        EntityType::Run,
        json!({ "commands": ["a", "b"] }),
    );
    node(
        &mut builder,
        "alpha/Run/both",
        EntityType::Run,
        json!({ "command": "cargo test", "commands": ["cargo test", "cargo build"] }),
    );
    node(
        &mut builder,
        "alpha/Artifact/handoff",
        EntityType::Artifact,
        json!({ "kind": "handoffs", "path": ".wai/projects/kaya/handoffs/1.md" }),
    );
    node(
        &mut builder,
        "alpha/Artifact/design",
        EntityType::Artifact,
        json!({ "kind": "designs", "path": ".wai/projects/kaya/designs/1.md" }),
    );
    node(
        &mut builder,
        "alpha/Artifact/no-path",
        EntityType::Artifact,
        json!({ "kind": "handoffs" }),
    );
    builder.build().expect("the reference graph builds")
}

/// The directory the reuse documents are projected under.
fn out_dir(dir: &TempDir) -> PathBuf {
    dir.path().join("docs").join("research")
}

fn reproduction_path(dir: &TempDir) -> PathBuf {
    out_dir(dir).join(REPRODUCTION_FILE)
}

fn handoff_path(dir: &TempDir) -> PathBuf {
    out_dir(dir).join(HANDOFF_FILE)
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
    // The marker line's newline belongs to the marker, not the body.
    if document[start..].starts_with('\n') {
        start += 1;
    }
    let length = document[start..]
        .find(&end)
        .unwrap_or_else(|| panic!("document lacks the {block_name} end marker:\n{document}"));
    &document[start..start + length]
}

/// Assert the exact bytes of a generated block, one expected line per entry.
#[track_caller]
fn assert_block(document: &str, block_name: &str, expected: &[&str]) {
    assert_eq!(
        body(document, block_name),
        format!("{}\n", expected.join("\n")),
        "the {block_name} block renders exactly these rows"
    );
}

#[test]
fn inputs_selects_references_from_the_graph() {
    let inputs = inputs(&reference_graph()).expect("well-formed attributes project");

    // A source contributes `path@commit` only when both attributes are
    // strings; a source missing its commit contributes nothing.
    assert_eq!(
        inputs.revisions,
        vec![source("alpha/Source/one", "src/main.rs@9f2c1a")],
        "only the fully attributed source contributes a revision"
    );

    // A `commands` array contributes one row per entry, naming the run. A run
    // carrying both forms contributes both.
    assert_eq!(
        inputs
            .commands
            .iter()
            .filter(|row| row.id == "alpha/Run/two")
            .map(|row| row.reference.as_str())
            .collect::<Vec<_>>(),
        vec!["a", "b"],
        "every commands entry becomes a row with the run id"
    );
    assert_eq!(
        inputs
            .commands
            .iter()
            .filter(|row| row.id == "alpha/Run/both")
            .map(|row| row.reference.as_str())
            .collect::<Vec<_>>(),
        vec!["cargo test", "cargo test", "cargo build"],
        "a run with both command forms contributes both, duplicates left to rendering"
    );

    // Only `handoffs` artifacts contribute, and only when they carry a path.
    assert_eq!(
        inputs.handoffs,
        vec![source(
            "alpha/Artifact/handoff",
            ".wai/projects/kaya/handoffs/1.md"
        )],
        "the designs artifact and the pathless handoff contribute nothing"
    );

    let dir = TempDir::new().expect("temp dir");
    let out = out_dir(&dir);
    fs::create_dir_all(&out).expect("output directory");

    // The bound property also requires that the ledger gains no entity.
    let ledger = dir.path().join(".kaya").join("ledger.jsonl");
    fs::create_dir_all(ledger.parent().expect("ledger directory")).expect("ledger directory");
    fs::write(&ledger, "{\"id\":\"ev-1\"}\n{\"id\":\"ev-2\"}\n").expect("seed the ledger");
    let ledger_before = fs::read(&ledger).expect("the ledger is readable");

    let built = build(&inputs, &out).expect("the selected references build");
    assert!(
        built.omitted.is_empty(),
        "both documents have references: {:?}",
        built.omitted
    );

    // Sorted by id, then by reference; the run's identical `cargo test` rows
    // collapse to one.
    let document = read(&reproduction_path(&dir));
    assert_eq!(
        body(&document, REPRODUCTION_BLOCK),
        "\
### Sources
- alpha/Source/one — src/main.rs@9f2c1a

### Commands
- alpha/Run/both — cargo build
- alpha/Run/both — cargo test
- alpha/Run/two — a
- alpha/Run/two — b
",
        "the reproduction block renders exactly the selected reference rows"
    );

    let handoff = read(&handoff_path(&dir));
    assert_eq!(
        body(&handoff, HANDOFF_BLOCK),
        "### Handoffs\n- alpha/Artifact/handoff — .wai/projects/kaya/handoffs/1.md\n",
        "the handoff block renders only the handoffs artifact"
    );

    assert!(
        !document.contains("alpha/Artifact/design") && !document.contains("alpha/Artifact/no-path"),
        "a designs or pathless artifact never reaches the reproduction package:\n{document}"
    );

    assert_eq!(
        built.summary.sources.get(REPRODUCTION_BLOCK),
        Some(&vec![
            "alpha/Run/both".to_owned(),
            "alpha/Run/two".to_owned(),
            "alpha/Source/one".to_owned(),
        ]),
        "the source map names every id backing the reproduction block"
    );

    assert_eq!(
        fs::read(&ledger).expect("the ledger is still readable"),
        ledger_before,
        "a reuse projection writes markdown only: the ledger gains no entity"
    );
}

#[test]
fn malformed_attributes_are_refused() {
    let cases = [
        (
            "alpha/Source/bad-path",
            EntityType::Source,
            json!({ "path": 7, "commit": "9f2c1a" }),
            "path",
        ),
        (
            "alpha/Source/bad-commit",
            EntityType::Source,
            json!({ "path": "src/main.rs", "commit": ["9f2c1a"] }),
            "commit",
        ),
        (
            "alpha/Run/bad-command",
            EntityType::Run,
            json!({ "command": ["cargo test"] }),
            "command",
        ),
        (
            "alpha/Run/bad-commands",
            EntityType::Run,
            json!({ "commands": ["cargo test", 7] }),
            "commands",
        ),
        (
            "alpha/Artifact/bad-path",
            EntityType::Artifact,
            json!({ "kind": "handoffs", "path": ["1.md"] }),
            "path",
        ),
    ];

    for (id, entity, attributes, attribute) in cases {
        let dir = TempDir::new().expect("temp dir");
        let out = out_dir(&dir);
        fs::create_dir_all(&out).expect("output directory");

        let error = inputs(&graph_with(id, entity, attributes))
            .expect_err("a non-string attribute is refused, never skipped");
        assert_eq!(
            LABEL_REUSE_ATTRIBUTE, LABEL_REUSE_ATTRIBUTE_LITERAL,
            "the refusal label is the documented reuse-attribute label"
        );
        assert_eq!(
            error.label, LABEL_REUSE_ATTRIBUTE,
            "a malformed {attribute} carries the reuse-attribute label"
        );
        assert!(
            error.message.contains(id),
            "the refusal names node {id}: {}",
            error.message
        );
        assert!(
            error.message.contains(attribute),
            "the refusal names attribute {attribute}: {}",
            error.message
        );
        assert_eq!(
            file_names(&out),
            Vec::<String>::new(),
            "a refused graph writes no document"
        );
    }
}

#[test]
fn blank_attributes_are_treated_as_absent() {
    let mut builder = GraphBuilder::new();
    node(
        &mut builder,
        "alpha/Source/blank-path",
        EntityType::Source,
        json!({ "path": "   ", "commit": "9f2c1a" }),
    );
    node(
        &mut builder,
        "alpha/Source/blank-commit",
        EntityType::Source,
        json!({ "path": "src/main.rs", "commit": "\t\r\n" }),
    );
    node(
        &mut builder,
        "alpha/Artifact/blank-handoff",
        EntityType::Artifact,
        json!({ "kind": "handoffs", "path": "  " }),
    );
    node(
        &mut builder,
        "alpha/Run/padded",
        EntityType::Run,
        json!({ "command": "  cargo test  " }),
    );
    let graph = builder.build().expect("the blank-attribute graph builds");

    let inputs = inputs(&graph).expect("blank attributes are absent, not malformed");
    assert!(
        inputs.revisions.is_empty(),
        "a blank path or commit contributes no revision: {:?}",
        inputs.revisions
    );
    assert!(
        inputs.handoffs.is_empty(),
        "a blank handoff path contributes no row: {:?}",
        inputs.handoffs
    );
    assert_eq!(
        inputs.commands,
        vec![source("alpha/Run/padded", "  cargo test  ")],
        "a padded command survives input selection and is trimmed at rendering"
    );

    let dir = TempDir::new().expect("temp dir");
    let out = out_dir(&dir);
    fs::create_dir_all(&out).expect("output directory");
    let built = build(&inputs, &out).expect("the surviving command builds");
    assert_eq!(
        built.omitted,
        vec![Omitted {
            family: "reuse-handoff",
            kind: "handoff",
            reason: REASON_NO_RECORDS,
        }],
        "the blank handoff path leaves the handoff kind empty"
    );
    assert!(
        !handoff_path(&dir).exists(),
        "an empty kind writes no document"
    );
    assert_eq!(
        body(&read(&reproduction_path(&dir)), REPRODUCTION_BLOCK),
        "### Commands\n- alpha/Run/padded — cargo test\n",
        "the rendered reference is trimmed"
    );

    let blank = TempDir::new().expect("temp dir");
    let blank_out = out_dir(&blank);
    fs::create_dir_all(&blank_out).expect("output directory");
    let blank_inputs = Inputs {
        revisions: vec![source("alpha/Source/one", " \n\t ")],
        ..Inputs::default()
    };
    let built = build(&blank_inputs, &blank_out).expect("an empty reference never renders");
    assert!(
        !reproduction_path(&blank).exists(),
        "a whitespace-only reference drops its row and writes no document"
    );
    assert_eq!(
        built.omitted,
        vec![
            Omitted {
                family: "reuse-handoff",
                kind: "reproduction-package",
                reason: REASON_NO_RECORDS,
            },
            Omitted {
                family: "reuse-handoff",
                kind: "handoff",
                reason: REASON_NO_RECORDS,
            },
        ],
        "a row without a reference leaves the kind empty"
    );
}

#[test]
fn multiline_references_render_on_one_line() {
    let reference =
        "cargo test\n<!-- kaya:begin generated:reuse-handoff.reproduction-package -->\rcargo build";
    let inputs = Inputs {
        commands: vec![source("alpha/Run/multi", reference)],
        ..Inputs::default()
    };

    let dir = TempDir::new().expect("temp dir");
    let out = out_dir(&dir);
    fs::create_dir_all(&out).expect("output directory");
    let built = build(&inputs, &out).expect("a multiline reference builds");
    assert_eq!(
        built.summary.recomputed,
        vec![REPRODUCTION_BLOCK.to_owned()],
        "only the command-bearing block is written"
    );

    let path = reproduction_path(&dir);
    let document = read(&path);
    let expected = "\
### Commands
- alpha/Run/multi — cargo test <!-- kaya:begin generated:reuse-handoff.reproduction-package --> cargo build
";
    assert_eq!(
        body(&document, REPRODUCTION_BLOCK),
        expected,
        "newlines and carriage returns collapse to single spaces"
    );
    assert_eq!(
        body(&document, REPRODUCTION_BLOCK).lines().count(),
        2,
        "the row occupies exactly one line"
    );

    let rebuilt = build(&inputs, &out).expect("the collapsed row still scans and rebuilds");
    assert!(
        rebuilt.summary.recomputed.is_empty(),
        "a second build recomputes nothing: {:?}",
        rebuilt.summary.recomputed
    );
    assert_eq!(
        rebuilt.summary.unchanged,
        vec![REPRODUCTION_BLOCK.to_owned()],
        "the written row matches the rendered row exactly"
    );
    assert_eq!(
        read(&path),
        document,
        "the document keeps its bytes when nothing changed"
    );
}

#[test]
fn identical_rows_are_deduplicated() {
    let inputs = Inputs {
        revisions: vec![
            source("alpha/Source/one", "src/main.rs@9f2c1a"),
            source("alpha/Source/one", "src/main.rs@9f2c1a"),
        ],
        commands: vec![
            source("alpha/Run/one", "cargo test"),
            source("alpha/Run/one", "cargo test"),
        ],
        ..Inputs::default()
    };

    let dir = TempDir::new().expect("temp dir");
    let out = out_dir(&dir);
    fs::create_dir_all(&out).expect("output directory");
    let built = build(&inputs, &out).expect("duplicate rows build");

    let document = read(&reproduction_path(&dir));
    assert_eq!(
        body(&document, REPRODUCTION_BLOCK),
        "\
### Sources
- alpha/Source/one — src/main.rs@9f2c1a

### Commands
- alpha/Run/one — cargo test
",
        "identical (id, reference) pairs render once"
    );
    assert_eq!(
        built.summary.sources.get(REPRODUCTION_BLOCK),
        Some(&vec![
            "alpha/Run/one".to_owned(),
            "alpha/Source/one".to_owned()
        ]),
        "the source map deduplicates the repeated ids"
    );
}

#[test]
fn removing_the_last_block_deletes_the_document() {
    let dir = TempDir::new().expect("temp dir");
    let out = out_dir(&dir);
    fs::create_dir_all(&out).expect("output directory");
    let path = reproduction_path(&dir);
    let seeded = "\
<!-- kaya:begin generated:reuse-handoff.reproduction-package -->
### Sources
- alpha/Source/one — src/main.rs@9f2c1a
<!-- kaya:end generated:reuse-handoff.reproduction-package -->
";
    fs::write(&path, seeded).expect("seed a block-only document");

    let built = build(&Inputs::default(), &out).expect("an empty projection removes the block");
    assert!(
        !path.exists(),
        "removing the last block deletes the now-empty document"
    );
    assert_eq!(
        file_names(&out),
        Vec::<String>::new(),
        "no zero-byte document is left behind"
    );
    assert_eq!(
        built.summary.recomputed,
        vec![REPRODUCTION_BLOCK.to_owned()],
        "the removed block is reported recomputed"
    );
    assert!(
        built.summary.unchanged.is_empty(),
        "the removed block is not reported unchanged: {:?}",
        built.summary.unchanged
    );
    assert_eq!(
        built.omitted,
        vec![
            Omitted {
                family: "reuse-handoff",
                kind: "reproduction-package",
                reason: REASON_NO_RECORDS,
            },
            Omitted {
                family: "reuse-handoff",
                kind: "handoff",
                reason: REASON_NO_RECORDS,
            },
        ],
        "both empty kinds are reported"
    );
}

#[test]
fn authored_and_foreign_bytes_survive_a_rebuild() {
    let seeded = "\
# Handoffs

Authored intro.

<!-- kaya:authored -->
Keep this sentence.
<!-- kaya:end authored -->

<!-- other-tool:begin -->
foreign bytes
<!-- other-tool:end -->
";
    let dir = TempDir::new().expect("temp dir");
    let out = out_dir(&dir);
    fs::create_dir_all(&out).expect("output directory");
    let path = handoff_path(&dir);
    fs::write(&path, seeded).expect("seed the authored document");

    let first = Inputs {
        handoffs: vec![source(
            "alpha/Artifact/handoff-1",
            ".wai/projects/kaya/handoffs/1.md",
        )],
        ..Inputs::default()
    };
    build(&first, &out).expect("the authored document builds");
    let document = read(&path);
    assert!(
        document.starts_with(seeded),
        "the build appends only its own block:\n{document}"
    );

    let second = Inputs {
        handoffs: vec![source(
            "alpha/Artifact/handoff-2",
            ".wai/projects/kaya/handoffs/2.md",
        )],
        ..Inputs::default()
    };
    build(&second, &out).expect("the authored document rebuilds");
    let rebuilt = read(&path);
    assert!(
        rebuilt.starts_with(seeded),
        "the rebuild preserves the authored and foreign bytes:\n{rebuilt}"
    );
    assert_eq!(
        body(&rebuilt, HANDOFF_BLOCK),
        "### Handoffs\n- alpha/Artifact/handoff-2 — .wai/projects/kaya/handoffs/2.md\n",
        "the rebuild replaces only the generated body"
    );
    assert_eq!(
        rebuilt.matches("Keep this sentence.").count(),
        1,
        "authored prose survives exactly once:\n{rebuilt}"
    );
    assert_eq!(
        rebuilt.matches("foreign bytes").count(),
        1,
        "foreign managed bytes survive exactly once:\n{rebuilt}"
    );
}

#[test]
fn second_build_reports_both_blocks_unchanged() {
    let inputs = Inputs {
        revisions: vec![source("alpha/Source/one", "src/main.rs@9f2c1a")],
        commands: vec![source("alpha/Run/one", "cargo test")],
        handoffs: vec![source(
            "alpha/Artifact/handoff",
            ".wai/projects/kaya/handoffs/1.md",
        )],
    };

    let dir = TempDir::new().expect("temp dir");
    let out = out_dir(&dir);
    fs::create_dir_all(&out).expect("output directory");

    let first = build(&inputs, &out).expect("the documents build");
    assert_eq!(
        first.summary.recomputed,
        vec![REPRODUCTION_BLOCK.to_owned(), HANDOFF_BLOCK.to_owned()],
        "the first build writes both blocks"
    );
    let reproduction_bytes = read(&reproduction_path(&dir));
    let handoff_bytes = read(&handoff_path(&dir));

    let second = build(&inputs, &out).expect("the documents rebuild");
    assert!(
        second.summary.recomputed.is_empty(),
        "the second build recomputes nothing: {:?}",
        second.summary.recomputed
    );
    assert_eq!(
        second.summary.unchanged,
        vec![REPRODUCTION_BLOCK.to_owned(), HANDOFF_BLOCK.to_owned()],
        "the second build reports both blocks unchanged"
    );
    assert_eq!(
        second.summary.sources, first.summary.sources,
        "the source map is preserved across rebuilds"
    );
    assert_eq!(
        second.summary.sources.get(REPRODUCTION_BLOCK),
        Some(&vec![
            "alpha/Run/one".to_owned(),
            "alpha/Source/one".to_owned()
        ]),
        "the reproduction source map names both backing ids"
    );
    assert_eq!(
        second.summary.sources.get(HANDOFF_BLOCK),
        Some(&vec!["alpha/Artifact/handoff".to_owned()]),
        "the handoff source map names its backing id"
    );
    assert_eq!(
        read(&reproduction_path(&dir)),
        reproduction_bytes,
        "an unchanged reproduction package keeps its bytes"
    );
    assert_eq!(
        read(&handoff_path(&dir)),
        handoff_bytes,
        "an unchanged handoff keeps its bytes"
    );
}

#[test]
fn handoff_renders_wai_artifact_references() {
    let dir = TempDir::new().expect("temp dir");
    let out = out_dir(&dir);
    fs::create_dir_all(&out).expect("output directory");

    let inputs = Inputs {
        handoffs: vec![
            source("artifact/wai-handoff-2", ".wai/projects/kaya/handoffs/2.md"),
            source("artifact/wai-handoff-1", ".wai/projects/kaya/handoffs/1.md"),
        ],
        ..Inputs::default()
    };

    let built = build(&inputs, &out).expect("a handoff document builds");
    assert_eq!(
        built.omitted,
        vec![Omitted {
            family: "reuse-handoff",
            kind: "reproduction-package",
            reason: REASON_NO_RECORDS,
        }],
        "the empty reproduction package is reported exactly once"
    );
    assert_eq!(
        built.summary.recomputed,
        vec![HANDOFF_BLOCK.to_owned()],
        "only the handoff block is recomputed"
    );
    assert_eq!(
        built.summary.sources.get(HANDOFF_BLOCK),
        Some(&vec![
            "artifact/wai-handoff-1".to_owned(),
            "artifact/wai-handoff-2".to_owned(),
        ]),
        "the source map names the handoff artifact ids in order"
    );

    assert!(
        !reproduction_path(&dir).exists(),
        "an omitted reproduction package writes no file"
    );
    assert_eq!(
        file_names(&out),
        vec![HANDOFF_FILE.to_owned()],
        "only the populated kind writes a document"
    );

    let document = read(&handoff_path(&dir));
    assert_block(
        &document,
        HANDOFF_BLOCK,
        &[
            "### Handoffs",
            "- artifact/wai-handoff-1 — .wai/projects/kaya/handoffs/1.md",
            "- artifact/wai-handoff-2 — .wai/projects/kaya/handoffs/2.md",
        ],
    );

    for (id, reference) in [
        ("artifact/wai-handoff-1", ".wai/projects/kaya/handoffs/1.md"),
        ("artifact/wai-handoff-2", ".wai/projects/kaya/handoffs/2.md"),
    ] {
        let row = format!("- {id} — {reference}");
        assert!(
            document.contains(&row),
            "every row carries its artifact id and reference: {row}\n{document}"
        );
    }
}

#[test]
fn empty_kinds_are_omitted_and_write_no_file() {
    let dir = TempDir::new().expect("temp dir");
    let out = out_dir(&dir);
    fs::create_dir_all(&out).expect("output directory");
    let untouched = out.join("notes.md");
    fs::write(&untouched, "# Notes\n").expect("seed an unrelated document");

    let built =
        build(&Inputs::default(), &out).expect("an empty projection builds without documents");

    assert_eq!(
        built.omitted,
        vec![
            Omitted {
                family: "reuse-handoff",
                kind: "reproduction-package",
                reason: REASON_NO_RECORDS,
            },
            Omitted {
                family: "reuse-handoff",
                kind: "handoff",
                reason: REASON_NO_RECORDS,
            },
        ],
        "each empty kind is reported exactly once, naming family, kind, and reason"
    );
    assert!(
        built.summary.recomputed.is_empty(),
        "no block is written: {:?}",
        built.summary.recomputed
    );
    assert!(
        built.summary.unchanged.is_empty(),
        "no block is considered: {:?}",
        built.summary.unchanged
    );
    assert!(
        built.summary.sources.is_empty(),
        "an omitted kind has no source-map entry: {:?}",
        built.summary.sources
    );

    assert!(
        !reproduction_path(&dir).exists(),
        "an empty reproduction package writes no file"
    );
    assert!(
        !handoff_path(&dir).exists(),
        "an empty handoff writes no file"
    );
    assert_eq!(
        file_names(&out),
        vec!["notes.md".to_owned()],
        "no projection file is created, not even an empty or partial one"
    );
    assert_eq!(
        read(&untouched),
        "# Notes\n",
        "building leaves an unrelated document byte-identical"
    );
}

#[test]
fn reuse_block_names_are_vocabulary_names() {
    assert_eq!(
        Kind::ReproductionPackage.block_name(),
        REPRODUCTION_BLOCK,
        "the reproduction package block name is its family.kind vocabulary name"
    );
    assert_eq!(
        Kind::Handoff.block_name(),
        HANDOFF_BLOCK,
        "the handoff block name is its family.kind vocabulary name"
    );
    assert_eq!(REPRODUCTION_BLOCK, "reuse-handoff.reproduction-package");
    assert_eq!(HANDOFF_BLOCK, "reuse-handoff.handoff");
}

#[test]
fn building_writes_markdown_only() {
    let dir = TempDir::new().expect("temp dir");
    let out = out_dir(&dir);
    fs::create_dir_all(&out).expect("output directory");

    let ledger = dir.path().join(".kaya").join("ledger.jsonl");
    fs::create_dir_all(ledger.parent().expect("ledger directory")).expect("ledger directory");
    fs::write(&ledger, LEDGER_SEED).expect("seed the ledger");
    let ledger_before = fs::read(&ledger).expect("ledger is readable");
    let events_before = String::from_utf8(ledger_before.clone())
        .expect("ledger seed is UTF-8")
        .lines()
        .count();
    assert_eq!(events_before, 2, "the seed holds two events");

    let inputs = Inputs {
        revisions: vec![source("src/repo/commit", "docs/research/spec.md@9f2c1a")],
        handoffs: vec![source(
            "artifact/wai-handoff-1",
            ".wai/projects/kaya/handoffs/1.md",
        )],
        ..Inputs::default()
    };
    let built = build(&inputs, &out).expect("a reuse projection builds");
    assert!(
        built.omitted.is_empty(),
        "both kinds have references: {:?}",
        built.omitted
    );

    assert_eq!(
        file_names(&out),
        vec![HANDOFF_FILE.to_owned(), REPRODUCTION_FILE.to_owned()],
        "the build writes exactly the two markdown documents"
    );
    for name in file_names(&out) {
        assert!(
            name.ends_with(".md"),
            "the build writes markdown only, not {name}"
        );
    }

    let ledger_after = fs::read(&ledger).expect("ledger still exists");
    assert_eq!(
        ledger_after, ledger_before,
        "building documents leaves every ledger byte identical"
    );
    assert_eq!(
        String::from_utf8(ledger_after)
            .expect("ledger is UTF-8")
            .lines()
            .count(),
        events_before,
        "building documents appends no event"
    );
}

#[test]
fn legacy_generated_block_in_a_reuse_document_is_refused() {
    let dir = TempDir::new().expect("temp dir");
    let out = out_dir(&dir);
    fs::create_dir_all(&out).expect("output directory");

    let path = reproduction_path(&dir);
    let document = "\
# Reproduction package

<!-- kaya:begin generated:experiment-report -->
## Runs

- p/Run/r1
<!-- kaya:end generated:experiment-report -->
";
    fs::write(&path, document).expect("seed a legacy generated block");

    let inputs = Inputs {
        revisions: vec![source("src/repo/commit", "docs/research/spec.md@9f2c1a")],
        handoffs: vec![source(
            "artifact/wai-handoff-1",
            ".wai/projects/kaya/handoffs/1.md",
        )],
        ..Inputs::default()
    };

    let error = build(&inputs, &out).expect_err("a block name outside the vocabulary is refused");
    assert_eq!(
        error.label, LABEL_LEGACY_BLOCK,
        "the refusal carries the legacy-block label"
    );
    assert!(
        error.message.contains("experiment-report"),
        "the refusal names the offending block: {}",
        error.message
    );
    assert_eq!(
        read(&path),
        document,
        "a refused build writes no byte to the existing document"
    );
    assert!(
        !handoff_path(&dir).exists(),
        "every existing document is validated before any document is written"
    );
}
