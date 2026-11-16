//! Integration tests for the CLI pipeline: `import`, `build`, `promote`,
//! `challenge`, and the `status` proposal counter.
//!
//! Every case drives the real binary in a tempfile project, writes its own
//! fixture tree, and reads the one JSON envelope each invocation prints.

use assert_cmd::Command;
use serde_json::{Value, json};
use std::fs;
use tempfile::TempDir;

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

fn experiment_event(event_id: &str, node_id: &str, title: &str) -> Value {
    json!({
        "id": event_id,
        "schema": "kaya.event/v0",
        "type": "model.entity.created",
        "timestamp": "2026-11-16T10:00:00Z",
        "actor": { "type": "human", "name": "alice" },
        "subject": { "type": "Experiment", "id": node_id },
        "data": { "attributes": { "title": title } },
        "provenance": { "tool": "kaya", "version": "0.1.0", "reference": event_id },
        "artifacts": [],
    })
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
fn build_writes_index_with_generated_block_and_artifact() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    write_wai_design(&dir, "x.md");
    assert_success(&import_wai(&dir));

    let built = assert_success(&run(&dir, &["build"]));
    assert_eq!(built["envelope_kind"], json!("build"));
    assert_eq!(built["data"]["out"], json!("docs/research"));
    assert_eq!(built["data"]["pending_proposals"], json!(0));
    assert_eq!(built["data"]["stale"], json!([]));
    assert!(
        built["data"]["ledger_digest"].is_string(),
        "build surfaces the ledger digest so callers can pin it: {built}"
    );
    let recomputed = built["data"]["recomputed"]
        .as_array()
        .expect("recomputed is a list");
    assert!(
        recomputed.iter().any(|name| name == "research-index"),
        "a fresh index is recomputed: {recomputed:?}"
    );

    let index = fs::read_to_string(dir.path().join(INDEX)).expect("index exists");
    assert!(
        index.contains("<!-- kaya:begin generated:research-index -->"),
        "index carries the generated block marker: {index}"
    );
    assert!(
        index.contains(".wai/projects/kaya/designs/x.md"),
        "index lists the observed artifact: {index}"
    );

    let again = assert_success(&run(&dir, &["build"]));
    let unchanged = again["data"]["unchanged"]
        .as_array()
        .expect("unchanged is a list");
    assert!(
        unchanged.iter().any(|name| name == "research-index"),
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
    write_ledger(&dir, &[experiment]);

    let built = assert_success(&run(&dir, &["build"]));
    let recomputed = built["data"]["recomputed"]
        .as_array()
        .expect("recomputed is a list");
    assert!(
        recomputed.iter().any(|name| name == "experiment-report"),
        "an experiment report is recomputed: {recomputed:?}"
    );

    let report = fs::read_to_string(dir.path().join("docs/research/experiments/exp-one.md"))
        .expect("the report key is the last id segment");
    assert!(
        report.contains("<!-- kaya:begin generated:experiment-report -->"),
        "report carries the generated block marker: {report}"
    );

    let index = fs::read_to_string(dir.path().join(INDEX)).expect("index exists");
    assert!(
        index.contains("kaya/Experiment/exp-one"),
        "index lists the experiment: {index}"
    );
}

#[test]
fn build_expect_digest_pins_the_ledger() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    write_wai_design(&dir, "x.md");
    assert_success(&import_wai(&dir));

    let built = assert_success(&run(&dir, &["build"]));
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
            .any(|name| name == "research-index"),
        "the pinned rebuild leaves the index alone"
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
