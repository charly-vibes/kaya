//! MVP-1 acceptance: the three-question dossier on kaya's own record.
//!
//! The first test copies kaya's committed wai artifact tree, model seed, and
//! checked-in dossier into a temporary workspace, commits them, imports the
//! seed through `kaya import model`, then runs the built binary the way a user
//! would: `kaya import wai`, `kaya import git --limit 5`, `kaya build
//! --allow-uncommitted`, a second build, and `kaya status`. The second test
//! commits the ledger before building and omits the flag.
//!
//! It gates two things: the three acceptance questions are answerable from the
//! generated dossier with provenance, and compilation appends/rewrites only
//! its own generated blocks — the authored answers and a foreign tool's
//! managed block survive byte for byte.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use assert_cmd::Command as AssertCommand;
use serde_json::{Value, json};
use tempfile::TempDir;

const EXPERIMENT: &str = "kaya/Experiment/add-core-research-record";
const QUESTION: &str = "kaya/Question/mvp-1-acceptance";
const HYPOTHESIS: &str = "kaya/Hypothesis/H-001";
const LEDGER: &str = ".kaya/ledger.jsonl";
const SEED: &str = ".kaya/seeds/add-core-research-record.jsonl";
const INDEX: &str = "docs/research/index.md";
const REPORT: &str = "docs/research/experiments/kaya-Experiment-add-core-research-record.md";
const CHECKED_IN_REPORT: &str =
    "docs/research/experiments/kaya-Experiment-add-core-research-record.md";
const SPECIFICATION_NAME: &str =
    "exploration.experiment-specification:kaya/Experiment/add-core-research-record";
const RECORD_NAME: &str = "exploration.experiment-record:kaya/Experiment/add-core-research-record";
const SEED_EVENTS: u64 = 13;
const SOURCE_COMMIT: &str = "43b606f";

const AUTHORED_BEGIN: &str = "<!-- kaya:authored -->";
const AUTHORED_END: &str = "<!-- kaya:end authored -->";
const FOREIGN_BEGIN: &str = "<!-- other-tool:begin -->";
const FOREIGN_END: &str = "<!-- other-tool:end -->";
const INDEX_BLOCK_BEGIN: &str = "<!-- kaya:begin generated:research-index -->";
const INDEX_BLOCK_END: &str = "<!-- kaya:end generated:research-index -->";
const SPECIFICATION_BLOCK_BEGIN: &str =
    "<!-- kaya:begin generated:exploration.experiment-specification -->";
const SPECIFICATION_BLOCK_END: &str =
    "<!-- kaya:end generated:exploration.experiment-specification -->";
const RECORD_BLOCK_BEGIN: &str = "<!-- kaya:begin generated:exploration.experiment-record -->";
const RECORD_BLOCK_END: &str = "<!-- kaya:end generated:exploration.experiment-record -->";

const QUESTION_HEADINGS: [&str; 3] = [
    "### Q1: Why are events not genesis envelopes?",
    "### Q2: What evidence supports \"adapters translate, never interpret\"?",
    "### Q3: What is deliberately unspecified (MVP-2+ seams)?",
];

/// Sources each answer names, so the questions are answerable with provenance.
const PROVENANCE_CITATIONS: [&str; 6] = [
    "openspec/changes/add-core-research-record/proposal.md",
    "openspec/changes/add-core-research-record/specs/kaya-event/layer.md",
    "openspec/changes/add-core-research-record/specs/kaya-adapters/spec.md",
    ".wai/projects/kaya/designs/2026-11-16-post-bootstrap-alignment-consumers-mvp-1-accep.md",
    "docs/src/index.md",
    "f39a1fd",
];

/// Repo files pinned as Source entities in the committed model seed.
const SOURCE_PATHS: [&str; 6] = [
    "openspec/changes/add-core-research-record/proposal.md",
    "openspec/changes/add-core-research-record/specs/kaya-event/layer.md",
    "openspec/changes/add-core-research-record/specs/kaya-adapters/spec.md",
    ".wai/projects/kaya/designs/2026-11-16-post-bootstrap-alignment-consumers-mvp-1-accep.md",
    "docs/src/index.md",
    "openspec/changes/add-core-research-record/specs/kaya-promotion/spec.md",
];

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Run the built binary in `workspace`, requiring exit 0 and an ok envelope.
fn run_kaya(workspace: &Path, args: &[&str]) -> Value {
    let mut command = AssertCommand::cargo_bin("kaya").expect("kaya binary is built");
    command.current_dir(workspace).args(args);
    let output = command.output().expect("kaya runs");
    assert!(
        output.status.success(),
        "kaya {args:?} exited with {:?}: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap_or_else(|err| {
        panic!(
            "kaya {args:?} did not print one JSON envelope ({err}): {}",
            String::from_utf8_lossy(&output.stdout)
        )
    });
    assert_eq!(
        envelope["ok"],
        json!(true),
        "kaya {args:?} returned an error envelope: {envelope}"
    );
    envelope
}

fn names(envelope: &Value, field: &str) -> Vec<String> {
    envelope["data"][field]
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn lists(names: &[String], expected: &str) -> bool {
    names.iter().any(|name| name == expected)
}

/// The exact bytes from `begin` through `end` in `document`.
fn between<'a>(document: &'a str, begin: &str, end: &str) -> &'a str {
    let start = document
        .find(begin)
        .unwrap_or_else(|| panic!("document lacks begin marker {begin}"));
    let end_offset = document[start..]
        .find(end)
        .unwrap_or_else(|| panic!("document lacks end marker {end}"));
    &document[start..start + end_offset + end.len()]
}

fn copy_markdown_tree(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).expect("create artifact directory");
    for entry in fs::read_dir(source).expect("read artifact directory") {
        let entry = entry.expect("artifact directory entry");
        let path = entry.path();
        let target = destination.join(entry.file_name());
        if path.is_dir() {
            copy_markdown_tree(&path, &target);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("md") {
            fs::copy(&path, &target).expect("copy markdown artifact");
        }
    }
}

fn copy_checked_in(source: &Path, destination: &Path) {
    let parent = destination.parent().expect("destination has a parent");
    fs::create_dir_all(parent).expect("create dossier directory");
    // The checked-in dossier already carries generated blocks from earlier
    // dogfood builds. The acceptance run proves the first-build path, so the
    // copy starts from the authored/foreign bytes only.
    let text = fs::read_to_string(source).expect("read checked-in dossier document");
    fs::write(destination, strip_generated_blocks(&text))
        .expect("write pre-build dossier document");
}

/// Copy the committed model seed into the workspace so the binary imports it.
fn copy_seed(workspace: &Path, manifest: &Path) -> PathBuf {
    let seed = workspace.join(SEED);
    let parent = seed.parent().expect("seed path has a parent");
    fs::create_dir_all(parent).expect("create seed directory");
    fs::copy(manifest.join(SEED), &seed).expect("copy committed model seed");
    seed
}

/// Remove kaya generated blocks, keeping authored and foreign bytes intact.
///
/// Every generated block is named `<family>.<kind>`, so the dossier's
/// `exploration.experiment-specification` and `exploration.experiment-record`
/// blocks both fall under the same `generated:` marker rule.
fn strip_generated_blocks(document: &str) -> String {
    let mut kept = String::new();
    let mut inside = false;
    for line in document.split_inclusive('\n') {
        let trimmed = line.trim();
        if trimmed.starts_with("<!-- kaya:begin generated:") {
            inside = true;
            continue;
        }
        if inside {
            if trimmed.starts_with("<!-- kaya:end generated:") {
                inside = false;
            }
            continue;
        }
        kept.push_str(line);
    }
    kept
}

/// One deterministic commit: fixed identity, fixed dates, no user config.
fn git(workspace: &Path, args: &[&str]) {
    let output = Command::new("git")
        .arg("-C")
        .arg(workspace)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .env("GIT_AUTHOR_NAME", "kaya acceptance")
        .env("GIT_AUTHOR_EMAIL", "acceptance@example.invalid")
        .env("GIT_COMMITTER_NAME", "kaya acceptance")
        .env("GIT_COMMITTER_EMAIL", "acceptance@example.invalid")
        .env("GIT_AUTHOR_DATE", "2026-11-16T00:00:00+00:00")
        .env("GIT_COMMITTER_DATE", "2026-11-16T00:00:00+00:00")
        .args(args)
        .output()
        .expect("git runs");
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn mvp1_three_question_dossier_compiles_from_the_repository_record() {
    let workspace = TempDir::new().expect("temp workspace");
    let root = workspace.path();
    let manifest = manifest_dir();

    // The report the build must compile names its experiment by construction:
    // `docs/research/experiments/<namespace-qualified id with / as ->.md`.
    let expected_report = format!(
        "docs/research/experiments/{}.md",
        EXPERIMENT.replace('/', "-")
    );
    assert_eq!(REPORT, expected_report.as_str());

    let artifacts = root.join(".wai/projects/kaya");
    for kind in ["designs", "research"] {
        copy_markdown_tree(
            &manifest.join(".wai/projects/kaya").join(kind),
            &artifacts.join(kind),
        );
    }

    let index_path = root.join(INDEX);
    let report_path = root.join(REPORT);
    copy_checked_in(&manifest.join(INDEX), &index_path);
    copy_checked_in(&manifest.join(CHECKED_IN_REPORT), &report_path);
    copy_seed(root, &manifest);

    git(root, &["init", "--quiet"]);
    git(root, &["add", "--all"]);
    git(
        root,
        &[
            "commit",
            "--quiet",
            "--message",
            "seed the kaya acceptance workspace",
        ],
    );

    let init = run_kaya(root, &["init"]);
    assert_eq!(init["envelope_kind"], json!("init"));

    let model = run_kaya(root, &["import", "model", "--file", SEED]);
    assert_eq!(model["data"]["source"], json!("model"));
    assert_eq!(
        model["data"]["events"],
        json!(SEED_EVENTS),
        "the committed seed carries every seeded model event: {model}"
    );
    assert_eq!(model["data"]["appended"], json!(SEED_EVENTS));
    assert_eq!(model["data"]["duplicates"], json!(0));

    let wai = run_kaya(root, &["import", "wai"]);
    assert_eq!(wai["data"]["source"], json!("wai"));
    assert!(
        wai["data"]["events"].as_u64().unwrap_or(0) > 0,
        "wai import captured at least one artifact: {wai}"
    );

    let imported_git = run_kaya(root, &["import", "git", "--limit", "5"]);
    assert_eq!(imported_git["data"]["source"], json!("git"));
    assert!(
        imported_git["data"]["events"].as_u64().unwrap_or(0) > 0,
        "git import captured the seeded commit: {imported_git}"
    );

    // Byte baselines: what the checked-in documents contain before kaya runs.
    let index_before = fs::read(&index_path).expect("copied index exists");
    let report_before = fs::read(&report_path).expect("copied report exists");
    let index_source = String::from_utf8(index_before.clone()).expect("index is UTF-8");
    let report_source = String::from_utf8(report_before.clone()).expect("report is UTF-8");
    let authored_answers = between(&report_source, AUTHORED_BEGIN, AUTHORED_END);
    let foreign_block = between(&index_source, FOREIGN_BEGIN, FOREIGN_END);
    assert!(
        !index_source.contains(INDEX_BLOCK_BEGIN),
        "the checked-in index carries no generated block; the first build appends it"
    );
    assert!(
        !report_source.contains(SPECIFICATION_BLOCK_BEGIN)
            && !report_source.contains(SPECIFICATION_BLOCK_END)
            && !report_source.contains(RECORD_BLOCK_BEGIN)
            && !report_source.contains(RECORD_BLOCK_END),
        "the checked-in report carries no generated block; the first build appends it"
    );

    let build = run_kaya(root, &["build", "--allow-uncommitted"]);
    assert_eq!(build["envelope_kind"], json!("build"));
    let recomputed = names(&build, "recomputed");
    assert!(
        lists(&recomputed, "research-index"),
        "first build compiles the index: {build}"
    );
    assert!(
        lists(&recomputed, SPECIFICATION_NAME),
        "first build compiles the experiment specification: {build}"
    );
    assert!(
        !lists(&recomputed, RECORD_NAME),
        "the dossier has no runs, so no record block is written: {build}"
    );
    assert_eq!(
        build["data"]["omitted"],
        json!([{
            "family": "exploration",
            "kind": "experiment-record",
            "reason": "no-records",
        }]),
        "the empty record kind is reported once with its family and reason: {build}"
    );

    // The index: generated block appended after the authored and foreign bytes.
    let index_after_bytes = fs::read(&index_path).expect("build wrote the index");
    assert!(
        index_after_bytes.starts_with(&index_before),
        "build appends to the index without rewriting authored or foreign bytes"
    );
    let index_after = String::from_utf8(index_after_bytes.clone()).expect("index is UTF-8");
    assert!(index_after.contains(INDEX_BLOCK_BEGIN), "{index_after}");
    assert!(index_after.contains(INDEX_BLOCK_END), "{index_after}");
    let generated_index = between(&index_after, INDEX_BLOCK_BEGIN, INDEX_BLOCK_END);
    assert!(
        generated_index.contains(EXPERIMENT),
        "the generated index lists the experiment"
    );
    assert!(
        generated_index.contains(QUESTION),
        "the generated index lists the question"
    );
    assert!(
        generated_index.contains(HYPOTHESIS),
        "the generated index lists the hypothesis"
    );
    assert!(
        generated_index.contains("### Sources\n"),
        "the generated index renders Sources: {generated_index}"
    );
    for path in SOURCE_PATHS {
        let pinned = format!("{path}@{SOURCE_COMMIT}");
        assert!(
            generated_index.contains(&pinned),
            "the generated index pins source {pinned}: {generated_index}"
        );
    }
    assert!(
        index_after.contains(foreign_block),
        "the foreign managed block survives the rebuild byte for byte"
    );

    // The wai adapter walks direct children of each kind directory, and the
    // generated index must name every artifact it captured.
    let mut wai_artifacts: Vec<PathBuf> = Vec::new();
    for kind in ["designs", "research"] {
        let mut captured: Vec<PathBuf> = fs::read_dir(artifacts.join(kind))
            .expect("copied artifact directory")
            .map(|entry| entry.expect("artifact entry").path())
            .filter(|path| path.is_file())
            .collect();
        captured.sort();
        wai_artifacts.extend(captured);
    }
    assert!(
        !wai_artifacts.is_empty(),
        "the wai artifact tree was copied"
    );
    for artifact in &wai_artifacts {
        let relative = artifact
            .strip_prefix(root)
            .expect("artifact path is inside the workspace");
        let rendered = relative.to_string_lossy().replace('\\', "/");
        assert!(
            generated_index.contains(&rendered),
            "the generated index lists wai artifact {rendered}"
        );
    }

    // The report: generated block plus the preserved authored dossier.
    let report_after_bytes = fs::read(&report_path).expect("build wrote the report");
    assert!(
        report_after_bytes.starts_with(&report_before),
        "build appends to the report without rewriting the authored answers"
    );
    let report_after = String::from_utf8(report_after_bytes.clone()).expect("report is UTF-8");
    assert!(
        report_after.contains(SPECIFICATION_BLOCK_BEGIN),
        "{report_after}"
    );
    assert!(
        report_after.contains(SPECIFICATION_BLOCK_END),
        "{report_after}"
    );
    assert!(
        !report_after.contains(RECORD_BLOCK_BEGIN) && !report_after.contains(RECORD_BLOCK_END),
        "the experiment has no runs, so the record block is omitted: {report_after}"
    );
    let generated = between(
        &report_after,
        SPECIFICATION_BLOCK_BEGIN,
        SPECIFICATION_BLOCK_END,
    );
    for heading in ["## Question", "## Hypothesis"] {
        assert!(
            generated.contains(heading),
            "the generated specification renders {heading}: {generated}"
        );
    }
    for absent in [
        "## Runs",
        "## Observations",
        "## Findings",
        "## Interpretations",
        "- _(none)_",
    ] {
        assert!(
            !generated.contains(absent),
            "an empty kind leaves no section or placeholder row: {absent}"
        );
    }
    assert!(
        generated.contains(QUESTION),
        "the generated specification names the investigated question: {generated}"
    );
    assert!(
        generated.contains(HYPOTHESIS),
        "the generated specification names the predicted hypothesis: {generated}"
    );
    assert!(
        report_after.contains(EXPERIMENT),
        "the report names the experiment it compiles"
    );
    assert!(
        report_after.contains(authored_answers),
        "the authored answers survive the rebuild byte for byte"
    );
    for heading in QUESTION_HEADINGS {
        assert!(
            report_after.contains(heading),
            "the dossier answers {heading}"
        );
    }
    for citation in PROVENANCE_CITATIONS {
        assert!(
            report_after.contains(citation),
            "the dossier names provenance source {citation}"
        );
    }

    // A second build must be a no-op on both projections.
    let rebuild = run_kaya(root, &["build", "--allow-uncommitted"]);
    assert!(
        names(&rebuild, "recomputed").is_empty(),
        "second build recomputes nothing: {rebuild}"
    );
    let unchanged = names(&rebuild, "unchanged");
    assert!(
        lists(&unchanged, "research-index"),
        "second build reports the index unchanged: {rebuild}"
    );
    assert!(
        lists(&unchanged, SPECIFICATION_NAME),
        "second build reports the specification unchanged: {rebuild}"
    );
    assert!(
        !lists(&unchanged, RECORD_NAME),
        "an omitted kind is never reported unchanged: {rebuild}"
    );
    assert_eq!(
        fs::read(&index_path).expect("index still exists"),
        index_after_bytes,
        "unchanged index keeps its bytes"
    );
    assert_eq!(
        fs::read(&report_path).expect("report still exists"),
        report_after_bytes,
        "unchanged report keeps its bytes"
    );

    // Status surfaces the pending promotion budget without prompting.
    let status = run_kaya(root, &["status"]);
    let pending = status["data"]["pending_proposals"]
        .as_u64()
        .unwrap_or_else(|| panic!("status reports pending_proposals: {status}"));
    assert_eq!(
        pending, 0,
        "no promotion events were seeded, so nothing is pending"
    );
}

#[test]
fn committed_ledger_builds_without_the_uncommitted_flag() {
    let workspace = TempDir::new().expect("temp workspace");
    let root = workspace.path();
    let manifest = manifest_dir();

    git(root, &["init", "--quiet"]);
    let init = run_kaya(root, &["init"]);
    assert_eq!(init["envelope_kind"], json!("init"));
    copy_seed(root, &manifest);

    let model = run_kaya(root, &["import", "model", "--file", SEED]);
    assert_eq!(model["data"]["appended"], json!(SEED_EVENTS));
    assert!(
        root.join(LEDGER).is_file(),
        "the ledger records the imported seed"
    );

    git(root, &["add", LEDGER, SEED]);
    git(
        root,
        &[
            "commit",
            "--quiet",
            "--message",
            "record the model seed before building",
        ],
    );

    // No flag: the ledger is committed, so git can verify the built bytes.
    let build = run_kaya(root, &["build"]);
    assert_eq!(build["envelope_kind"], json!("build"));
    let recomputed = names(&build, "recomputed");
    assert!(
        lists(&recomputed, "research-index"),
        "committed build compiles the index: {build}"
    );
    assert!(
        lists(&recomputed, SPECIFICATION_NAME),
        "committed build compiles the experiment specification: {build}"
    );

    let report = fs::read_to_string(root.join(REPORT)).expect("build wrote the report");
    let generated = between(&report, SPECIFICATION_BLOCK_BEGIN, SPECIFICATION_BLOCK_END);
    assert!(
        !report.contains(RECORD_BLOCK_BEGIN) && !report.contains(RECORD_BLOCK_END),
        "the seeded experiment has no runs, so the record block is omitted: {report}"
    );
    assert!(
        generated.contains(QUESTION) && generated.contains(HYPOTHESIS),
        "the generated report cites its question and hypothesis: {generated}"
    );

    let index = fs::read_to_string(root.join(INDEX)).expect("build wrote the index");
    let generated_index = between(&index, INDEX_BLOCK_BEGIN, INDEX_BLOCK_END);
    for path in SOURCE_PATHS {
        let pinned = format!("{path}@{SOURCE_COMMIT}");
        assert!(
            generated_index.contains(&pinned),
            "the generated index pins source {pinned}: {generated_index}"
        );
    }
}
