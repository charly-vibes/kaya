//! Integration tests for the MVP-1 CLI skeleton: the init/status envelopes.

use assert_cmd::Command;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs;
use tempfile::TempDir;

const LEDGER: &str = ".kaya/ledger.jsonl";
const EMPTY_SHA256: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

fn kaya(dir: &TempDir) -> Command {
    let mut command = Command::cargo_bin("kaya").expect("kaya binary is built");
    command.current_dir(dir.path());
    command
}

fn parse_envelope(output: &std::process::Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|err| {
        panic!(
            "expected one JSON envelope on stdout ({err}); stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

fn assert_success_envelope(envelope: &Value) {
    assert_eq!(envelope["ok"], json!(true));
    assert_eq!(envelope["envelope_version"], json!("0.1"));
    assert_eq!(envelope["cli_version"], json!(env!("CARGO_PKG_VERSION")));
    assert_eq!(envelope["warnings"], json!([]));
    assert_eq!(envelope["hints"], json!([]));
    assert_eq!(envelope["meta"], json!({}));
    assert!(
        !envelope
            .as_object()
            .expect("envelope is a JSON object")
            .contains_key("error"),
        "success envelopes omit the error field"
    );
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[test]
fn init_creates_empty_ledger() {
    let dir = tempfile::tempdir().unwrap();

    let output = kaya(&dir).arg("init").output().unwrap();
    assert!(
        output.status.success(),
        "init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let envelope = parse_envelope(&output);
    assert_success_envelope(&envelope);
    assert_eq!(envelope["envelope_kind"], json!("init"));
    assert_eq!(envelope["data"]["ledger"], json!(LEDGER));
    assert_eq!(envelope["data"]["created"], json!(true));

    let ledger_path = dir.path().join(LEDGER);
    assert!(ledger_path.is_file(), "init must create {LEDGER}");
    assert!(
        fs::read(&ledger_path).unwrap().is_empty(),
        "init must create an empty ledger"
    );
}

#[test]
fn init_twice_is_idempotent_and_never_truncates() {
    let dir = tempfile::tempdir().unwrap();
    let ledger_path = dir.path().join(LEDGER);

    let first = kaya(&dir).arg("init").output().unwrap();
    assert!(first.status.success());
    assert_eq!(parse_envelope(&first)["data"]["created"], json!(true));

    let stored = b"{\"id\":\"evt-1\",\"schema\":\"kaya.event/v0\"}\n";
    fs::write(&ledger_path, stored).unwrap();

    let second = kaya(&dir).arg("init").output().unwrap();
    assert!(
        second.status.success(),
        "second init failed: {}",
        String::from_utf8_lossy(&second.stderr)
    );

    let envelope = parse_envelope(&second);
    assert_success_envelope(&envelope);
    assert_eq!(envelope["envelope_kind"], json!("init"));
    assert_eq!(envelope["data"]["created"], json!(false));
    assert_eq!(
        fs::read(&ledger_path).unwrap().as_slice(),
        stored.as_slice(),
        "init must leave an existing ledger byte-identical"
    );
}

#[test]
fn status_reports_empty_ledger() {
    let dir = tempfile::tempdir().unwrap();
    assert!(kaya(&dir).arg("init").output().unwrap().status.success());

    let output = kaya(&dir).arg("status").output().unwrap();
    assert!(
        output.status.success(),
        "status failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let envelope = parse_envelope(&output);
    assert_success_envelope(&envelope);
    assert_eq!(envelope["envelope_kind"], json!("status"));
    assert_eq!(envelope["data"]["ledger"], json!(LEDGER));
    assert_eq!(envelope["data"]["events"], json!(0));
    assert_eq!(envelope["data"]["bytes"], json!(0));
    assert_eq!(envelope["data"]["digest"], json!(EMPTY_SHA256));
    assert_eq!(envelope["data"]["replay_integrity"], json!("ok"));
}

#[test]
fn status_counts_events_and_digests_exact_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let ledger_path = dir.path().join(LEDGER);
    fs::create_dir_all(ledger_path.parent().unwrap()).unwrap();

    let first = json!({ "id": "evt-1", "schema": "kaya.event/v0" });
    let second = json!({ "id": "evt-2", "schema": "kaya.event/v0" });
    let body = format!(
        "{}\n{}\n",
        serde_json::to_string(&first).unwrap(),
        serde_json::to_string(&second).unwrap()
    );
    fs::write(&ledger_path, &body).unwrap();

    let output = kaya(&dir).arg("status").output().unwrap();
    assert!(
        output.status.success(),
        "status failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let envelope = parse_envelope(&output);
    assert_success_envelope(&envelope);
    assert_eq!(envelope["envelope_kind"], json!("status"));
    assert_eq!(envelope["data"]["events"], json!(2));
    assert_eq!(envelope["data"]["bytes"], json!(body.len() as u64));
    assert_eq!(
        envelope["data"]["digest"],
        json!(sha256_hex(body.as_bytes()))
    );
    assert_eq!(envelope["data"]["replay_integrity"], json!("ok"));
}

#[test]
fn status_flags_a_truncated_tail() {
    let dir = tempfile::tempdir().unwrap();
    let ledger_path = dir.path().join(LEDGER);
    fs::create_dir_all(ledger_path.parent().unwrap()).unwrap();

    let complete = format!(
        "{}\n",
        serde_json::to_string(&json!({ "id": "evt-1", "schema": "kaya.event/v0" })).unwrap()
    );
    let truncated = "{\"id\":\"evt-2\"";
    let body = format!("{complete}{truncated}");
    fs::write(&ledger_path, &body).unwrap();

    let output = kaya(&dir).arg("status").output().unwrap();
    assert!(
        output.status.success(),
        "status failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let envelope = parse_envelope(&output);
    assert_success_envelope(&envelope);
    assert_eq!(
        envelope["data"]["events"],
        json!(1),
        "only the complete event line counts"
    );
    assert_eq!(envelope["data"]["bytes"], json!(body.len() as u64));
    assert_eq!(
        envelope["data"]["digest"],
        json!(sha256_hex(body.as_bytes()))
    );
    assert_eq!(
        envelope["data"]["replay_integrity"],
        json!("recovered-truncated-tail")
    );
}

#[test]
fn status_without_ledger_is_a_labeled_error() {
    let dir = tempfile::tempdir().unwrap();

    let output = kaya(&dir).arg("status").output().unwrap();
    assert_eq!(output.status.code(), Some(1));

    let envelope = parse_envelope(&output);
    assert_eq!(envelope["ok"], json!(false));
    assert_eq!(envelope["envelope_kind"], json!("error"));
    assert_eq!(envelope["data"], Value::Null);

    assert_eq!(
        envelope["error"]["label"],
        json!("missing-ledger"),
        "an absent ledger is the missing-ledger failure class"
    );

    let message = envelope["error"]["message"]
        .as_str()
        .expect("error envelopes carry a message");
    assert!(
        message.contains(LEDGER),
        "error message must name {LEDGER}: {message}"
    );
    assert!(
        message.contains("kaya init"),
        "error message must suggest `kaya init`: {message}"
    );
}

#[test]
fn init_refuses_a_directory_at_the_ledger_path() {
    let dir = tempfile::tempdir().unwrap();
    let ledger_path = dir.path().join(LEDGER);
    fs::create_dir_all(&ledger_path).unwrap();

    let output = kaya(&dir).arg("init").output().unwrap();
    assert_eq!(output.status.code(), Some(1));

    let envelope = parse_envelope(&output);
    assert_eq!(envelope["ok"], json!(false));
    assert_eq!(envelope["envelope_kind"], json!("error"));
    assert_eq!(envelope["data"], Value::Null);
    assert_eq!(
        envelope["error"]["label"],
        json!("not-a-file"),
        "a non-file shadowing the ledger is the not-a-file failure class"
    );

    let message = envelope["error"]["message"]
        .as_str()
        .expect("error envelopes carry a message");
    assert!(
        message.contains(LEDGER),
        "error message must name {LEDGER}: {message}"
    );
    assert!(
        message.contains("not a regular ledger file"),
        "error message must say the path is not a regular ledger file: {message}"
    );
    assert!(
        ledger_path.is_dir(),
        "init must never replace a non-file with a ledger"
    );
}

#[test]
fn unknown_subcommand_is_a_labeled_usage_error() {
    let dir = tempfile::tempdir().unwrap();

    let output = kaya(&dir).arg("frobnicate").output().unwrap();
    assert_eq!(output.status.code(), Some(1));

    let envelope = parse_envelope(&output);
    assert_eq!(envelope["ok"], json!(false));
    assert_eq!(envelope["envelope_kind"], json!("error"));
    assert_eq!(envelope["data"], Value::Null);
    assert_eq!(
        envelope["error"]["label"],
        json!("usage"),
        "argparse failures are the usage failure class"
    );

    let message = envelope["error"]["message"]
        .as_str()
        .expect("error envelopes carry a message");
    assert!(
        message.contains("frobnicate"),
        "usage message must name the offending subcommand: {message}"
    );
}

#[test]
fn version_json_prints_version_envelope() {
    let dir = tempfile::tempdir().unwrap();

    let output = kaya(&dir).arg("--version").arg("--json").output().unwrap();
    assert_eq!(output.status.code(), Some(0));

    let envelope = parse_envelope(&output);
    assert_eq!(envelope["ok"], json!(true));
    assert_eq!(envelope["envelope_kind"], json!("version"));
    assert_eq!(envelope["data"]["name"], json!("kaya"));
    assert_eq!(envelope["data"]["version"], json!(env!("CARGO_PKG_VERSION")));
}

#[test]
fn help_and_version_stay_human_readable_and_succeed() {
    let dir = tempfile::tempdir().unwrap();

    let help = kaya(&dir).arg("--help").output().unwrap();
    assert_eq!(help.status.code(), Some(0));
    let help_text = String::from_utf8_lossy(&help.stdout);
    assert!(
        help_text.contains("Usage"),
        "--help must print clap's usage text, not an envelope: {help_text}"
    );

    let version = kaya(&dir).arg("--version").output().unwrap();
    assert_eq!(version.status.code(), Some(0));
    assert!(
        String::from_utf8_lossy(&version.stdout).contains(env!("CARGO_PKG_VERSION")),
        "--version must print the version"
    );
}
