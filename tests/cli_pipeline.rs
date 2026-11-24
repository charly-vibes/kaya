//! Integration tests for the CLI pipeline: `import`, `build`, `publish`,
//! `promote`, `challenge`, and the `status` proposal counter.
//!
//! Every case drives the real binary in a tempfile project, writes its own
//! fixture tree, and reads the one JSON envelope each invocation prints.

use assert_cmd::Command;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fs;
use std::process::Command as StdCommand;
use tempfile::TempDir;

use kaya::projection::report::BRIEF_BLOCK;
use kaya::publication::admission;

const LEDGER: &str = ".kaya/ledger.jsonl";
const INDEX: &str = "docs/research/index.md";

fn kaya(dir: &TempDir) -> Command {
    let mut command = Command::cargo_bin("kaya").expect("kaya binary is built");
    command.current_dir(dir.path());
    command
}

fn run(dir: &TempDir, args: &[&str]) -> std::process::Output {
    kaya(dir).args(args).output().expect("kaya runs")
}

fn import_wai(dir: &TempDir) -> std::process::Output {
    run(dir, &["import", "wai"])
}

fn promote(dir: &TempDir, assertion_id: &str, actor: &str) -> std::process::Output {
    run(dir, &["promote", assertion_id, "--actor", actor])
}

fn challenge(
    dir: &TempDir,
    conclusion_id: &str,
    events: &[&str],
    actor: &str,
) -> std::process::Output {
    let mut args = vec!["challenge", conclusion_id, "--events"];
    args.extend(events.iter().copied());
    args.push("--actor");
    args.push(actor);
    run(dir, &args)
}

fn parse_envelope(output: &std::process::Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|err| {
        panic!(
            "expected one JSON envelope on stdout ({err}); stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

#[track_caller]
fn assert_success(output: &std::process::Output) -> Value {
    assert!(
        output.status.success(),
        "expected success (exit {:?}); stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    let envelope = parse_envelope(output);
    assert_eq!(envelope["ok"], json!(true));
    assert!(
        !envelope
            .as_object()
            .expect("envelope is a JSON object")
            .contains_key("error"),
        "success envelopes omit the error field"
    );
    envelope
}

#[track_caller]
fn assert_failure(output: &std::process::Output) -> Value {
    assert_eq!(
        output.status.code(),
        Some(1),
        "expected exit 1; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let envelope = parse_envelope(output);
    assert_eq!(envelope["ok"], json!(false));
    assert_eq!(envelope["envelope_kind"], json!("error"));
    assert_eq!(envelope["data"], Value::Null);
    envelope
}

fn event_line(event: &Value) -> String {
    format!(
        "{}\n",
        serde_json::to_string(event).expect("event serializes")
    )
}

fn init(dir: &TempDir) {
    assert_success(&run(dir, &["init"]));
}

fn write_wai_design(dir: &TempDir, name: &str) {
    let path = dir
        .path()
        .join(".wai")
        .join("projects")
        .join("kaya")
        .join("designs")
        .join(name);
    fs::create_dir_all(path.parent().expect("design dir")).unwrap();
    fs::write(&path, "# design\n").unwrap();
}

fn write_ledger(dir: &TempDir, lines: &[String]) {
    let path = dir.path().join(LEDGER);
    fs::create_dir_all(path.parent().expect("ledger dir")).unwrap();
    fs::write(&path, lines.concat()).unwrap();
}

fn ledger_lines(dir: &TempDir) -> Vec<String> {
    fs::read_to_string(dir.path().join(LEDGER))
        .expect("ledger is readable")
        .lines()
        .map(str::to_owned)
        .collect()
}

/// Bytes between a generated block's begin and end markers.
fn generated_block_body(doc: &str, name: &str) -> String {
    let begin = format!("<!-- kaya:begin generated:{name} -->");
    let end = format!("<!-- kaya:end generated:{name} -->");
    let start = doc
        .find(&begin)
        .unwrap_or_else(|| panic!("the document carries the {name} block: {doc}"))
        + begin.len();
    let stop = doc[start..]
        .find(&end)
        .unwrap_or_else(|| panic!("the {name} block is closed: {doc}"))
        + start;
    doc[start..stop].to_string()
}

fn assertion_event(id: &str, text: &str, cites: &[&str]) -> Value {
    json!({
        "id": id,
        "schema": "kaya.event/v0",
        "type": "promotion.assertion.recorded",
        "timestamp": "2026-11-16T10:00:00Z",
        "actor": { "type": "human", "name": "alice" },
        "subject": { "type": "assertion", "id": id },
        "data": {
            "text": text,
            "status": "proposed",
            "event_ids": cites,
            "machine_proposed": false,
        },
        "provenance": { "tool": "kaya", "version": "0.1.0", "reference": id },
        "artifacts": [],
    })
}

fn node_event(event_id: &str, kind: &str, node_id: &str, title: &str) -> Value {
    json!({
        "id": event_id,
        "schema": "kaya.event/v0",
        "type": "model.entity.created",
        "timestamp": "2026-11-16T10:00:00Z",
        "actor": { "type": "human", "name": "alice" },
        "subject": { "type": kind, "id": node_id },
        "data": { "attributes": { "title": title } },
        "provenance": { "tool": "kaya", "version": "0.1.0", "reference": event_id },
        "artifacts": [],
    })
}

fn relationship_event(event_id: &str, relationship: &str, from: &str, to: &str) -> Value {
    json!({
        "id": event_id,
        "schema": "kaya.event/v0",
        "type": "model.relationship.asserted",
        "timestamp": "2026-11-16T10:00:00Z",
        "actor": { "type": "human", "name": "alice" },
        "subject": { "type": "Relationship", "id": from },
        "data": { "relationship": relationship, "from": from, "to": to },
        "provenance": { "tool": "kaya", "version": "0.1.0", "reference": event_id },
        "artifacts": [],
    })
}

fn experiment_event(event_id: &str, node_id: &str, title: &str) -> Value {
    node_event(event_id, "Experiment", node_id, title)
}

fn artifact_event(event_id: &str, node_id: &str, kind: &str, path: &str) -> Value {
    json!({
        "id": event_id,
        "schema": "kaya.event/v0",
        "type": "capture.artifact.observed",
        "timestamp": "2026-11-16T10:00:00Z",
        "actor": { "type": "tool", "name": "wai" },
        "subject": { "type": "Artifact", "id": node_id },
        "data": { "kind": kind, "path": path },
        "provenance": {
            "tool": "wai",
            "version": "2026.10.10",
            "reference": path
        },
        "artifacts": [path],
    })
}

fn source_event(event_id: &str, node_id: &str, path: &str, commit: &str) -> Value {
    json!({
        "id": event_id,
        "schema": "kaya.event/v0",
        "type": "model.entity.created",
        "timestamp": "2026-11-16T10:00:00Z",
        "actor": { "type": "human", "name": "alice" },
        "subject": { "type": "Source", "id": node_id },
        "data": { "attributes": { "path": path, "commit": commit } },
        "provenance": { "tool": "kaya", "version": "0.1.0", "reference": event_id },
        "artifacts": [],
    })
}

fn run_event(event_id: &str, run_id: &str, passed: u64) -> Value {
    json!({
        "id": event_id,
        "schema": "kaya.event/v0",
        "type": "experiment.run.completed",
        "timestamp": "2026-11-16T10:00:00Z",
        "actor": { "type": "tool", "name": "test-runner" },
        "subject": { "type": "Run", "id": run_id },
        "data": { "passed": passed, "failed": 0, "ignored": 0, "total": passed },
        "provenance": {
            "tool": "test-runner",
            "version": "libtest-json",
            "reference": "target/run.jsonl"
        },
        "artifacts": [],
    })
}

fn run_correction(event_id: &str, run_id: &str, supersedes: &str, passed: u64) -> Value {
    let mut event = run_event(event_id, run_id, passed);
    event["data"]["supersedes"] = json!(supersedes);
    event
}

fn git(dir: &TempDir, args: &[&str]) {
    let output = StdCommand::new("git")
        .current_dir(dir.path())
        .args(args)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .env("GIT_AUTHOR_NAME", "kaya")
        .env("GIT_AUTHOR_EMAIL", "kaya@example.com")
        .env("GIT_COMMITTER_NAME", "kaya")
        .env("GIT_COMMITTER_EMAIL", "kaya@example.com")
        .output()
        .expect("git runs");
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn commit_ledger(dir: &TempDir) {
    git(dir, &["init", "--quiet"]);
    git(dir, &["add", LEDGER]);
    git(dir, &["commit", "--quiet", "--message", "record ledger"]);
}

fn record_assertion(dir: &TempDir, id: &str, text: &str, cites: &[&str]) {
    let line = event_line(&assertion_event(id, text, cites));
    write_ledger(dir, &[line]);
}

#[test]
fn import_wai_appends_each_event_once() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    write_wai_design(&dir, "x.md");

    let first = assert_success(&import_wai(&dir));
    assert_eq!(first["envelope_kind"], json!("import"));
    assert_eq!(first["data"]["source"], json!("wai"));
    assert_eq!(first["data"]["events"], json!(1));
    assert_eq!(first["data"]["appended"], json!(1));
    assert_eq!(first["data"]["duplicates"], json!(0));
    assert_eq!(first["data"]["ledger"], json!(LEDGER));

    let stored = ledger_lines(&dir);
    assert_eq!(stored.len(), 1, "one artifact yields one ledger line");
    let event: Value = serde_json::from_str(&stored[0]).unwrap();
    assert_eq!(event["schema"], json!("kaya.event/v0"));
    assert_eq!(event["type"], json!("capture.artifact.observed"));
    assert_eq!(
        event["data"]["path"],
        json!(".wai/projects/kaya/designs/x.md")
    );

    let second = assert_success(&import_wai(&dir));
    assert_eq!(second["envelope_kind"], json!("import"));
    assert_eq!(second["data"]["events"], json!(1));
    assert_eq!(second["data"]["appended"], json!(0));
    assert_eq!(second["data"]["duplicates"], json!(1));
    assert_eq!(
        ledger_lines(&dir).len(),
        1,
        "a duplicate import never appends"
    );
}

#[test]
fn reimport_of_unchanged_tree_appends_nothing() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    write_wai_design(&dir, "2026-11-24-design.md");

    let first = assert_success(&import_wai(&dir));
    assert_eq!(first["data"]["appended"], json!(1));

    let second = assert_success(&import_wai(&dir));
    assert_eq!(second["data"]["appended"], json!(0));
    assert_eq!(second["data"]["duplicates"], json!(1));
    assert_eq!(ledger_lines(&dir).len(), 1, "unchanged tree appends nothing");
}

#[test]
fn edited_record_yields_correction_never_id_reuse() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    write_wai_design(&dir, "2026-11-24-design.md");

    let first = assert_success(&import_wai(&dir));
    assert_eq!(first["data"]["appended"], json!(1));

    let before = ledger_lines(&dir);
    assert_eq!(before.len(), 1, "one artifact yields one ledger line");
    let original: Value = serde_json::from_str(&before[0]).unwrap();
    let original_id = original["id"].as_str().expect("event carries an id");

    // Same path keeps the declared timestamp; only the content changes.
    let path = dir
        .path()
        .join(".wai")
        .join("projects")
        .join("kaya")
        .join("designs")
        .join("2026-11-24-design.md");
    fs::write(&path, "# revised heading\n").unwrap();

    let correction = assert_success(&import_wai(&dir));
    assert_eq!(correction["data"]["appended"], json!(1));

    let after = ledger_lines(&dir);
    assert_eq!(after.len(), 2, "the correction appends exactly one line");
    assert_eq!(
        after[0], before[0],
        "the ledger is append-only: the original line is untouched"
    );
    let correction_event: Value = serde_json::from_str(&after[1]).unwrap();
    let correction_id = correction_event["id"]
        .as_str()
        .expect("correction carries an id");
    assert_ne!(
        correction_id, original_id,
        "a correction mints a fresh id, never reuses the original"
    );
    assert_eq!(
        correction_event["data"]["supersedes"], original["id"],
        "the correction points back at the record it replaces"
    );
    assert_eq!(
        correction_event["data"]["title"],
        json!("revised heading"),
        "the title is the first # heading, minus its marker"
    );

    let replay = assert_success(&import_wai(&dir));
    assert_eq!(replay["data"]["appended"], json!(0));
    assert_eq!(replay["data"]["duplicates"], json!(1));
}

#[test]
fn corrected_state_revisit_mints_fresh_correction_and_replays_stable() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    write_wai_design(&dir, "2026-11-24-design.md");

    let design_path = dir
        .path()
        .join(".wai")
        .join("projects")
        .join("kaya")
        .join("designs")
        .join("2026-11-24-design.md");

    // "# design" -> "# revised heading" -> "# design" -> no change.
    let first = assert_success(&import_wai(&dir));
    assert_eq!(first["data"]["appended"], json!(1));
    let original: Value = serde_json::from_str(&ledger_lines(&dir)[0]).unwrap();
    let original_id = original["id"].as_str().expect("event carries an id").to_owned();

    fs::write(&design_path, "# revised heading\n").unwrap();
    let second = assert_success(&import_wai(&dir));
    assert_eq!(second["data"]["appended"], json!(1));
    let revised: Value = serde_json::from_str(&ledger_lines(&dir)[1]).unwrap();
    let revised_id = revised["id"].as_str().expect("correction carries an id").to_owned();
    assert_eq!(revised["data"]["supersedes"], original["id"]);

    // Back to the original heading: a fresh correction must override the
    // stored correction B, not collide with either existing id.
    fs::write(&design_path, "# design\n").unwrap();
    let third = assert_success(&import_wai(&dir));
    assert_eq!(third["data"]["appended"], json!(1));
    let before = ledger_lines(&dir);
    assert_eq!(before.len(), 3, "the revisit appends exactly one line");
    let revisit: Value = serde_json::from_str(&before[2]).unwrap();
    let revisit_id = revisit["id"].as_str().expect("correction carries an id");
    assert_ne!(revisit_id, original_id, "never reuses the original id");
    assert_ne!(revisit_id, revised_id, "a revisit mints a fresh correction id");
    assert_eq!(revisit["data"]["supersedes"], original["id"]);
    assert_eq!(
        revisit["data"]["title"],
        json!("design"),
        "the correction restores the original heading"
    );

    // No change: the whole journey replays byte-identically.
    let fourth = assert_success(&import_wai(&dir));
    assert_eq!(fourth["data"]["appended"], json!(0));
    assert_eq!(
        fourth["data"]["duplicates"], json!(1),
        "the replay lands on the stored correction byte-identically"
    );
    assert_eq!(ledger_lines(&dir).len(), 3, "no further lines are appended");
    let lines = ledger_lines(&dir);
    assert_eq!(lines[0], before[0], "the original line is untouched");
    let correction_ids: Vec<String> = [&lines[1], &lines[2]]
        .iter()
        .map(|line| {
            serde_json::from_str::<Value>(line)
                .unwrap()["id"]
                .as_str()
                .expect("correction carries an id")
                .to_owned()
        })
        .collect();
    assert_ne!(correction_ids[0], correction_ids[1]);
}

#[test]
fn revert_to_original_heading_mints_correction() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    write_wai_design(&dir, "2026-11-24-design.md");
    let design_path = dir
        .path()
        .join(".wai")
        .join("projects")
        .join("kaya")
        .join("designs")
        .join("2026-11-24-design.md");

    assert_success(&import_wai(&dir));
    let original: Value = serde_json::from_str(&ledger_lines(&dir)[0]).unwrap();
    let original_id = original["id"].as_str().expect("event carries an id");

    fs::write(&design_path, "# revised heading\n").unwrap();
    assert_success(&import_wai(&dir));
    assert_eq!(ledger_lines(&dir).len(), 2);

    // Revert the file to the original heading and import.
    fs::write(&design_path, "# design\n").unwrap();
    let revert = assert_success(&import_wai(&dir));
    assert_eq!(revert["data"]["appended"], json!(1));
    let lines = ledger_lines(&dir);
    assert_eq!(lines.len(), 3);
    let correction: Value = serde_json::from_str(&lines[2]).unwrap();
    assert_eq!(correction["data"]["supersedes"], original["id"]);
    assert_eq!(correction["data"]["title"], json!("design"));
    assert_ne!(
        correction["id"].as_str().expect("correction carries an id"),
        original_id,
        "the revert mints a fresh correction id"
    );

    // Importing again replays without appending.
    let replay = assert_success(&import_wai(&dir));
    assert_eq!(replay["data"]["appended"], json!(0));
    assert_eq!(
        replay["data"]["duplicates"], json!(1),
        "the replay lands on the stored correction byte-identically"
    );
}

#[test]
fn import_without_a_ledger_is_a_missing_ledger_error() {
    let dir = tempfile::tempdir().unwrap();

    let envelope = assert_failure(&import_wai(&dir));
    assert_eq!(envelope["error"]["label"], json!("missing-ledger"));
    let message = envelope["error"]["message"]
        .as_str()
        .expect("error envelopes carry a message");
    assert!(
        message.contains("kaya init"),
        "the message must suggest `kaya init`: {message}"
    );
    assert!(
        !dir.path().join(LEDGER).exists(),
        "import must not create a missing ledger"
    );
}

#[test]
fn failed_import_writes_no_events_to_the_ledger() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    write_wai_design(&dir, "x.md");

    let unparsable = dir
        .path()
        .join(".wai/projects/kaya/research/undated-binary.md");
    fs::create_dir_all(unparsable.parent().expect("research dir")).unwrap();
    fs::write(&unparsable, [0xff_u8, 0xfe]).unwrap();

    let envelope = assert_failure(&import_wai(&dir));
    assert_eq!(
        envelope["error"]["label"],
        json!("kaya.adapters.failure_atomic")
    );
    let message = envelope["error"]["message"]
        .as_str()
        .expect("error envelopes carry a message");
    assert!(message.contains("wai"), "names the source: {message}");
    assert!(
        message.contains("undated-binary.md"),
        "names the record: {message}"
    );
    assert!(
        ledger_lines(&dir).is_empty(),
        "a failed adapter run appends no events"
    );
}

#[test]
fn build_writes_index_with_generated_block_and_artifact() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    write_wai_design(&dir, "x.md");
    assert_success(&import_wai(&dir));

    let built = assert_success(&run(&dir, &["build", "--allow-uncommitted"]));
    assert_eq!(built["envelope_kind"], json!("build"));
    assert_eq!(built["data"]["out"], json!("docs/research"));
    assert_eq!(built["data"]["pending_proposals"], json!(0));
    assert_eq!(built["data"]["stale"], json!([]));
    assert!(
        built["data"]["ledger_digest"].is_string(),
        "build surfaces the ledger digest so callers can pin it: {built}"
    );
    assert_eq!(
        built["data"]["events"],
        json!(1),
        "build reports the number of ledger events it compiled"
    );
    let recomputed = built["data"]["recomputed"]
        .as_array()
        .expect("recomputed is a list");
    assert!(
        recomputed
            .iter()
            .any(|name| name == "orientation.research-map"),
        "a fresh research map is recomputed: {recomputed:?}"
    );

    let index = fs::read_to_string(dir.path().join(INDEX)).expect("index exists");
    assert!(
        index.contains("<!-- kaya:begin generated:orientation.research-map -->"),
        "index carries the generated block marker: {index}"
    );
    assert!(
        index.contains(".wai/projects/kaya/designs/x.md"),
        "index lists the observed artifact: {index}"
    );

    let again = assert_success(&run(&dir, &["build", "--allow-uncommitted"]));
    let unchanged = again["data"]["unchanged"]
        .as_array()
        .expect("unchanged is a list");
    assert!(
        unchanged
            .iter()
            .any(|name| name == "orientation.research-map"),
        "a byte-identical rebuild is unchanged: {unchanged:?}"
    );
    assert_eq!(
        again["data"]["recomputed"],
        json!([]),
        "nothing is rewritten when the block matches"
    );
}

#[test]
fn build_writes_one_report_per_experiment() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    let experiment = event_line(&experiment_event(
        "entity-1",
        "kaya/Experiment/exp-one",
        "Exp One",
    ));
    let other_project = event_line(&experiment_event(
        "entity-2",
        "alpha/Experiment/exp-one",
        "Other Exp One",
    ));
    // Each experiment declares intent, so a specification block renders.
    let question = event_line(&node_event(
        "entity-3",
        "Question",
        "kaya/Question/q-one",
        "Q One",
    ));
    let other_question = event_line(&node_event(
        "entity-4",
        "Question",
        "alpha/Question/q-one",
        "Other Q One",
    ));
    let investigates = event_line(&relationship_event(
        "rel-1",
        "investigates",
        "kaya/Question/q-one",
        "kaya/Experiment/exp-one",
    ));
    let other_investigates = event_line(&relationship_event(
        "rel-2",
        "investigates",
        "alpha/Question/q-one",
        "alpha/Experiment/exp-one",
    ));
    write_ledger(
        &dir,
        &[
            experiment,
            other_project,
            question,
            other_question,
            investigates,
            other_investigates,
        ],
    );

    let built = assert_success(&run(&dir, &["build", "--allow-uncommitted"]));
    let recomputed = built["data"]["recomputed"]
        .as_array()
        .expect("recomputed is a list");
    for name in [
        "exploration.experiment-specification:kaya/Experiment/exp-one",
        "exploration.experiment-specification:alpha/Experiment/exp-one",
    ] {
        assert!(
            recomputed.iter().any(|candidate| candidate == name),
            "each specification block has a distinct summary name ({name}): {recomputed:?}"
        );
    }

    for path in [
        "docs/research/experiments/kaya-Experiment-exp-one.md",
        "docs/research/experiments/alpha-Experiment-exp-one.md",
    ] {
        let report = fs::read_to_string(dir.path().join(path))
            .unwrap_or_else(|err| panic!("the report path sanitizes every id for {path}: {err}"));
        assert!(
            report.contains("<!-- kaya:begin generated:exploration.experiment-specification -->"),
            "report carries the specification block marker: {report}"
        );
        assert!(
            !report.contains("exploration.experiment-record"),
            "an experiment with no runs writes no record block: {report}"
        );
    }

    let sources = built["data"]["sources"]
        .as_object()
        .expect("build exposes the merged source map");
    assert!(
        sources.contains_key("exploration.experiment-specification:kaya/Experiment/exp-one")
            && sources
                .contains_key("exploration.experiment-specification:alpha/Experiment/exp-one"),
        "each report has its own source-map entry: {sources:?}"
    );

    let index = fs::read_to_string(dir.path().join(INDEX)).expect("index exists");
    assert!(
        index.contains("kaya/Experiment/exp-one") && index.contains("alpha/Experiment/exp-one"),
        "index lists both experiments: {index}"
    );
    assert!(
        index.contains("_4 nodes from 6 events._"),
        "the index summary counts rendered nodes and ledger events: {index}"
    );
}

#[test]
fn build_writes_understanding_from_recorded_records() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    // Run and Artifact records exist, so the findings-and-analysis block
    // proves it renders only its own kind: neither id nor path may leak.
    let lines = [
        event_line(&node_event(
            "entity-1",
            "Finding",
            "kaya/Finding/f-hit-rate",
            "Cache hit rate rose",
        )),
        event_line(&node_event(
            "entity-2",
            "Decision",
            "kaya/Decision/d-ship-cache",
            "Ship the cache",
        )),
        event_line(&source_event(
            "entity-3",
            "kaya/Source/src-cache-rs@abc123",
            "src/cache.rs",
            "abc123",
        )),
        event_line(&run_event("run-1", "kaya/Run/run-1", 3)),
        event_line(&artifact_event(
            "artifact-1",
            "kaya/Artifact/run-jsonl",
            "reports",
            "target/run.jsonl",
        )),
        event_line(&relationship_event(
            "rel-1",
            "influences",
            "kaya/Decision/d-ship-cache",
            "kaya/Finding/f-hit-rate",
        )),
    ];
    write_ledger(&dir, &lines);

    let built = assert_success(&run(&dir, &["build", "--allow-uncommitted"]));
    let recomputed = built["data"]["recomputed"]
        .as_array()
        .expect("recomputed is a list");
    for name in [
        "understanding.findings-and-analysis",
        "understanding.decision-record",
    ] {
        assert!(
            recomputed.iter().any(|candidate| candidate == name),
            "the build compiles the {name} block: {recomputed:?}"
        );
    }
    let omitted = built["data"]["omitted"]
        .as_array()
        .expect("omitted is a list");
    assert!(
        !omitted
            .iter()
            .any(|entry| entry["family"] == json!("understanding")),
        "a kind with records is never reported omitted: {omitted:?}"
    );

    let sources = built["data"]["sources"]
        .as_object()
        .expect("build exposes the merged source map");
    assert_eq!(
        sources["understanding.findings-and-analysis"],
        json!(["kaya/Finding/f-hit-rate"]),
        "the findings block's source map names the rendered finding: {sources:?}"
    );
    assert!(
        sources.contains_key("understanding.decision-record"),
        "the decision record has its own source-map entry: {sources:?}"
    );
    assert_eq!(
        sources["understanding.decision-record"],
        json!([
            "kaya/Decision/d-ship-cache",
            "kaya/Finding/f-hit-rate",
            "kaya/Source/src-cache-rs@abc123",
        ]),
        "the decision source map is exactly its decisions and evidence: {sources:?}"
    );
    for name in [
        "understanding.findings-and-analysis",
        "understanding.decision-record",
    ] {
        let ids = sources[name]
            .as_array()
            .unwrap_or_else(|| panic!("{name} maps to a list of ids: {sources:?}"));
        assert!(
            !ids.iter().any(|id| {
                let id = id.as_str().expect("every source id is a string");
                id.starts_with("kaya/Run/") || id.starts_with("kaya/Artifact/")
            }),
            "run and artifact ids stay out of the understanding source maps: {ids:?}"
        );
    }

    let understanding = fs::read_to_string(dir.path().join("docs/research/understanding.md"))
        .expect("the understanding document is written");
    let findings = generated_block_body(&understanding, "understanding.findings-and-analysis");
    assert!(
        findings.contains("- kaya/Finding/f-hit-rate — Cache hit rate rose"),
        "the findings block renders the recorded finding: {findings}"
    );
    for forbidden in [
        "kaya/Run/run-1",
        "### Runs",
        "kaya/Artifact/run-jsonl",
        "target/run.jsonl",
    ] {
        assert!(
            !findings.contains(forbidden),
            "run and artifact rows stay out of the findings block ({forbidden}): {findings}"
        );
    }

    let decisions = generated_block_body(&understanding, "understanding.decision-record");
    assert!(
        decisions.contains("- kaya/Decision/d-ship-cache — Ship the cache"),
        "the decision record renders the recorded decision: {decisions}"
    );
    assert!(
        decisions.contains("- kaya/Source/src-cache-rs@abc123 — src/cache.rs@abc123"),
        "evidence names the source revision as path@commit: {decisions}"
    );
    assert!(
        decisions.contains("- kaya/Finding/f-hit-rate — see kaya/Decision/d-ship-cache"),
        "the influences edge renders as a finding reference: {decisions}"
    );
    for forbidden in ["kaya/Run/run-1", "target/run.jsonl"] {
        assert!(
            !decisions.contains(forbidden),
            "run and artifact rows stay out of the decision record ({forbidden}): {decisions}"
        );
    }
}

#[test]
fn build_flags_authored_understanding_prose_and_pins_the_decision_source_map() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    let authored = "# Understanding notes\n\n\
        <!-- kaya:authored -->\n\
        The corrected run settled the cache question.\n\
        <!-- kaya:cites run-1 -->\n\
        <!-- kaya:end authored -->\n";
    fs::create_dir_all(dir.path().join("docs/research")).unwrap();
    fs::write(dir.path().join("docs/research/understanding.md"), authored)
        .expect("the authored understanding document is writable");

    // A run exists, is superseded, and produces an artifact: none of those
    // facts may leak into the understanding blocks or their source maps.
    let lines = [
        event_line(&node_event(
            "entity-1",
            "Finding",
            "kaya/Finding/f-hit-rate",
            "Cache hit rate rose",
        )),
        event_line(&node_event(
            "entity-2",
            "Decision",
            "kaya/Decision/d-ship-cache",
            "Ship the cache",
        )),
        event_line(&source_event(
            "entity-3",
            "kaya/Source/src-cache-rs@abc123",
            "src/cache.rs",
            "abc123",
        )),
        event_line(&run_event("run-1", "kaya/Run/run-1", 3)),
        event_line(&run_correction(
            "correction-1",
            "kaya/Run/run-1",
            "run-1",
            4,
        )),
        event_line(&artifact_event(
            "artifact-1",
            "kaya/Artifact/run-jsonl",
            "reports",
            "target/run.jsonl",
        )),
        event_line(&relationship_event(
            "rel-1",
            "influences",
            "kaya/Decision/d-ship-cache",
            "kaya/Finding/f-hit-rate",
        )),
    ];
    write_ledger(&dir, &lines);

    let built = assert_success(&run(&dir, &["build", "--allow-uncommitted"]));

    assert_eq!(
        built["data"]["stale"],
        json!([{ "block": "authored-1", "evidence": ["run-1"] }]),
        "a correction under authored understanding prose produces a stale entry: {built}"
    );

    let sources = built["data"]["sources"]
        .as_object()
        .expect("build exposes the merged source map");
    assert_eq!(
        sources["understanding.decision-record"],
        json!([
            "kaya/Decision/d-ship-cache",
            "kaya/Finding/f-hit-rate",
            "kaya/Source/src-cache-rs@abc123",
        ]),
        "the decision source map names the decision, its evidence, and the source: {sources:?}"
    );
    assert_eq!(
        sources["understanding.findings-and-analysis"],
        json!(["kaya/Finding/f-hit-rate"]),
        "the findings source map names the rendered records only: {sources:?}"
    );
    for name in [
        "understanding.findings-and-analysis",
        "understanding.decision-record",
    ] {
        let ids = sources[name]
            .as_array()
            .unwrap_or_else(|| panic!("{name} maps to a list of ids: {sources:?}"));
        assert!(
            !ids.iter().any(|id| {
                let id = id.as_str().expect("every source id is a string");
                id.starts_with("kaya/Run/") || id.starts_with("kaya/Artifact/")
            }),
            "run and artifact ids stay out of the understanding source maps: {ids:?}"
        );
    }

    let understanding = fs::read_to_string(dir.path().join("docs/research/understanding.md"))
        .expect("the understanding document is written");
    assert!(
        understanding.starts_with(authored),
        "authored understanding prose is never rewritten: {understanding}"
    );
    let decisions = generated_block_body(&understanding, "understanding.decision-record");
    for forbidden in [
        "kaya/Run/run-1",
        "kaya/Artifact/run-jsonl",
        "target/run.jsonl",
    ] {
        assert!(
            !decisions.contains(forbidden),
            "run and artifact rows stay out of the decision record ({forbidden}): {decisions}"
        );
    }
    assert!(
        decisions.contains("- kaya/Source/src-cache-rs@abc123 — src/cache.rs@abc123"),
        "the pinned source renders once as path@commit: {decisions}"
    );
}

#[test]
fn omitted_kinds_reported_once_in_build_envelope() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    let lines = [
        event_line(&experiment_event(
            "entity-1",
            "kaya/Experiment/exp-one",
            "Exp One",
        )),
        event_line(&experiment_event(
            "entity-2",
            "kaya/Experiment/exp-two",
            "Exp Two",
        )),
        event_line(&node_event(
            "entity-3",
            "Question",
            "kaya/Question/q-one",
            "Q One",
        )),
        event_line(&node_event(
            "entity-4",
            "Question",
            "kaya/Question/q-two",
            "Q Two",
        )),
        event_line(&relationship_event(
            "rel-1",
            "investigates",
            "kaya/Question/q-one",
            "kaya/Experiment/exp-one",
        )),
        event_line(&relationship_event(
            "rel-2",
            "investigates",
            "kaya/Question/q-two",
            "kaya/Experiment/exp-two",
        )),
        // Only exp-one has a run; exp-two has no records to report.
        event_line(&run_event("run-1", "kaya/Run/run-1", 1)),
        event_line(&relationship_event(
            "rel-3",
            "executes",
            "kaya/Run/run-1",
            "kaya/Experiment/exp-one",
        )),
    ];
    write_ledger(&dir, &lines);

    let built = assert_success(&run(&dir, &["build", "--allow-uncommitted"]));
    assert_eq!(
        built["data"]["omitted"],
        json!([
            {
                "family": "exploration",
                "kind": "experiment-record",
                "reason": "no-records",
            },
            {
                "family": "reuse-handoff",
                "kind": "handoff",
                "reason": "no-records",
            },
            {
                "family": "reuse-handoff",
                "kind": "knowledge-brief",
                "reason": "no-records",
            },
            {
                "family": "reuse-handoff",
                "kind": "reproduction-package",
                "reason": "no-records",
            },
            {
                "family": "reuse-handoff",
                "kind": "technical-reference",
                "reason": "no-records",
            },
            {
                "family": "understanding",
                "kind": "decision-record",
                "reason": "no-records",
            },
            {
                "family": "understanding",
                "kind": "findings-and-analysis",
                "reason": "no-records",
            },
        ]),
        "each omitted kind is named exactly once, sorted and deduplicated: {built}"
    );

    let recomputed = built["data"]["recomputed"]
        .as_array()
        .expect("recomputed is a list");
    assert!(
        recomputed
            .iter()
            .any(|name| name == "exploration.experiment-record:kaya/Experiment/exp-one"),
        "the experiment with a run writes a record block: {recomputed:?}"
    );
    assert!(
        !recomputed
            .iter()
            .any(|name| name == "exploration.experiment-record:kaya/Experiment/exp-two"),
        "an omitted kind has no summary key: {recomputed:?}"
    );

    let with_run = fs::read_to_string(
        dir.path()
            .join("docs/research/experiments/kaya-Experiment-exp-one.md"),
    )
    .expect("the run-bearing report exists");
    assert!(
        with_run.contains("<!-- kaya:begin generated:exploration.experiment-record -->"),
        "a kind with records keeps its block: {with_run}"
    );

    let without_run = fs::read_to_string(
        dir.path()
            .join("docs/research/experiments/kaya-Experiment-exp-two.md"),
    )
    .expect("the runless report exists");
    assert!(
        without_run.contains("<!-- kaya:begin generated:exploration.experiment-specification -->"),
        "the runless experiment still renders its specification: {without_run}"
    );
    assert!(
        !without_run.contains("exploration.experiment-record"),
        "an empty kind leaves no block, markers, or placeholder: {without_run}"
    );
}

#[test]
fn omitted_kinds_are_sorted_and_deduped_across_experiments() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    // exp-a and exp-c declare intent but have no runs; exp-b has no edges at
    // all, so it omits both kinds and gets no report document.
    let lines = [
        event_line(&experiment_event(
            "entity-1",
            "kaya/Experiment/exp-a",
            "Exp A",
        )),
        event_line(&experiment_event(
            "entity-2",
            "kaya/Experiment/exp-b",
            "Exp B",
        )),
        event_line(&experiment_event(
            "entity-3",
            "kaya/Experiment/exp-c",
            "Exp C",
        )),
        event_line(&node_event(
            "entity-4",
            "Question",
            "kaya/Question/q-a",
            "Q A",
        )),
        event_line(&node_event(
            "entity-5",
            "Question",
            "kaya/Question/q-c",
            "Q C",
        )),
        event_line(&relationship_event(
            "rel-1",
            "investigates",
            "kaya/Question/q-a",
            "kaya/Experiment/exp-a",
        )),
        event_line(&relationship_event(
            "rel-2",
            "investigates",
            "kaya/Question/q-c",
            "kaya/Experiment/exp-c",
        )),
    ];
    write_ledger(&dir, &lines);

    let built = assert_success(&run(&dir, &["build", "--allow-uncommitted"]));
    assert_eq!(
        built["data"]["omitted"],
        json!([
            {
                "family": "exploration",
                "kind": "experiment-record",
                "reason": "no-records",
            },
            {
                "family": "exploration",
                "kind": "experiment-specification",
                "reason": "no-records",
            },
            {
                "family": "reuse-handoff",
                "kind": "handoff",
                "reason": "no-records",
            },
            {
                "family": "reuse-handoff",
                "kind": "knowledge-brief",
                "reason": "no-records",
            },
            {
                "family": "reuse-handoff",
                "kind": "reproduction-package",
                "reason": "no-records",
            },
            {
                "family": "reuse-handoff",
                "kind": "technical-reference",
                "reason": "no-records",
            },
            {
                "family": "understanding",
                "kind": "decision-record",
                "reason": "no-records",
            },
            {
                "family": "understanding",
                "kind": "findings-and-analysis",
                "reason": "no-records",
            },
        ]),
        "three experiments yield one sorted, deduplicated pair of omissions: {built}"
    );

    assert!(
        !dir.path()
            .join("docs/research/experiments/kaya-Experiment-exp-b.md")
            .exists(),
        "an experiment with nothing to render gets no report document"
    );
}

#[test]
fn build_tolerates_a_missing_report_when_corrections_exist() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    // The correction makes `flag_stale_authored` inspect every compiled
    // document, including the report the edge-less experiment never writes.
    let lines = [
        event_line(&experiment_event(
            "entity-1",
            "kaya/Experiment/exp-empty",
            "Empty Exp",
        )),
        event_line(&run_event("run-1", "kaya/Run/run-1", 1)),
        event_line(&run_correction(
            "correction-1",
            "kaya/Run/run-1",
            "run-1",
            2,
        )),
    ];
    write_ledger(&dir, &lines);

    let built = assert_success(&run(&dir, &["build", "--allow-uncommitted"]));
    assert_eq!(
        built["data"]["omitted"],
        json!([
            {
                "family": "exploration",
                "kind": "experiment-record",
                "reason": "no-records",
            },
            {
                "family": "exploration",
                "kind": "experiment-specification",
                "reason": "no-records",
            },
            {
                "family": "exploration",
                "kind": "exploration-brief",
                "reason": "no-records",
            },
            {
                "family": "orientation",
                "kind": "project-overview",
                "reason": "no-records",
            },
            {
                "family": "reuse-handoff",
                "kind": "handoff",
                "reason": "no-records",
            },
            {
                "family": "reuse-handoff",
                "kind": "knowledge-brief",
                "reason": "no-records",
            },
            {
                "family": "reuse-handoff",
                "kind": "reproduction-package",
                "reason": "no-records",
            },
            {
                "family": "reuse-handoff",
                "kind": "technical-reference",
                "reason": "no-records",
            },
            {
                "family": "understanding",
                "kind": "decision-record",
                "reason": "no-records",
            },
            {
                "family": "understanding",
                "kind": "findings-and-analysis",
                "reason": "no-records",
            },
        ]),
        "a runless, edgeless experiment omits both kinds: {built}"
    );
    assert!(
        !dir.path()
            .join("docs/research/experiments/kaya-Experiment-exp-empty.md")
            .exists(),
        "no document is written when both kinds are omitted"
    );
}

#[test]
fn ledger_assertions_do_not_create_a_record_block_for_a_runless_experiment() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    // The assertion cites evidence unrelated to this experiment: only
    // declared intent renders, and the record kind stays omitted.
    let lines = [
        event_line(&experiment_event(
            "entity-1",
            "kaya/Experiment/exp-one",
            "Exp One",
        )),
        event_line(&node_event(
            "entity-2",
            "Question",
            "kaya/Question/q-one",
            "Q One",
        )),
        event_line(&relationship_event(
            "rel-1",
            "investigates",
            "kaya/Question/q-one",
            "kaya/Experiment/exp-one",
        )),
        event_line(&assertion_event(
            "assertion-1",
            "the model holds",
            &["evt-unrelated"],
        )),
    ];
    write_ledger(&dir, &lines);

    let built = assert_success(&run(&dir, &["build", "--allow-uncommitted"]));
    let report = fs::read_to_string(
        dir.path()
            .join("docs/research/experiments/kaya-Experiment-exp-one.md"),
    )
    .expect("the specification-bearing report exists");
    assert!(
        report.contains("<!-- kaya:begin generated:exploration.experiment-specification -->"),
        "declared intent still renders: {report}"
    );
    assert!(
        !report.contains("exploration.experiment-record"),
        "a ledger-global assertion must not materialize a record block: {report}"
    );
    assert!(
        !report.contains("## Interpretations"),
        "no interpretation section survives without observed execution: {report}"
    );
    assert_eq!(
        built["data"]["omitted"],
        json!([
            {
                "family": "exploration",
                "kind": "experiment-record",
                "reason": "no-records",
            },
            {
                "family": "reuse-handoff",
                "kind": "handoff",
                "reason": "no-records",
            },
            {
                "family": "reuse-handoff",
                "kind": "knowledge-brief",
                "reason": "no-records",
            },
            {
                "family": "reuse-handoff",
                "kind": "reproduction-package",
                "reason": "no-records",
            },
            {
                "family": "reuse-handoff",
                "kind": "technical-reference",
                "reason": "no-records",
            },
            {
                "family": "understanding",
                "kind": "decision-record",
                "reason": "no-records",
            },
            {
                "family": "understanding",
                "kind": "findings-and-analysis",
                "reason": "no-records",
            },
        ]),
        "the record kind is omitted even though an assertion exists: {built}"
    );
}

#[test]
fn build_expect_digest_pins_the_ledger() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    write_wai_design(&dir, "x.md");
    assert_success(&import_wai(&dir));

    let built = assert_success(&run(&dir, &["build", "--allow-uncommitted"]));
    let digest = built["data"]["ledger_digest"]
        .as_str()
        .expect("build reports its ledger digest")
        .to_owned();

    let pinned = assert_success(&run(&dir, &["build", "--expect-digest", digest.as_str()]));
    assert_eq!(pinned["data"]["ledger_digest"], json!(digest));
    assert!(
        pinned["data"]["unchanged"]
            .as_array()
            .expect("unchanged is a list")
            .iter()
            .any(|name| name == "orientation.research-map"),
        "the pinned rebuild leaves the research map alone"
    );

    let bogus = "0".repeat(64);
    let rejected = assert_failure(&run(&dir, &["build", "--expect-digest", bogus.as_str()]));
    assert_eq!(
        rejected["error"]["label"],
        json!("kaya.ledger.digest_mismatch"),
        "an unpinned ledger is refused, not served"
    );
}

#[test]
fn build_flags_authored_citations_superseded_by_a_correction() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    let authored = "# Notes\n\n\
        <!-- kaya:authored -->\n\
        The run was fast.\n\
        <!-- kaya:cites run-1 -->\n\
        <!-- kaya:end authored -->\n";
    fs::create_dir_all(dir.path().join("docs/research")).unwrap();
    fs::write(dir.path().join(INDEX), authored).unwrap();

    let run_line = event_line(&run_event("run-1", "kaya/Run/run-1", 1));
    let correction = event_line(&run_correction(
        "correction-1",
        "kaya/Run/run-1",
        "run-1",
        2,
    ));
    write_ledger(&dir, &[run_line, correction]);

    let built = assert_success(&run(&dir, &["build", "--allow-uncommitted"]));
    assert_eq!(
        built["data"]["stale"],
        json!([{ "block": "authored-1", "evidence": ["run-1"] }]),
        "the correction moved evidence cited by authored prose"
    );

    let after = fs::read_to_string(dir.path().join(INDEX)).expect("index exists");
    assert!(
        after.starts_with(authored),
        "authored bytes are never rewritten: {after}"
    );
    assert!(
        after.contains("_1 nodes from 2 events._"),
        "the summary counts rendered nodes and all ledger events: {after}"
    );
}

#[test]
fn build_refuses_an_in_place_edit_of_committed_ledger_history() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    let first = event_line(&run_event("run-1", "kaya/Run/run-1", 1));
    write_ledger(&dir, &[first]);
    commit_ledger(&dir);

    let path = dir.path().join(LEDGER);
    let committed = fs::read_to_string(&path).expect("ledger is readable");
    fs::write(&path, committed.replacen("\"run-1\"", "\"run-edited\"", 1)).unwrap();

    let envelope = assert_failure(&run(&dir, &["build", "--allow-uncommitted"]));
    assert_eq!(
        envelope["error"]["label"],
        json!("kaya.ledger.uncommitted_history"),
        "rewritten committed bytes are never served, even with --allow-uncommitted"
    );
    let message = envelope["error"]["message"]
        .as_str()
        .expect("error envelopes carry a message");
    assert!(
        message.contains("line 1"),
        "the refusal names the first unverified line: {message}"
    );
}

#[test]
fn build_accepts_committed_ledger_growth_with_complete_lines() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    let first = event_line(&run_event("run-1", "kaya/Run/run-1", 1));
    write_ledger(&dir, std::slice::from_ref(&first));
    commit_ledger(&dir);

    let second = event_line(&run_event("run-2", "kaya/Run/run-2", 2));
    write_ledger(&dir, &[first, second]);

    let built = assert_success(&run(&dir, &["build"]));
    assert!(
        built["data"]["recomputed"]
            .as_array()
            .expect("recomputed is a list")
            .iter()
            .any(|name| name == "orientation.research-map"),
        "a complete committed-growth suffix is verified and built: {built}"
    );
}

#[test]
fn build_accepts_an_untracked_ledger_only_when_allowed_or_pinned() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    write_ledger(
        &dir,
        &[event_line(&run_event("run-1", "kaya/Run/run-1", 1))],
    );

    let refused = assert_failure(&run(&dir, &["build"]));
    assert_eq!(
        refused["error"]["label"],
        json!("kaya.ledger.uncommitted_history")
    );
    let message = refused["error"]["message"]
        .as_str()
        .expect("error envelopes carry a message");
    assert!(
        message.contains("--expect-digest") && message.contains("--allow-uncommitted"),
        "the refusal explains how to pin or accept the ledger: {message}"
    );

    let built = assert_success(&run(&dir, &["build", "--allow-uncommitted"]));
    assert_eq!(built["data"]["pending_proposals"], json!(0));
}

#[test]
fn build_writes_reference_documents_and_leaves_the_ledger_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    let handoff_path = ".wai/projects/kaya/handoffs/h.md";
    write_ledger(
        &dir,
        &[event_line(&artifact_event(
            "artifact-1",
            "kaya/Artifact/handoffs-h",
            "handoffs",
            handoff_path,
        ))],
    );

    let ledger = dir.path().join(LEDGER);
    let bytes_before = fs::read(&ledger).expect("the ledger is readable");
    let events_before = ledger_lines(&dir).len();

    let built = assert_success(&run(&dir, &["build", "--allow-uncommitted"]));

    let recomputed = built["data"]["recomputed"]
        .as_array()
        .expect("recomputed is a list");
    assert!(
        recomputed
            .iter()
            .any(|name| name == "reuse-handoff.handoff"),
        "the handoff document reports its reuse block: {recomputed:?}"
    );
    assert_eq!(
        built["data"]["omitted"],
        json!([
            {
                "family": "exploration",
                "kind": "exploration-brief",
                "reason": "no-records",
            },
            {
                "family": "orientation",
                "kind": "project-overview",
                "reason": "no-records",
            },
            {
                "family": "reuse-handoff",
                "kind": "knowledge-brief",
                "reason": "no-records",
            },
            {
                "family": "reuse-handoff",
                "kind": "reproduction-package",
                "reason": "no-records",
            },
            {
                "family": "reuse-handoff",
                "kind": "technical-reference",
                "reason": "no-records",
            },
            {
                "family": "understanding",
                "kind": "decision-record",
                "reason": "no-records",
            },
            {
                "family": "understanding",
                "kind": "findings-and-analysis",
                "reason": "no-records",
            },
        ]),
        "no Source nodes means the reproduction package is omitted: {built}"
    );

    let handoff = fs::read_to_string(dir.path().join("docs/research/handoff.md"))
        .expect("the handoff document is written");
    assert!(
        handoff.contains("<!-- kaya:begin generated:reuse-handoff.handoff -->"),
        "the handoff document carries its generated block marker: {handoff}"
    );
    assert!(
        handoff.contains(handoff_path),
        "every handoff row names its wai artifact reference: {handoff}"
    );
    assert!(
        handoff.contains("kaya/Artifact/handoffs-h"),
        "the reference row keeps the source node id: {handoff}"
    );
    assert!(
        !handoff.contains("reuse-handoff.reproduction-package"),
        "an omitted kind leaves no block in the handoff document: {handoff}"
    );
    assert!(
        !dir.path()
            .join("docs/research/reproduction-package.md")
            .exists(),
        "an omitted kind gets no document of its own"
    );

    let bytes_after = fs::read(&ledger).expect("the ledger is readable");
    assert_eq!(
        bytes_after, bytes_before,
        "a reuse document copies no payload into the ledger"
    );
    assert_eq!(
        ledger_lines(&dir).len(),
        events_before,
        "a reuse document creates no ledger entity"
    );
}

#[test]
fn build_refuses_a_legacy_block_in_a_reuse_document() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    let handoff_path = ".wai/projects/kaya/handoffs/h.md";
    // A Source row means the reproduction package has something to write:
    // the refusal must happen before that document is created.
    write_ledger(
        &dir,
        &[
            event_line(&source_event(
                "source-1",
                "kaya/Source/src-main-rs@9f2c1a",
                "src/main.rs",
                "9f2c1a",
            )),
            event_line(&artifact_event(
                "artifact-1",
                "kaya/Artifact/handoffs-h",
                "handoffs",
                handoff_path,
            )),
        ],
    );

    let seeded = "\
# Handoffs

<!-- kaya:begin generated:experiment-report -->
## Runs

- kaya/Run/run-1
<!-- kaya:end generated:experiment-report -->
";
    let document = dir.path().join("docs/research/handoff.md");
    fs::create_dir_all(document.parent().expect("document dir")).unwrap();
    fs::write(&document, seeded).unwrap();

    let envelope = assert_failure(&run(&dir, &["build", "--allow-uncommitted"]));
    assert_eq!(
        envelope["error"]["label"],
        json!("kaya.documents.legacy_block_refused"),
        "a legacy block in a reuse document is refused by the vocabulary gate"
    );
    let message = envelope["error"]["message"]
        .as_str()
        .expect("error envelopes carry a message");
    assert!(
        message.contains("experiment-report"),
        "the refusal names the offending block: {message}"
    );
    assert_eq!(
        fs::read_to_string(&document).expect("the document is readable"),
        seeded,
        "a refused document keeps its bytes untouched"
    );
    assert!(
        !dir.path()
            .join("docs/research/reproduction-package.md")
            .exists(),
        "the reproduction package is not written before the refusal"
    );
}

#[test]
fn status_counts_pending_proposals() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    record_assertion(&dir, "assertion-1", "the model holds", &["evt-1"]);

    let status = assert_success(&run(&dir, &["status"]));
    assert_eq!(status["envelope_kind"], json!("status"));
    assert_eq!(status["data"]["events"], json!(1));
    assert_eq!(
        status["data"]["pending_proposals"],
        json!(1),
        "one proposed assertion awaits a human"
    );
}

/// Over-budget generation is ranked/capped at the point of use, never surfaced
/// whole (pending_proposals_bounded).
#[test]
fn status_caps_pending_proposals_at_the_budget() {
    let budget = kaya::promotion::PROPOSAL_BUDGET;

    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    // write_ledger replaces the file, so all assertion lines are written in
    // one batch instead of one overwriting call at a time.
    let lines: Vec<String> = (1..=(budget + 1))
        .map(|i| {
            event_line(&assertion_event(
                &format!("assertion-{i}"),
                "the model holds",
                &["evt"],
            ))
        })
        .collect();
    write_ledger(&dir, &lines);

    let status = assert_success(&run(&dir, &["status"]));
    assert_eq!(
        status["data"]["pending_proposals"],
        json!(budget),
        "status surfaces at most the proposal budget, not every proposal"
    );
}

#[test]
fn promote_then_challenge_records_human_events() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    record_assertion(&dir, "assertion-1", "the model holds", &["evt-1"]);

    let promoted = assert_success(&promote(&dir, "assertion-1", "alice"));
    assert_eq!(promoted["envelope_kind"], json!("promote"));
    assert_eq!(
        promoted["data"]["event"]["type"],
        json!("promotion.conclusion.established")
    );
    assert_eq!(promoted["data"]["event"]["actor"]["type"], json!("human"));
    assert_eq!(
        promoted["data"]["event"]["data"]["assertion"],
        json!("assertion-1")
    );
    assert_eq!(ledger_lines(&dir).len(), 2, "approval is one new event");

    let status = assert_success(&run(&dir, &["status"]));
    assert_eq!(status["data"]["pending_proposals"], json!(0));

    let challenged = assert_success(&challenge(&dir, "assertion-1", &["evt-2", "evt-3"], "bob"));
    assert_eq!(challenged["envelope_kind"], json!("challenge"));
    assert_eq!(
        challenged["data"]["event"]["type"],
        json!("promotion.conclusion.challenged")
    );
    assert_eq!(
        challenged["data"]["event"]["data"]["counter_event_ids"],
        json!(["evt-2", "evt-3"])
    );
    assert_eq!(ledger_lines(&dir).len(), 3, "challenge is one new event");
}

#[test]
fn promote_without_a_recorded_assertion_is_a_promotion_refusal() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);

    let envelope = assert_failure(&promote(&dir, "assertion-missing", "alice"));
    let label = envelope["error"]["label"]
        .as_str()
        .expect("error envelopes carry a label");
    assert!(
        label.starts_with("kaya.promotion."),
        "the refusal carries the promotion label: {label}"
    );
    assert_eq!(label, "kaya.promotion.unknown_assertion");
    assert!(
        ledger_lines(&dir).is_empty(),
        "a refused promotion appends nothing"
    );
}

#[test]
fn challenge_of_an_unestablished_conclusion_is_a_promotion_refusal() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    record_assertion(&dir, "assertion-1", "the model holds", &["evt-1"]);

    let envelope = assert_failure(&challenge(&dir, "assertion-1", &["evt-2"], "bob"));
    assert_eq!(
        envelope["error"]["label"],
        json!("kaya.promotion.challenge_target")
    );
    assert_eq!(
        ledger_lines(&dir).len(),
        1,
        "a refused challenge appends nothing"
    );
}

/// `data.pages` names each published page, either as a bare name or as a page
/// object; both shapes carry the same vocabularies.
fn published_page_names(pages: &Value) -> Vec<String> {
    pages
        .as_array()
        .expect("publish reports its pages as a list")
        .iter()
        .map(|page| {
            page.as_str()
                .or_else(|| page.get("name").and_then(Value::as_str))
                .unwrap_or_else(|| panic!("each published page names itself: {page}"))
                .to_owned()
        })
        .collect()
}

/// The `kaya/...` node ids a rendered publication block cites, sorted and
/// deduplicated so they compare as a set.
fn block_source_ids(page: &str, block: &str) -> Vec<String> {
    let begin = format!("kaya:begin generated:{block}");
    let end = format!("kaya:end generated:{block}");
    let start = page
        .find(&begin)
        .unwrap_or_else(|| panic!("the page carries the {block} block: {page}"));
    let stop = start
        + page[start..]
            .find(&end)
            .unwrap_or_else(|| panic!("the {block} block is closed: {page}"));
    let body = &page[start..stop];
    source_ids(body)
}

/// The `kaya/...` node ids named in `text`, sorted and deduplicated.
///
/// `@` stays inside the token so a Source reference keeps its `@commit`
/// suffix: `kaya/Source/src-main-rs@9f2c1a` is one id, never the bare path.
fn source_ids(text: &str) -> Vec<String> {
    let mut ids: Vec<String> = text
        .split(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '/' | '-' | '_' | '.' | '@')))
        .filter(|token| token.starts_with("kaya/"))
        .map(str::to_owned)
        .collect();
    ids.sort();
    ids.dedup();
    ids
}

/// The `### <kind>` sections of a rendered product block body, in page order.
///
/// A rendered document may carry its own `###` subheadings, so only a heading
/// naming one of `kinds` starts a new section; every other line stays content.
fn product_sections(body: &str, kinds: &[&str]) -> Vec<(String, String)> {
    let mut sections: Vec<(String, String)> = Vec::new();
    for line in body.lines() {
        if let Some(heading) = line.strip_prefix("### ") {
            let heading = heading.trim();
            if kinds.contains(&heading) {
                sections.push((heading.to_owned(), String::new()));
                continue;
            }
        }
        if let Some((_, buffer)) = sections.last_mut() {
            buffer.push_str(line);
            buffer.push('\n');
        }
    }
    sections
}

#[test]
fn build_writes_overview_and_publish_uses_it() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    // One Question investigated by one Experiment: the seeded edge makes
    // `build` render the research map, and the Question backs the project
    // overview. A run and its captured artifact exist but must never enter
    // the overview.
    write_ledger(
        &dir,
        &[
            event_line(&node_event(
                "event-question",
                "Question",
                "kaya/Question/q-one",
                "Q One",
            )),
            event_line(&experiment_event(
                "event-experiment",
                "kaya/Experiment/exp-one",
                "Exp One",
            )),
            event_line(&relationship_event(
                "event-relationship",
                "investigates",
                "kaya/Question/q-one",
                "kaya/Experiment/exp-one",
            )),
            event_line(&run_event("run-1", "kaya/Run/run-1", 3)),
            event_line(&artifact_event(
                "artifact-1",
                "kaya/Artifact/run-jsonl",
                "reports",
                "target/run.jsonl",
            )),
        ],
    );

    let built = assert_success(&run(&dir, &["build", "--allow-uncommitted"]));
    let recomputed = built["data"]["recomputed"]
        .as_array()
        .expect("recomputed is a list");
    for name in ["orientation.research-map", "orientation.project-overview"] {
        assert!(
            recomputed.iter().any(|candidate| candidate == name),
            "build compiles the {name} block: {recomputed:?}"
        );
    }

    let overview = fs::read_to_string(dir.path().join("docs/research/overview.md"))
        .expect("the project overview is written");
    let overview_body = generated_block_body(&overview, "orientation.project-overview");
    assert!(
        overview_body.contains("### Questions"),
        "the overview renders its Questions section: {overview_body}"
    );
    assert!(
        overview_body.contains("- kaya/Question/q-one — Q One"),
        "the overview renders the recorded question: {overview_body}"
    );
    for forbidden in [
        "kaya/Run/run-1",
        "### Runs",
        "kaya/Artifact/run-jsonl",
        "target/run.jsonl",
        "### Artifacts",
    ] {
        assert!(
            !overview_body.contains(forbidden),
            "run and artifact rows stay out of the overview ({forbidden}): {overview_body}"
        );
    }

    let sources = built["data"]["sources"]
        .as_object()
        .expect("build exposes the merged source map");
    assert_eq!(
        sources["orientation.project-overview"],
        json!(["kaya/Question/q-one"]),
        "the overview source map names exactly the rendered question: {sources:?}"
    );
    let omitted = built["data"]["omitted"]
        .as_array()
        .expect("omitted is a list");
    assert!(
        !omitted.iter().any(|entry| {
            entry["family"] == json!("orientation") && entry["kind"] == json!("project-overview")
        }),
        "a question backs the overview, so the kind is not omitted: {omitted:?}"
    );

    let index = fs::read_to_string(dir.path().join(INDEX)).expect("the research map is written");
    let map_row = index
        .lines()
        .find(|line| line.contains("-[investigates]->"))
        .expect("the research map renders the seeded edge")
        .to_owned();

    let ledger = dir.path().join(LEDGER);
    let bytes_before = fs::read(&ledger).expect("the ledger is readable");
    let lines_before = ledger_lines(&dir).len();

    let published = assert_success(&run(&dir, &["publish"]));
    assert_eq!(published["envelope_kind"], json!("publish"));
    assert_eq!(
        published["data"]["out"],
        json!("docs/public"),
        "publish defaults to docs/public: {published}"
    );
    // The question backs the project overview and the research map, so the
    // products admitting either emit; the understanding, reuse, and experiment
    // products have no compiled document and are reported omitted.
    assert_eq!(
        published["data"]["omitted"],
        json!([
            { "tier": "product", "name": "concept", "reason": "no-backing-document" },
            { "tier": "product", "name": "example", "reason": "no-backing-document" },
            { "tier": "product", "name": "faq", "reason": "no-backing-document" },
            { "tier": "product", "name": "how-to", "reason": "no-backing-document" },
            { "tier": "product", "name": "troubleshooting", "reason": "no-backing-document" },
        ]),
        "every declared product without an admissible compiled document is omitted, sorted by name: {published}"
    );
    let pages = published_page_names(&published["data"]["pages"]);
    assert_eq!(
        pages,
        [
            "architecture-design",
            "developer-guide",
            "getting-started",
            "glossary",
            "overview",
            "project-site",
            "reference",
            "release-migration",
            "status-compatibility",
        ],
        "the declared packages and every backed product get a page, sorted by name: {pages:?}"
    );

    let overview_page = fs::read_to_string(dir.path().join("docs/public/overview.md"))
        .expect("the overview page is written");
    assert!(
        overview_page.contains("generated:publication.product.overview"),
        "the overview page carries its generated block: {overview_page}"
    );
    // The project overview body is rendered once, on the first page that admits
    // it in page order; the overview product cross-references that page.
    assert!(
        overview_page
            .contains("See [getting-started](getting-started.md#orientationproject-overview)."),
        "the overview page cross-references the page that renders the shared body: {overview_page}"
    );
    assert!(
        !overview_page.contains("- kaya/Question/q-one — Q One"),
        "the shared overview body is not rendered twice: {overview_page}"
    );
    let getting_started = fs::read_to_string(dir.path().join("docs/public/getting-started.md"))
        .expect("the getting-started page is written");
    assert!(
        getting_started.contains("generated:publication.product.getting-started"),
        "the getting-started page carries its generated block: {getting_started}"
    );
    assert!(
        getting_started.contains("- kaya/Question/q-one — Q One"),
        "the project overview body is rendered on the first page that admits it: {getting_started}"
    );

    let reference = fs::read_to_string(dir.path().join("docs/public/reference.md"))
        .expect("the reference page is written");
    assert!(
        reference.contains("generated:publication.product.reference"),
        "the reference page carries its generated block: {reference}"
    );
    // architecture-design sorts first among the pages admitting the research
    // map, so it renders the body; reference cross-references it.
    let architecture = fs::read_to_string(dir.path().join("docs/public/architecture-design.md"))
        .expect("the architecture-design page is written");
    assert!(
        architecture.contains(&map_row),
        "the architecture-design page carries the research map body row {map_row:?}: {architecture}"
    );
    let cross_reference =
        "See [architecture-design](architecture-design.md#orientationresearch-map).";
    assert!(
        reference.contains(cross_reference),
        "the reference page cross-references the page that renders the shared research map: {reference}"
    );

    let package = fs::read_to_string(dir.path().join("docs/public/project-site.md"))
        .expect("the project-site package page is written");
    assert!(
        package.contains("generated:publication.package.project-site"),
        "the package page carries its generated block: {package}"
    );
    assert!(
        package.contains("[overview](overview.md)"),
        "the package links the emitted overview: {package}"
    );
    assert!(
        package.contains("[reference](reference.md)"),
        "an emitted product is linked from the package: {package}"
    );
    assert!(
        !package.contains("overview — unavailable"),
        "the backed overview is linked, never named unavailable: {package}"
    );

    // The developer-guide package names all six members in declaration order;
    // only getting-started is backed here, so the other five stay unavailable
    // markers instead of links to pages publish never wrote.
    let developer_guide = fs::read_to_string(dir.path().join("docs/public/developer-guide.md"))
        .expect("the developer-guide package page is written");
    assert_eq!(
        generated_block_body(&developer_guide, "publication.package.developer-guide")
            .trim_start_matches('\n'),
        concat!(
            "## Developer-guide\n\n",
            "- [getting-started](getting-started.md)\n",
            "- how-to — unavailable\n",
            "- concept — unavailable\n",
            "- example — unavailable\n",
            "- troubleshooting — unavailable\n",
            "- faq — unavailable\n",
        ),
        "the package block lists six members in declaration order, linking the emitted one: {developer_guide}"
    );

    let export: Value = serde_json::from_str(
        &fs::read_to_string(dir.path().join("docs/public/publication.json"))
            .expect("the structured export is written"),
    )
    .expect("the export is JSON");
    let collections = export["collections"]
        .as_array()
        .expect("the export lists collections");
    assert_eq!(
        collections,
        &[json!({ "emitted": true, "name": "glossary" })],
        "the glossary collection is exported with emitted=true: {export}"
    );
    let products = export["products"]
        .as_array()
        .expect("the export lists products");
    let overview_entry = products
        .iter()
        .find(|product| product["name"] == json!("overview"))
        .expect("the export names the emitted overview product");
    let overview_documents = overview_entry["documents"]
        .as_array()
        .expect("the overview product lists its documents");
    let overview_document = overview_documents
        .iter()
        .find(|document| document["kind"] == json!("orientation.project-overview"))
        .expect("the export names the project overview document under the overview product");
    assert_eq!(
        overview_document["event_ids"],
        json!(["kaya/Question/q-one"]),
        "the exported overview attributes the rendered question: {export}"
    );
    let reference_entry = products
        .iter()
        .find(|product| product["name"] == json!("reference"))
        .expect("the export names the emitted reference product");
    let documents = reference_entry["documents"]
        .as_array()
        .expect("the reference product lists its documents");
    let map = documents
        .iter()
        .find(|document| document["kind"] == json!("orientation.research-map"))
        .expect("the export names the research map document");
    let mut exported_ids: Vec<String> = map["event_ids"]
        .as_array()
        .expect("the document lists its event ids")
        .iter()
        .map(|id| {
            id.as_str()
                .expect("every exported event id is a string")
                .to_owned()
        })
        .collect();
    exported_ids.sort();
    exported_ids.dedup();

    let page_ids = block_source_ids(&architecture, "publication.product.architecture-design");
    assert_eq!(
        exported_ids, page_ids,
        "both formats trace to the same source ids: {export}"
    );
    for id in ["kaya/Question/q-one", "kaya/Experiment/exp-one"] {
        assert!(
            exported_ids.iter().any(|candidate| candidate == id),
            "the research map attribution keeps {id}: {exported_ids:?}"
        );
    }

    for (page, block, text) in [
        (
            "reference",
            "publication.product.reference",
            reference.as_str(),
        ),
        (
            "project-site",
            "publication.package.project-site",
            package.as_str(),
        ),
    ] {
        assert_eq!(
            text.matches(&format!("kaya:begin generated:{block}"))
                .count(),
            1,
            "the {page} page carries exactly one begin marker for {block}: {text}"
        );
        assert_eq!(
            text.matches(&format!("kaya:end generated:{block}")).count(),
            1,
            "the {page} page closes its {block} block exactly once: {text}"
        );
    }

    let recomputed = published["data"]["recomputed"]
        .as_array()
        .expect("publish reports the recomputed blocks");
    for name in [
        "publication.product.getting-started",
        "publication.product.overview",
        "publication.product.reference",
        "publication.package.developer-guide",
        "publication.package.project-site",
        "publication.collection.glossary",
        "publication.json",
    ] {
        assert!(
            recomputed.iter().any(|entry| entry == name),
            "a fresh publish recomputes {name}: {recomputed:?}"
        );
    }
    let unchanged = published["data"]["unchanged"]
        .as_array()
        .expect("publish reports the unchanged blocks");
    assert!(
        unchanged.is_empty(),
        "a fresh publish has nothing already current: {published}"
    );

    // The envelope's compiled documents match the export, kind by kind.
    let mut exported_documents: Vec<&Value> = Vec::new();
    for product in products {
        if let Some(documents) = product["documents"].as_array() {
            exported_documents.extend(documents.iter());
        }
    }
    let envelope_documents = published["data"]["documents"]
        .as_array()
        .expect("publish reports the compiled documents");
    assert!(
        !envelope_documents.is_empty(),
        "the compiled documents are reported: {published}"
    );
    for document in envelope_documents {
        let kind = document["kind"]
            .as_str()
            .expect("each compiled document names its kind");
        let ids = document["event_ids"]
            .as_array()
            .expect("each compiled document lists its event ids");
        let exported = exported_documents
            .iter()
            .find(|candidate| candidate["kind"] == json!(kind))
            .unwrap_or_else(|| panic!("the export names compiled document kind {kind}: {export}"));
        assert_eq!(
            ids,
            exported["event_ids"]
                .as_array()
                .expect("the exported document lists its event ids"),
            "the envelope and export agree on the event ids for {kind}"
        );
    }

    // Publishing twice changes no byte and reports the same payload; only the
    // recomputed/unchanged split moves, from all-recomputed to all-unchanged.
    let public = dir.path().join("docs/public");
    let overview_bytes =
        fs::read(public.join("overview.md")).expect("the overview page is readable");
    let reference_bytes =
        fs::read(public.join("reference.md")).expect("the reference page is readable");
    let package_bytes =
        fs::read(public.join("project-site.md")).expect("the package page is readable");
    let export_bytes = fs::read(public.join("publication.json")).expect("the export is readable");

    let second = assert_success(&run(&dir, &["publish"]));
    for key in ["out", "pages", "omitted", "documents"] {
        assert_eq!(
            second["data"][key], published["data"][key],
            "a second publish reports the same {key}: {second}"
        );
    }
    assert_eq!(
        second["data"]["recomputed"],
        json!([]),
        "a second publish recomputes nothing: {second}"
    );
    let unchanged = second["data"]["unchanged"]
        .as_array()
        .expect("a second publish reports the unchanged blocks");
    for name in [
        "publication.product.getting-started",
        "publication.product.overview",
        "publication.product.reference",
        "publication.package.developer-guide",
        "publication.package.project-site",
        "publication.collection.glossary",
        "publication.json",
    ] {
        assert!(
            unchanged.iter().any(|entry| entry == name),
            "a second publish reports {name} unchanged: {unchanged:?}"
        );
    }
    assert_eq!(
        fs::read(public.join("overview.md")).expect("the overview page is readable"),
        overview_bytes,
        "publishing twice leaves the overview page byte-identical"
    );
    assert_eq!(
        fs::read(public.join("reference.md")).expect("the reference page is readable"),
        reference_bytes,
        "publishing twice leaves the reference page byte-identical"
    );
    assert_eq!(
        fs::read(public.join("project-site.md")).expect("the package page is readable"),
        package_bytes,
        "publishing twice leaves the package page byte-identical"
    );
    assert_eq!(
        fs::read(public.join("publication.json")).expect("the export is readable"),
        export_bytes,
        "publishing twice leaves the export byte-identical"
    );

    assert_eq!(
        fs::read(&ledger).expect("the ledger is readable"),
        bytes_before,
        "publishing copies no payload into the ledger"
    );
    assert_eq!(
        ledger_lines(&dir).len(),
        lines_before,
        "publishing creates no ledger entity"
    );
}

/// The glossary collection ships as a rendered page: the same seed and build
/// that back the products, then `publish` writes `docs/public/glossary.md`
/// with its `publication.collection.glossary` managed block and lists the
/// page among the published pages, sorted by name.
#[test]
fn publish_renders_glossary_collection_page() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    write_ledger(
        &dir,
        &[
            event_line(&node_event(
                "event-question",
                "Question",
                "kaya/Question/q-one",
                "Q One",
            )),
            event_line(&experiment_event(
                "event-experiment",
                "kaya/Experiment/exp-one",
                "Exp One",
            )),
            event_line(&relationship_event(
                "event-relationship",
                "investigates",
                "kaya/Question/q-one",
                "kaya/Experiment/exp-one",
            )),
            event_line(&run_event("run-1", "kaya/Run/run-1", 3)),
            event_line(&artifact_event(
                "artifact-1",
                "kaya/Artifact/run-jsonl",
                "reports",
                "target/run.jsonl",
            )),
        ],
    );

    assert_success(&run(&dir, &["build", "--allow-uncommitted"]));

    let published = assert_success(&run(&dir, &["publish"]));
    let recomputed = published["data"]["recomputed"]
        .as_array()
        .expect("publish reports the recomputed blocks");
    assert!(
        recomputed
            .iter()
            .any(|candidate| candidate == "publication.collection.glossary"),
        "publish composes the glossary collection block: {recomputed:?}"
    );
    let pages = published_page_names(&published["data"]["pages"]);
    assert_eq!(
        pages,
        [
            "architecture-design",
            "developer-guide",
            "getting-started",
            "glossary",
            "overview",
            "project-site",
            "reference",
            "release-migration",
            "status-compatibility",
        ],
        "the glossary collection gets a page among the sorted published pages: {pages:?}"
    );

    let glossary = fs::read_to_string(dir.path().join("docs/public/glossary.md"))
        .expect("the glossary page is written");
    assert!(
        !glossary.is_empty(),
        "the glossary page is non-empty: {glossary}"
    );
    assert!(
        glossary.contains("generated:publication.collection.glossary"),
        "the glossary page carries its generated managed block: {glossary}"
    );
}

/// `publish` declares every product this release ships and reports each one
/// without an admissible compiled document as omitted — with its reason, sorted
/// by name — instead of stubbing a page for it.
#[test]
fn publish_declares_every_product_and_reports_justified_omissions() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    // A Question backs the project overview, and one committed Source backs the
    // reproduction package; `build` also renders the research map (one isolated
    // Question row). No understanding or experiment document exists, so those
    // products have no backing.
    write_ledger(
        &dir,
        &[
            event_line(&node_event(
                "event-question",
                "Question",
                "kaya/Question/q-one",
                "Q One",
            )),
            event_line(&source_event(
                "event-source",
                "kaya/Source/src-main-rs@9f2c1a",
                "src/main.rs",
                "9f2c1a",
            )),
        ],
    );
    assert_success(&run(&dir, &["build", "--allow-uncommitted"]));

    let published = assert_success(&run(&dir, &["publish"]));
    assert_eq!(
        published["data"]["out"],
        json!("docs/public"),
        "publish defaults to docs/public: {published}"
    );

    let pages = published_page_names(&published["data"]["pages"]);
    assert_eq!(
        pages,
        [
            "architecture-design",
            "developer-guide",
            "getting-started",
            "glossary",
            "how-to",
            "overview",
            "project-site",
            "reference",
            "release-migration",
            "status-compatibility",
        ],
        "the declared packages and every backed product get a page, sorted by name: {pages:?}"
    );

    assert_eq!(
        published["data"]["omitted"],
        json!([
            { "tier": "product", "name": "concept", "reason": "no-backing-document" },
            { "tier": "product", "name": "example", "reason": "no-backing-document" },
            { "tier": "product", "name": "faq", "reason": "no-backing-document" },
            { "tier": "product", "name": "troubleshooting", "reason": "no-backing-document" },
        ]),
        "the omitted products are exactly the declared ones without an admissible document, sorted by name: {published}"
    );

    let public = dir.path().join("docs/public");
    for name in ["concept", "example", "faq", "troubleshooting"] {
        assert!(
            !public.join(format!("{name}.md")).exists(),
            "an omitted product is never stubbed into a page: {name}"
        );
    }
    for name in [
        "developer-guide",
        "getting-started",
        "how-to",
        "overview",
        "project-site",
        "reference",
    ] {
        assert!(
            public.join(format!("{name}.md")).exists(),
            "the emitted {name} page is written"
        );
    }

    // getting-started is the first page admitting both of its documents, so it
    // composes the project overview and the reproduction package in full.
    let getting_started = fs::read_to_string(public.join("getting-started.md"))
        .expect("the getting-started page is written");
    let getting_started_block =
        generated_block_body(&getting_started, "publication.product.getting-started");
    for section in [
        "### orientation.project-overview",
        "### reuse-handoff.reproduction-package",
    ] {
        assert!(
            getting_started_block.contains(section),
            "getting-started composes both admitted kinds ({section}): {getting_started}"
        );
    }
    assert!(
        getting_started_block.contains("- kaya/Question/q-one — Q One"),
        "the project overview body is rendered on the first page that admits it: {getting_started}"
    );
    assert!(
        getting_started_block.contains("- kaya/Source/src-main-rs@9f2c1a — src/main.rs@9f2c1a"),
        "the reproduction package body names the committed source: {getting_started}"
    );

    // how-to admits the technical reference and the reproduction package; only
    // the latter exists, and getting-started already owns it, so how-to
    // cross-references it by its generated heading anchor.
    let how_to = fs::read_to_string(public.join("how-to.md")).expect("the how-to page is written");
    assert!(
        how_to.contains(
            "See [getting-started](getting-started.md#reuse-handoffreproduction-package)."
        ),
        "how-to cross-references the page rendering the shared reproduction package: {how_to}"
    );
    assert!(
        !how_to.contains("- kaya/Source/src-main-rs@9f2c1a — src/main.rs@9f2c1a"),
        "the reproduction body is not duplicated on how-to: {how_to}"
    );
    assert!(
        !how_to.contains("### reuse-handoff.technical-reference"),
        "how-to composes no section for a document the ledger never produced: {how_to}"
    );

    let overview =
        fs::read_to_string(public.join("overview.md")).expect("the overview page is written");
    assert!(
        overview.contains("See [getting-started](getting-started.md#orientationproject-overview)."),
        "the overview product cross-references the shared body: {overview}"
    );
    assert!(
        !overview.contains("- kaya/Question/q-one — Q One"),
        "the shared body is not duplicated across pages: {overview}"
    );

    // The developer-guide package names all six members in declaration order,
    // linking exactly the two products the compiled documents back.
    let developer_guide = fs::read_to_string(public.join("developer-guide.md"))
        .expect("the developer-guide package page is written");
    assert_eq!(
        generated_block_body(&developer_guide, "publication.package.developer-guide")
            .trim_start_matches('\n'),
        concat!(
            "## Developer-guide\n\n",
            "- [getting-started](getting-started.md)\n",
            "- [how-to](how-to.md)\n",
            "- concept — unavailable\n",
            "- example — unavailable\n",
            "- troubleshooting — unavailable\n",
            "- faq — unavailable\n",
        ),
        "the package block lists six members in declaration order, linking the emitted two: {developer_guide}"
    );
}

/// `publish` declares the three remaining reader-question products, and the
/// structured export attributes every rendered document to the same event ids
/// its Markdown section cites; a second publish changes no byte.
#[test]
fn publish_declares_the_remaining_products_and_export_agrees() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    // A Question backs the project overview and the research map, and one
    // committed Source backs the reproduction package, so each remaining
    // product has at least one admissible compiled document.
    write_ledger(
        &dir,
        &[
            event_line(&node_event(
                "event-question",
                "Question",
                "kaya/Question/q-one",
                "Q One",
            )),
            event_line(&source_event(
                "event-source",
                "kaya/Source/src-main-rs@9f2c1a",
                "src/main.rs",
                "9f2c1a",
            )),
        ],
    );
    assert_success(&run(&dir, &["build", "--allow-uncommitted"]));

    let published = assert_success(&run(&dir, &["publish"]));
    let pages = published_page_names(&published["data"]["pages"]);
    assert_eq!(
        pages,
        [
            "architecture-design",
            "developer-guide",
            "getting-started",
            "glossary",
            "how-to",
            "overview",
            "project-site",
            "reference",
            "release-migration",
            "status-compatibility",
        ],
        "the remaining products emit beside the already-declared ones, sorted by name: {pages:?}"
    );
    assert_eq!(
        published["data"]["omitted"],
        json!([
            { "tier": "product", "name": "concept", "reason": "no-backing-document" },
            { "tier": "product", "name": "example", "reason": "no-backing-document" },
            { "tier": "product", "name": "faq", "reason": "no-backing-document" },
            { "tier": "product", "name": "troubleshooting", "reason": "no-backing-document" },
        ]),
        "every remaining product is backed, so only the unbacked products are omitted: {published}"
    );

    let public = dir.path().join("docs/public");
    let export: Value = serde_json::from_str(
        &fs::read_to_string(public.join("publication.json")).expect("the export is written"),
    )
    .expect("the export is JSON");
    let products = export["products"]
        .as_array()
        .expect("the export lists products");
    assert_eq!(
        products.len(),
        11,
        "the export names every declared product, emitted or not: {export}"
    );

    let mut attributed: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut rendered: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for product in products {
        let name = product["name"].as_str().expect("each product names itself");
        let documents = product["documents"]
            .as_array()
            .expect("each product lists its documents");
        if product["emitted"] == json!(false) {
            assert!(
                documents.is_empty(),
                "an omitted product attributes no document: {product}"
            );
            continue;
        }
        assert!(
            !documents.is_empty(),
            "an emitted product attributes at least one document: {product}"
        );

        let mut exported: BTreeMap<&str, Vec<String>> = BTreeMap::new();
        for document in documents {
            let kind = document["kind"]
                .as_str()
                .expect("each document names its kind");
            let ids: Vec<String> = document["event_ids"]
                .as_array()
                .expect("each document lists its event ids")
                .iter()
                .map(|id| {
                    id.as_str()
                        .expect("every exported event id is a string")
                        .to_owned()
                })
                .collect();
            assert!(
                !ids.is_empty(),
                "{name} attributes {kind} to at least one event id"
            );
            let existing = attributed
                .entry(kind.to_owned())
                .or_insert_with(|| ids.clone());
            assert_eq!(
                existing.as_slice(),
                ids.as_slice(),
                "every product composing {kind} attributes the same event ids"
            );
            exported.insert(kind, ids);
        }

        let page = match fs::read_to_string(public.join(format!("{name}.md"))) {
            Ok(page) => page,
            Err(error) => panic!("the {name} page is written: {error}"),
        };
        let block = format!("publication.product.{name}");
        let exported_kinds: Vec<&str> = documents
            .iter()
            .map(|document| {
                document["kind"]
                    .as_str()
                    .expect("each document names its kind")
            })
            .collect();
        let sections = product_sections(&generated_block_body(&page, &block), admission(name));
        let kinds: Vec<String> = sections.iter().map(|(kind, _)| kind.clone()).collect();
        assert_eq!(
            kinds, exported_kinds,
            "the {name} page names exactly the documents the export attributes to it, in declaration order"
        );

        for (kind, section) in sections {
            let ids = source_ids(&section);
            if ids.is_empty() {
                assert!(
                    section.trim_start().starts_with("See ["),
                    "the {name} section for {kind} cross-references the page rendering it: {section}"
                );
                continue;
            }
            // The assertion above establishes every rendered section names an
            // exported document, so this row lookup cannot miss.
            let expected = &exported[kind.as_str()];
            assert_eq!(
                ids.as_slice(),
                expected.as_slice(),
                "the {name} section for {kind} cites exactly its own product's exported event ids"
            );
            assert!(
                rendered.insert(kind.clone(), ids).is_none(),
                "exactly one page renders {kind}, so its facts appear once"
            );
        }
    }

    assert_eq!(
        rendered, attributed,
        "the pages and the export trace every document to the same event ids"
    );
    assert_eq!(
        rendered.keys().cloned().collect::<Vec<_>>(),
        [
            "orientation.project-overview",
            "orientation.research-map",
            "reuse-handoff.reproduction-package",
        ],
        "the fixture compiles exactly three documents: {rendered:?}"
    );

    // A second publish from unchanged inputs rewrites nothing: every page and
    // the export keep the bytes the first publish wrote.
    let files: Vec<String> = pages
        .iter()
        .map(|name| format!("{name}.md"))
        .chain(std::iter::once("publication.json".to_owned()))
        .collect();
    let mut before: Vec<Vec<u8>> = Vec::new();
    for file in &files {
        before.push(fs::read(public.join(file)).expect("the published file is readable"));
    }

    let second = assert_success(&run(&dir, &["publish"]));
    assert_eq!(
        second["data"]["recomputed"],
        json!([]),
        "a second publish recomputes nothing: {second}"
    );
    assert_eq!(
        second["data"]["pages"], published["data"]["pages"],
        "a second publish reports the same pages: {second}"
    );
    assert_eq!(
        second["data"]["omitted"], published["data"]["omitted"],
        "a second publish reports the same omissions: {second}"
    );
    for (file, bytes) in files.iter().zip(before) {
        assert_eq!(
            fs::read(public.join(file)).expect("the published file is readable"),
            bytes,
            "a second publish leaves {file} byte-identical"
        );
    }
}

/// An omitted product owns only its managed block: a document at that path
/// without the block keeps every byte, while a page whose only content is the
/// block is removed once the product loses its backing document.
#[test]
fn publish_preserves_authored_pages_at_omitted_product_paths() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    write_ledger(
        &dir,
        &[event_line(&node_event(
            "event-question",
            "Question",
            "kaya/Question/q-one",
            "Q One",
        ))],
    );
    assert_success(&run(&dir, &["build", "--allow-uncommitted"]));

    // `how-to` has no compiled document, so publish treats its page as omitted
    // from the first run; the file is authored, not generated, and no publish
    // may touch it.
    let public = dir.path().join("docs/public");
    fs::create_dir_all(&public).expect("public directory");
    let authored =
        "# Working notes\n\nA human wrote this page; publish owns only its own blocks.\n";
    fs::write(public.join("how-to.md"), authored).expect("the authored page is writable");
    let authored_bytes = fs::read(public.join("how-to.md")).expect("the authored page is readable");

    let first = assert_success(&run(&dir, &["publish"]));
    // Every product admitting the project overview is backed by it, since no
    // other compiled document is present to back them.
    let overview_backed = [
        "getting-started",
        "overview",
        "release-migration",
        "status-compatibility",
    ];
    for name in overview_backed {
        assert!(
            public.join(format!("{name}.md")).exists(),
            "the {name} page backed by the project overview is written: {first}"
        );
    }
    assert_eq!(
        fs::read(public.join("how-to.md")).expect("the authored page is still readable"),
        authored_bytes,
        "an omitted product leaves a document carrying no managed block byte-identical"
    );

    // The project overview stops compiling, so every product that admitted it
    // is newly omitted; their pages hold nothing but the managed block.
    fs::remove_file(dir.path().join("docs/research/overview.md"))
        .expect("the compiled overview document is removable");
    let second = assert_success(&run(&dir, &["publish"]));
    for name in overview_backed {
        assert!(
            !public.join(format!("{name}.md")).exists(),
            "a newly omitted product removes the page that held only its block: {name}"
        );
    }
    assert_eq!(
        fs::read(public.join("how-to.md")).expect("the authored page is still readable"),
        authored_bytes,
        "a later publish still leaves the authored page byte-identical"
    );
    assert_eq!(
        second["data"]["pages"]
            .as_array()
            .expect("pages is a list")
            .len(),
        5,
        "only the packages, the research-map-backed reference and the always-emitted glossary remain: {second}"
    );
    assert_eq!(
        second["data"]["omitted"],
        json!([
            { "tier": "product", "name": "concept", "reason": "no-backing-document" },
            { "tier": "product", "name": "example", "reason": "no-backing-document" },
            { "tier": "product", "name": "faq", "reason": "no-backing-document" },
            { "tier": "product", "name": "getting-started", "reason": "no-backing-document" },
            { "tier": "product", "name": "how-to", "reason": "no-backing-document" },
            { "tier": "product", "name": "overview", "reason": "no-backing-document" },
            { "tier": "product", "name": "release-migration", "reason": "no-backing-document" },
            { "tier": "product", "name": "status-compatibility", "reason": "no-backing-document" },
            { "tier": "product", "name": "troubleshooting", "reason": "no-backing-document" },
        ]),
        "the products whose backing document disappeared are reported omitted: {second}"
    );
}

#[test]
fn unknown_subcommand_returns_the_usage_envelope() {
    let dir = tempfile::tempdir().unwrap();

    let envelope = assert_failure(&run(&dir, &["frobnicate"]));
    assert_eq!(envelope["error"]["label"], json!("usage"));
    let message = envelope["error"]["message"]
        .as_str()
        .expect("error envelopes carry a message");
    assert!(
        message.contains("frobnicate"),
        "usage message must name the offending subcommand: {message}"
    );
}

#[test]
fn build_omits_exploration_brief_without_question_records() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    // A run alone backs no exploration brief: the kind is reported omitted.
    write_ledger(&dir, &[event_line(&run_event("run-1", "kaya/Run/run-1", 3))]);

    let built = assert_success(&run(&dir, &["build", "--allow-uncommitted"]));
    let omitted = built["data"]["omitted"]
        .as_array()
        .expect("omitted is a list");
    assert!(
        omitted.iter().any(|entry| {
            entry == &json!({
                "family": "exploration",
                "kind": "exploration-brief",
                "reason": "no-records",
            })
        }),
        "a ledger without question records omits the exploration brief: {omitted:?}"
    );
}

#[test]
fn build_renders_exploration_brief_from_question_idea_hypothesis() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    write_ledger(
        &dir,
        &[
            event_line(&node_event(
                "event-question",
                "Question",
                "kaya/Question/q-1",
                "Q One",
            )),
            event_line(&node_event(
                "event-idea",
                "Idea",
                "kaya/Idea/i-1",
                "Idea One",
            )),
            event_line(&node_event(
                "event-hypothesis",
                "Hypothesis",
                "kaya/Hypothesis/h-1",
                "Hypothesis One",
            )),
        ],
    );

    let built = assert_success(&run(&dir, &["build", "--allow-uncommitted"]));
    let brief = fs::read_to_string(dir.path().join("docs/research/brief.md"))
        .expect("the exploration brief is written next to the project documents");
    let marker = format!("<!-- kaya:begin generated:{BRIEF_BLOCK} -->");
    assert!(
        brief.contains(&marker),
        "the brief carries the generated exploration block marker: {brief}"
    );
    for (section, id) in [
        ("## Question", "kaya/Question/q-1"),
        ("## Idea", "kaya/Idea/i-1"),
        ("## Hypothesis", "kaya/Hypothesis/h-1"),
    ] {
        let start = brief
            .find(section)
            .unwrap_or_else(|| panic!("the brief carries a {section} section: {brief}"));
        assert!(
            brief[start..].contains(id),
            "the {section} section names {id}: {brief}"
        );
    }

    let omitted = built["data"]["omitted"]
        .as_array()
        .expect("omitted is a list");
    assert!(
        !omitted.iter().any(|entry| {
            entry["family"] == json!("exploration") && entry["kind"] == json!("exploration-brief")
        }),
        "records back the brief, so the kind is never reported omitted: {omitted:?}"
    );
}

#[test]
fn stale_brief_without_records_is_removed() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    write_ledger(
        &dir,
        &[event_line(&node_event(
            "event-question",
            "Question",
            "kaya/Question/q-1",
            "Q One",
        ))],
    );
    assert_success(&run(&dir, &["build", "--allow-uncommitted"]));
    let brief_path = dir.path().join("docs/research/brief.md");
    assert!(
        brief_path.exists(),
        "the first build renders the exploration brief"
    );

    // write_ledger replaces the file, so the records the brief was built
    // from are gone: the stale brief must be removed, not left behind.
    write_ledger(&dir, &[event_line(&run_event("run-1", "kaya/Run/run-1", 3))]);
    assert_success(&run(&dir, &["build", "--allow-uncommitted"]));
    assert!(
        !brief_path.exists(),
        "a brief without backing records is removed by the rebuild"
    );
}
