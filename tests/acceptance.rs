//! MVP-1 acceptance: the three-question dossier on kaya's own record.
//!
//! The test copies kaya's committed wai artifact tree and the checked-in
//! dossier into a temporary workspace, commits them, seeds the research model
//! with three events, then runs the built binary the way a user would:
//! `kaya import wai`, `kaya import git --limit 5`, `kaya build`, a second
//! `kaya build`, and `kaya status`.
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

use kaya::event::parse_line;
use kaya::ledger::{AppendOutcome, Ledger};

const EXPERIMENT: &str = "kaya/Experiment/add-core-research-record";
const QUESTION: &str = "kaya/Question/mvp-1-acceptance";
const LEDGER: &str = ".kaya/ledger.jsonl";
const INDEX: &str = "docs/research/index.md";
const REPORT: &str = "docs/research/experiments/add-core-research-record.md";

const AUTHORED_BEGIN: &str = "<!-- kaya:authored -->";
const AUTHORED_END: &str = "<!-- kaya:end authored -->";
const FOREIGN_BEGIN: &str = "<!-- other-tool:begin -->";
const FOREIGN_END: &str = "<!-- other-tool:end -->";
const INDEX_BLOCK_BEGIN: &str = "<!-- kaya:begin generated:research-index -->";
const INDEX_BLOCK_END: &str = "<!-- kaya:end generated:research-index -->";
const REPORT_BLOCK_BEGIN: &str = "<!-- kaya:begin generated:experiment-report -->";
const REPORT_BLOCK_END: &str = "<!-- kaya:end generated:experiment-report -->";

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

/// Remove kaya generated blocks, keeping authored and foreign bytes intact.
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

fn seed_event(
    id: &str,
    event_type: &str,
    subject_type: &str,
    subject_id: &str,
    data: Value,
) -> Value {
    json!({
        "id": id,
        "schema": "kaya.event/v0",
        "type": event_type,
        "timestamp": "2026-11-16T00:00:00Z",
        "actor": {"type": "human", "name": "acceptance-test"},
        "subject": {"type": subject_type, "id": subject_id},
        "data": data,
        "provenance": {
            "tool": "acceptance",
            "version": "0.1",
            "reference": "tests/acceptance.rs"
        },
        "artifacts": []
    })
}

/// Admit the Experiment, the Question, and the investigating edge.
fn seed_model(workspace: &Path) {
    let mut ledger = Ledger::open(workspace.join(LEDGER)).expect("kaya init created the ledger");
    let records = [
        seed_event(
            "acceptance-experiment",
            "model.entity.created",
            "Experiment",
            EXPERIMENT,
            json!({"attributes": {"title": "add-core-research-record"}}),
        ),
        seed_event(
            "acceptance-question",
            "model.entity.created",
            "Question",
            QUESTION,
            json!({"attributes": {"title": "MVP-1 acceptance: three-question dossier"}}),
        ),
        seed_event(
            "acceptance-investigates",
            "model.relationship.asserted",
            "Experiment",
            EXPERIMENT,
            json!({"relationship": "investigates", "from": QUESTION, "to": EXPERIMENT}),
        ),
    ];
    for record in records {
        let line = serde_json::to_string(&record).expect("seed event serializes");
        let event = parse_line(&line).expect("seed line is a valid kaya.event/v0 record");
        assert_eq!(
            ledger.append(&event).expect("seed event appends"),
            AppendOutcome::Appended
        );
    }
}

#[test]
fn mvp1_three_question_dossier_compiles_from_the_repository_record() {
    let workspace = TempDir::new().expect("temp workspace");
    let root = workspace.path();
    let manifest = manifest_dir();

    // The report the build must compile names its experiment by construction:
    // `docs/research/experiments/<key>.md` for `kaya/Experiment/<key>`.
    let experiment_key = EXPERIMENT
        .rsplit('/')
        .next()
        .expect("experiment id has a key");
    let expected_report = format!("docs/research/experiments/{experiment_key}.md");
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
    copy_checked_in(&manifest.join(REPORT), &report_path);

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
    seed_model(root);

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
        !report_source.contains(REPORT_BLOCK_BEGIN),
        "the checked-in report carries no generated block; the first build appends it"
    );

    let build = run_kaya(root, &["build"]);
    assert_eq!(build["envelope_kind"], json!("build"));
    let recomputed = names(&build, "recomputed");
    assert!(
        lists(&recomputed, "research-index"),
        "first build compiles the index: {build}"
    );
    assert!(
        lists(&recomputed, "experiment-report"),
        "first build compiles the experiment report: {build}"
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
    assert!(
        index_after.contains(EXPERIMENT),
        "index lists the experiment"
    );
    assert!(index_after.contains(QUESTION), "index lists the question");
    assert!(
        index_after.contains(foreign_block),
        "the foreign managed block survives the rebuild byte for byte"
    );

    // The wai adapter walks direct children of each kind directory.
    let mut designs: Vec<PathBuf> = fs::read_dir(artifacts.join("designs"))
        .expect("copied design directory")
        .map(|entry| entry.expect("design entry").path())
        .filter(|path| path.is_file())
        .collect();
    designs.sort();
    assert!(!designs.is_empty(), "the wai design tree was copied");
    for design in &designs {
        let relative = design
            .strip_prefix(root)
            .expect("design artifact path is inside the workspace");
        let rendered = relative.to_string_lossy().replace('\\', "/");
        assert!(
            index_after.contains(&rendered),
            "index lists design artifact {rendered}"
        );
    }

    // The report: generated block plus the preserved authored dossier.
    let report_after_bytes = fs::read(&report_path).expect("build wrote the report");
    assert!(
        report_after_bytes.starts_with(&report_before),
        "build appends to the report without rewriting the authored answers"
    );
    let report_after = String::from_utf8(report_after_bytes.clone()).expect("report is UTF-8");
    assert!(report_after.contains(REPORT_BLOCK_BEGIN), "{report_after}");
    assert!(report_after.contains(REPORT_BLOCK_END), "{report_after}");
    let generated = between(&report_after, REPORT_BLOCK_BEGIN, REPORT_BLOCK_END);
    assert!(generated.contains("## Question"), "{generated}");
    assert!(
        generated.contains(QUESTION),
        "the generated report names the investigated question: {generated}"
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
    let rebuild = run_kaya(root, &["build"]);
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
        lists(&unchanged, "experiment-report"),
        "second build reports the report unchanged: {rebuild}"
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
