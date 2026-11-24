//! Integration tests for the capture CLI: `kaya note`, `kaya question`,
//! `kaya decision`, and `kaya finding`.
//!
//! Purpose: pin the add-capture-commands contract — one `model.entity.created`
//! event per invocation, secrets redacted before bounds, deterministic ids,
//! edits superseding stored bytes, and only human/agent authorship.
//! Responsibilities: every case drives the real binary in a tempfile project,
//! asserts the one JSON envelope each invocation prints, and reads
//! `.kaya/ledger.jsonl` to verify the stored `kaya.event/v0` record.
//! Rationale: the envelope names the outcome (`appended`, `duplicate`,
//! `corrected`) and the event id; the ledger bytes stay the source of truth.

use assert_cmd::Command;
use kaya::envelope::CLI_VERSION;
use kaya::event::parse_line;
use kaya::ledger::resolve_effective;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs;
use tempfile::TempDir;

const LEDGER: &str = ".kaya/ledger.jsonl";

/// An unlisted 40-character base64 run: secret by the default rule.
const HIGH_ENTROPY_RUN: &str = "Qx7Yw2Vt9Us4Rp3Nm6Kj1Hf8Dd5Gg0Zz3Bb4CcAa";

fn kaya(dir: &TempDir) -> Command {
    let mut command = Command::cargo_bin("kaya").expect("kaya binary is built");
    command.current_dir(dir.path());
    command
}

fn run(dir: &TempDir, args: &[&str]) -> std::process::Output {
    kaya(dir).args(args).output().expect("kaya runs")
}

/// Run a capture subcommand expecting success and the capture envelope kind.
fn capture(dir: &TempDir, args: &[&str]) -> Value {
    let envelope = assert_success(&run(dir, args));
    assert_eq!(
        envelope["envelope_kind"],
        json!("capture"),
        "capture prints one capture envelope"
    );
    envelope
}

/// Run a capture subcommand expecting a labeled refusal envelope.
fn refusal(dir: &TempDir, args: &[&str]) -> Value {
    assert_failure(&run(dir, args))
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

fn init(dir: &TempDir) {
    assert_success(&run(dir, &["init"]));
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

/// The whole ledger file, for secret-absence sweeps.
fn ledger_text(dir: &TempDir) -> String {
    fs::read_to_string(dir.path().join(LEDGER)).expect("ledger is readable")
}

fn stored_event(dir: &TempDir, index: usize) -> Value {
    let lines = ledger_lines(dir);
    serde_json::from_str(&lines[index]).expect("stored ledger line parses")
}

fn event_line(event: &Value) -> String {
    format!(
        "{}\n",
        serde_json::to_string(event).expect("event serializes")
    )
}

/// The closed-vocabulary entity type a capture subcommand records.
fn entity_type_for(subcommand: &str) -> &'static str {
    match subcommand {
        "note" => "Observation",
        "question" => "Question",
        "decision" => "Decision",
        "finding" => "Finding",
        other => panic!("unknown capture subcommand: {other}"),
    }
}

/// Assert an RFC 3339 UTC timestamp at seconds precision.
#[track_caller]
fn assert_rfc3339_utc_seconds(timestamp: &str) {
    let bytes = timestamp.as_bytes();
    assert_eq!(bytes.len(), 20, "RFC 3339 UTC seconds: {timestamp}");
    assert!(
        bytes[4] == b'-'
            && bytes[7] == b'-'
            && bytes[10] == b'T'
            && bytes[13] == b':'
            && bytes[16] == b':'
            && bytes[19] == b'Z',
        "expected RFC 3339 UTC seconds, found {timestamp}"
    );
    for (index, byte) in bytes[..19].iter().enumerate() {
        if matches!(index, 4 | 7 | 10 | 13 | 16) {
            continue;
        }
        assert!(
            byte.is_ascii_digit(),
            "expected digits at {index} in {timestamp}"
        );
    }
}

/// Capture once and assert the full first-capture event shape in the ledger.
///
/// Returns the envelope; the stored event must name the envelope's event id,
/// carry exactly `data: {text, key}`, provenance `kaya` + CLI version +
/// `<subcommand>/<key>`, and the given actor.
#[track_caller]
fn assert_first_capture_shape(
    dir: &TempDir,
    subcommand: &str,
    text: &str,
    key: &str,
    actor: &str,
    actor_type: &str,
) -> Value {
    let mut args = vec![subcommand, text, "--actor", actor, "--key", key];
    if actor_type != "human" {
        args.extend(["--actor-type", actor_type]);
    }
    let envelope = capture(dir, &args);
    assert_eq!(envelope["data"]["outcome"], json!("appended"));

    let lines = ledger_lines(dir);
    assert_eq!(lines.len(), 1, "one capture appends exactly one line");
    let event: Value = serde_json::from_str(&lines[0]).expect("stored ledger line parses");

    let entity = entity_type_for(subcommand);
    let subject_id = format!("kaya/{entity}/{key}");
    assert_eq!(
        event["id"], envelope["data"]["event"],
        "the envelope names the stored event's id"
    );
    assert_eq!(event["schema"], json!("kaya.event/v0"));
    assert_eq!(event["type"], json!("model.entity.created"));
    assert_eq!(
        event["subject"],
        json!({ "type": entity, "id": subject_id }),
        "the subject id is `<project>/<TYPE>/<key>`"
    );
    let data = event["data"].as_object().expect("data is an object");
    assert_eq!(
        data.len(),
        2,
        "a first capture carries exactly the data keys text and key"
    );
    assert_eq!(event["data"]["text"], json!(text));
    assert_eq!(event["data"]["key"], json!(key));
    assert_eq!(
        event["provenance"],
        json!({
            "tool": "kaya",
            "version": CLI_VERSION,
            "reference": format!("{subcommand}/{key}"),
        }),
        "provenance names kaya, the CLI version, and <subcommand>/<key>"
    );
    assert_eq!(
        event["actor"],
        json!({ "type": actor_type, "name": actor }),
        "the actor is carried on the stored event"
    );
    assert_rfc3339_utc_seconds(event["timestamp"].as_str().expect("timestamp"));
    envelope
}

/// The effective `data` for a subject id under the ledger's correction rules.
fn effective_data(dir: &TempDir, subject_id: &str) -> Value {
    let events: Vec<_> = ledger_lines(dir)
        .iter()
        .map(|line| parse_line(line).expect("stored ledger line parses"))
        .collect();
    resolve_effective(&events)
        .into_iter()
        .find(|event| event.subject.id == subject_id)
        .expect("an effective event for the subject id")
        .data
}

#[test]
fn note_appends_an_observation_event() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    assert_first_capture_shape(
        &dir,
        "note",
        "buffers drain on shutdown",
        "drain-behaviour",
        "sasha",
        "human",
    );
    assert_eq!(
        stored_event(&dir, 0)["subject"]["type"],
        json!("Observation"),
        "note captures an Observation subject"
    );
}

#[test]
fn question_appends_a_question_event() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    assert_first_capture_shape(
        &dir,
        "question",
        "does the tail repair preserve digests?",
        "tail-repair",
        "sasha",
        "human",
    );
    assert_eq!(
        stored_event(&dir, 0)["subject"]["type"],
        json!("Question"),
        "question captures a Question subject"
    );
}

#[test]
fn decision_appends_a_decision_event() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    assert_first_capture_shape(
        &dir,
        "decision",
        "keep JSONL, no database",
        "storage",
        "charly",
        "human",
    );
    assert_eq!(
        stored_event(&dir, 0)["subject"]["type"],
        json!("Decision"),
        "decision captures a Decision subject"
    );
}

#[test]
fn finding_appends_a_finding_event() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    assert_first_capture_shape(
        &dir,
        "finding",
        "recovery appends, never truncates complete lines",
        "recovery-appends",
        "charly",
        "human",
    );
    assert_eq!(
        stored_event(&dir, 0)["subject"]["type"],
        json!("Finding"),
        "finding captures a Finding subject"
    );
}

#[test]
fn capture_subject_id_is_namespace_qualified() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    let envelope = capture(
        &dir,
        &[
            "note",
            "buffers drain on shutdown",
            "--actor",
            "sasha",
            "--key",
            "drain-behaviour",
            "--project",
            "kaya",
        ],
    );
    assert_eq!(envelope["data"]["outcome"], json!("appended"));
    let event = stored_event(&dir, 0);
    assert_eq!(
        event["subject"],
        json!({ "type": "Observation", "id": "kaya/Observation/drain-behaviour" }),
        "the subject id is the namespace-qualified node id"
    );
}

#[test]
fn capture_project_namespaces_the_subject_id() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    capture(
        &dir,
        &[
            "decision",
            "keep JSONL, no database",
            "--actor",
            "charly",
            "--key",
            "storage",
            "--project",
            "research",
        ],
    );
    let event = stored_event(&dir, 0);
    assert_eq!(
        event["subject"]["id"],
        json!("research/Decision/storage"),
        "the --project name namespaces the subject id"
    );
    assert_eq!(
        event["provenance"]["reference"],
        json!("decision/storage"),
        "the provenance reference stays ledger-internal"
    );
}

#[test]
fn capture_data_carries_exactly_text_and_key() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    capture(
        &dir,
        &[
            "note",
            "no interpretation keys here",
            "--actor",
            "sasha",
            "--key",
            "plain-note",
        ],
    );
    let event = stored_event(&dir, 0);
    let keys: Vec<&String> = event["data"]
        .as_object()
        .expect("data is an object")
        .keys()
        .collect();
    assert!(
        keys.len() == 2
            && keys.iter().any(|key| key.as_str() == "text")
            && keys.iter().any(|key| key.as_str() == "key"),
        "data carries exactly text and key, found {keys:?}"
    );
}

#[test]
fn capture_provenance_names_kaya_and_the_subcommand() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    capture(
        &dir,
        &[
            "decision",
            "keep JSONL, no database",
            "--actor",
            "charly",
            "--key",
            "storage",
        ],
    );
    let event = stored_event(&dir, 0);
    assert_eq!(
        event["provenance"],
        json!({
            "tool": "kaya",
            "version": CLI_VERSION,
            "reference": "decision/storage",
        }),
        "provenance names the tool, the CLI version, and <subcommand>/<key>"
    );
}

#[test]
fn default_key_is_date_with_content_digest() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    let text = "buffers drain on shutdown";
    let envelope = capture(&dir, &["note", text, "--actor", "sasha"]);
    assert_eq!(envelope["data"]["outcome"], json!("appended"));

    let event = stored_event(&dir, 0);
    let stored_key = event["data"]["key"].as_str().expect("data.key");
    let digest = Sha256::digest(text.as_bytes());
    let hex = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let date = &event["timestamp"].as_str().expect("timestamp")[..10];
    assert_eq!(
        stored_key,
        format!("{date}-{}", &hex[..8]),
        "the default key is <UTC date>-<sha256(redacted text)[:8]>"
    );
    assert!(
        stored_key.len() <= 64
            && stored_key
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-'),
        "the derived key respects the key bounds: {stored_key}"
    );
    assert_eq!(
        event["subject"]["id"],
        json!(format!("kaya/Observation/{stored_key}")),
        "the derived key names the subject id"
    );
    assert_eq!(
        event["provenance"]["reference"],
        json!(format!("note/{stored_key}")),
        "the derived key names the provenance reference"
    );
}

#[test]
fn capture_against_a_missing_ledger_refused() {
    for subcommand in ["note", "question", "decision", "finding"] {
        let dir = tempfile::tempdir().unwrap();
        let envelope = refusal(&dir, &[subcommand, "text", "--actor", "sasha"]);
        assert_eq!(
            envelope["error"]["label"],
            json!("missing-ledger"),
            "{subcommand} refuses a missing ledger"
        );
        assert!(
            envelope["error"]["message"]
                .as_str()
                .expect("error message")
                .contains("kaya init"),
            "the refusal points at `kaya init`"
        );
        assert!(
            !dir.path().join(LEDGER).exists(),
            "{subcommand} never auto-creates the ledger"
        );
    }
}

#[test]
fn unchanged_recapture_reports_duplicate_and_appends_nothing() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    let first = capture(
        &dir,
        &[
            "note",
            "buffers drain on shutdown",
            "--actor",
            "sasha",
            "--key",
            "drain-behaviour",
        ],
    );
    assert_eq!(first["data"]["outcome"], json!("appended"));
    let first_id = first["data"]["event"].as_str().expect("event id");

    let replay = capture(
        &dir,
        &[
            "note",
            "buffers drain on shutdown",
            "--actor",
            "sasha",
            "--key",
            "drain-behaviour",
        ],
    );
    assert_eq!(
        replay["data"]["outcome"],
        json!("duplicate"),
        "an unchanged recapture is a duplicate"
    );
    assert_eq!(
        replay["data"]["event"],
        json!(first_id),
        "the duplicate names the stored event's id"
    );
    assert_eq!(
        ledger_lines(&dir).len(),
        1,
        "an unchanged recapture appends nothing"
    );
}

#[test]
fn redacted_text_hashes_deterministically() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    let text = "rotate the sk-proj-abc123def456ghijkl key tomorrow";
    let args = ["note", text, "--actor", "sasha", "--key", "rotate-key"];

    let first = capture(&dir, &args);
    assert_eq!(first["data"]["outcome"], json!("appended"));
    let first_id = first["data"]["event"]
        .as_str()
        .expect("event id")
        .to_owned();
    let stored = stored_event(&dir, 0);
    assert_eq!(
        stored["data"]["text"],
        json!("rotate the [redacted] key tomorrow"),
        "the secret is redacted before id derivation and storage"
    );

    let replay = capture(&dir, &args);
    assert_eq!(
        replay["data"]["outcome"],
        json!("duplicate"),
        "the same redacted bytes replay as a byte-exact duplicate"
    );
    assert_eq!(
        replay["data"]["event"],
        json!(first_id),
        "both invocations derive the same id from the redacted bytes"
    );
    assert_eq!(ledger_lines(&dir).len(), 1, "the replay appends nothing");
    assert!(
        !ledger_text(&dir).contains("sk-proj-abc123def456ghijkl"),
        "the pasted secret appears nowhere in the ledger"
    );
}

#[test]
fn distinct_captures_never_share_an_id() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    let first = capture(
        &dir,
        &[
            "note",
            "first text",
            "--actor",
            "sasha",
            "--key",
            "first-key",
        ],
    );
    let second = capture(
        &dir,
        &[
            "note",
            "second text",
            "--actor",
            "sasha",
            "--key",
            "second-key",
        ],
    );
    assert_eq!(first["data"]["outcome"], json!("appended"));
    assert_eq!(second["data"]["outcome"], json!("appended"));
    assert_ne!(
        first["data"]["event"], second["data"]["event"],
        "two different captures never share an id"
    );
    assert_eq!(ledger_lines(&dir).len(), 2);
}

#[test]
fn edited_capture_emits_a_superseding_correction() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    let first = capture(
        &dir,
        &[
            "note",
            "first wording",
            "--actor",
            "sasha",
            "--key",
            "drain-behaviour",
        ],
    );
    assert_eq!(first["data"]["outcome"], json!("appended"));
    let first_id = first["data"]["event"]
        .as_str()
        .expect("event id")
        .to_owned();
    let before = ledger_lines(&dir);
    assert_eq!(before.len(), 1);

    let correction = capture(
        &dir,
        &[
            "note",
            "revised wording",
            "--actor",
            "sasha",
            "--key",
            "drain-behaviour",
        ],
    );
    assert_eq!(
        correction["data"]["outcome"],
        json!("corrected"),
        "changed text on an existing key is a correction"
    );

    let after = ledger_lines(&dir);
    assert_eq!(after.len(), 2, "the correction appends exactly one line");
    assert_eq!(
        after[0], before[0],
        "the ledger is append-only: the original line is untouched"
    );
    let correction_event: Value = serde_json::from_str(&after[1]).expect("correction line parses");
    let correction_id = correction_event["id"].as_str().expect("event id");
    assert_ne!(
        correction_id, first_id,
        "a correction mints a fresh id, never reuses the original"
    );
    assert_eq!(
        correction["data"]["event"],
        json!(correction_id),
        "the envelope names the correction's id"
    );
    assert_eq!(
        correction["data"]["supersedes"],
        json!(first_id),
        "the corrected envelope names the event it supersedes"
    );
    assert_eq!(
        correction_event["type"],
        json!("model.entity.created"),
        "a correction keeps the shipped event kind"
    );
    assert_eq!(
        correction_event["subject"],
        json!({ "type": "Observation", "id": "kaya/Observation/drain-behaviour" }),
        "a correction preserves the subject id"
    );
    let data = correction_event["data"]
        .as_object()
        .expect("data is an object");
    assert!(
        data.len() == 3
            && data.contains_key("text")
            && data.contains_key("key")
            && data.contains_key("supersedes"),
        "correction data is exactly text, key, and supersedes"
    );
    assert_eq!(
        correction_event["data"]["supersedes"],
        json!(first_id),
        "the correction names the prior effective event id"
    );
    assert_eq!(
        correction_event["data"]["text"],
        json!("revised wording"),
        "the correction carries the new redacted text"
    );

    let effective = effective_data(&dir, "kaya/Observation/drain-behaviour");
    assert_eq!(
        effective["text"],
        json!("revised wording"),
        "the effective stored text resolves to the correction's text"
    );
    assert!(
        effective.get("supersedes").is_none(),
        "the resolved effective data drops the supersedes instruction"
    );
}

#[test]
fn correction_replay_appends_nothing() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    capture(
        &dir,
        &[
            "note",
            "first wording",
            "--actor",
            "sasha",
            "--key",
            "drain-behaviour",
        ],
    );
    let correction = capture(
        &dir,
        &[
            "note",
            "revised wording",
            "--actor",
            "sasha",
            "--key",
            "drain-behaviour",
        ],
    );
    assert_eq!(correction["data"]["outcome"], json!("corrected"));
    let correction_id = correction["data"]["event"]
        .as_str()
        .expect("event id")
        .to_owned();

    let replay = capture(
        &dir,
        &[
            "note",
            "revised wording",
            "--actor",
            "sasha",
            "--key",
            "drain-behaviour",
        ],
    );
    assert_eq!(
        replay["data"]["outcome"],
        json!("duplicate"),
        "re-running the same correction is a duplicate"
    );
    assert_eq!(
        replay["data"]["event"],
        json!(correction_id),
        "the replay names the already-stored correction's id"
    );
    assert_eq!(
        ledger_lines(&dir).len(),
        2,
        "the correction replay appends nothing"
    );
}

#[test]
fn key_without_stored_text_treated_as_absent() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    // Seed-imported attributes carry `data.attributes`, never `data.text`.
    let seed = json!({
        "id": "01JSEED0000000000000000000",
        "schema": "kaya.event/v0",
        "type": "model.entity.created",
        "timestamp": "2026-11-16T10:00:00Z",
        "actor": { "type": "human", "name": "alice" },
        "subject": { "type": "Observation", "id": "kaya/Observation/seeded" },
        "data": { "attributes": { "title": "seeded attribute" } },
        "provenance": { "tool": "kaya", "version": "0.1.0", "reference": "seed" },
        "artifacts": [],
    });
    write_ledger(&dir, &[event_line(&seed)]);

    let envelope = capture(
        &dir,
        &[
            "note",
            "fresh capture text",
            "--actor",
            "sasha",
            "--key",
            "seeded",
        ],
    );
    assert_eq!(
        envelope["data"]["outcome"],
        json!("appended"),
        "a key whose stored events carry no data.text counts as absent"
    );
    let lines = ledger_lines(&dir);
    assert_eq!(lines.len(), 2, "the first capture appends its own event");
    let event: Value = serde_json::from_str(&lines[1]).expect("capture line parses");
    assert!(
        event["data"].get("supersedes").is_none(),
        "a first capture never carries supersedes"
    );
    assert_eq!(event["data"]["text"], json!("fresh capture text"));
}

#[test]
fn pasted_api_key_replaced_in_place_with_marker() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    let token = "sk-proj-abc123def456ghijkl";
    let text = format!("buffers drain on shutdown {token} then recovery");
    let envelope = capture(
        &dir,
        &[
            "note",
            &text,
            "--actor",
            "sasha",
            "--key",
            "drain-behaviour",
        ],
    );
    assert_eq!(envelope["data"]["outcome"], json!("appended"));
    assert_eq!(
        stored_event(&dir, 0)["data"]["text"],
        json!("buffers drain on shutdown [redacted] then recovery"),
        "the marker replaces the token in place, preserving the surrounding words"
    );
    assert!(
        !ledger_text(&dir).contains(token),
        "the token appears nowhere in the ledger"
    );
}

#[test]
fn high_entropy_run_treated_as_secret() {
    assert_eq!(HIGH_ENTROPY_RUN.len(), 40);
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    let text = format!("the digest blob {HIGH_ENTROPY_RUN} looked machine-made");
    capture(
        &dir,
        &["note", &text, "--actor", "sasha", "--key", "blob-note"],
    );
    assert_eq!(
        stored_event(&dir, 0)["data"]["text"],
        json!("the digest blob [redacted] looked machine-made"),
        "an unlisted base64 run of 40 characters is redacted in place"
    );
    assert!(
        !ledger_text(&dir).contains(HIGH_ENTROPY_RUN),
        "the high-entropy run appears nowhere in the ledger"
    );
}

#[test]
fn benign_text_survives_redaction_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    let text = "kaya.ledger.id_reuse stays unreachable; the password policy has \
                no default; short digest Zz9Yy8Xx7 is not a secret";
    let envelope = capture(
        &dir,
        &["note", text, "--actor", "sasha", "--key", "benign-note"],
    );
    assert_eq!(envelope["data"]["outcome"], json!("appended"));
    assert_eq!(
        stored_event(&dir, 0)["data"]["text"],
        json!(text),
        "benign text — a refused-label name, a valueless password word, a short \
         digest — is stored byte-identically"
    );
}

#[test]
fn redaction_filter_covers_the_inherited_pattern_list() {
    // One representative per inherited family, each exactly the string the
    // family's pattern matches: replaced in place, surroundings untouched.
    let families: &[(&str, &str)] = &[
        ("sk- token", "sk-proj-Zyxwvutsrqponmlkji"),
        ("ghp_ token", "ghp_A1bC2dE3fG4hI5jK6lL7mN8oP9qR0sT1uV2w"),
        ("github_pat_ token", "github_pat_ABCDEFGHIJKLMNOPQRSTUVWX"),
        ("AWS AKIA id", "AKIAIOSFODNN7EXAMPLE"),
        ("AWS ASIA id", "ASIAIOSFODNN7EXAMPLE"),
        // Assembled via concat!: no single source literal matches the Slack
        // token shape (GitHub push protection), while the runtime value is
        // still exactly the string the family's pattern matches.
        (
            "Slack token",
            concat!("xoxb-", "123456789012-abcdefghijklmnop"),
        ),
        (
            "JWT",
            "eyJhbGciOiJIUzI1NiJ9.payloadpart123456.signaturepart654321",
        ),
        (
            "generic private-key armour",
            "-----BEGIN RSA PRIVATE KEY-----",
        ),
        (
            "PGP private-key armour",
            "-----BEGIN PGP PRIVATE KEY BLOCK-----",
        ),
        ("PuTTY key header", "PuTTY-User-Key-File"),
        ("token assignment", "token: 9f8e7d6c5b4a321f"),
    ];
    for (family, secret) in families {
        let dir = tempfile::tempdir().unwrap();
        init(&dir);
        let text = format!("prefix words for {secret} stay visible");
        let envelope = capture(
            &dir,
            &["note", &text, "--actor", "sasha", "--key", "family-probe"],
        );
        assert_eq!(
            envelope["data"]["outcome"],
            json!("appended"),
            "{family} still appends its capture"
        );
        assert_eq!(
            stored_event(&dir, 0)["data"]["text"],
            json!("prefix words for [redacted] stay visible"),
            "{family} is replaced with the marker in place"
        );
        assert!(
            !ledger_text(&dir).contains(secret),
            "{family} appears nowhere in the ledger"
        );
    }
}

#[test]
fn redaction_runs_before_the_bound_check() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    // 1980 filler chars + 1 space + the 40-char secret: 2021 raw characters,
    // but 1991 characters once redacted — under the bound and admitted.
    let filler = "a".repeat(1980);
    let text = format!("{filler} {HIGH_ENTROPY_RUN}");
    assert_eq!(text.chars().count(), 2021, "the raw text is over the bound");
    let envelope = capture(
        &dir,
        &["note", &text, "--actor", "sasha", "--key", "overage-secret"],
    );
    assert_eq!(
        envelope["data"]["outcome"],
        json!("appended"),
        "the bound applies to the redacted text, never the raw paste"
    );
    let stored = stored_event(&dir, 0)["data"]["text"]
        .as_str()
        .expect("data.text")
        .to_owned();
    assert_eq!(
        stored,
        format!("{filler} [redacted]"),
        "the stored text is the redacted bytes"
    );
    assert_eq!(stored.chars().count(), 1991);
    assert!(!ledger_text(&dir).contains(HIGH_ENTROPY_RUN));
}

#[test]
fn capture_text_over_bound_refused_without_truncation() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    let text = "a".repeat(2001);
    let envelope = refusal(
        &dir,
        &["note", &text, "--actor", "sasha", "--key", "too-long-text"],
    );
    assert_eq!(
        envelope["error"]["label"],
        json!("kaya.capture.bound_exceeded"),
        "an over-bound payload is refused, never truncated"
    );
    let message = envelope["error"]["message"].as_str().expect("message");
    assert!(
        message.contains("text") && message.contains("2000"),
        "the refusal names the field and its limit: {message}"
    );
    assert_eq!(
        ledger_lines(&dir).len(),
        0,
        "the ledger gains no event for a refused payload"
    );
}

#[test]
fn capture_key_over_bound_or_off_alphabet_refused() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    let oversized = "a".repeat(65);
    for bad_key in [oversized.as_str(), "Drain-Behaviour", "drain_behaviour"] {
        let envelope = refusal(
            &dir,
            &["note", "text", "--actor", "sasha", "--key", bad_key],
        );
        assert_eq!(
            envelope["error"]["label"],
            json!("kaya.capture.invalid_key"),
            "key {bad_key:?} is refused"
        );
        let message = envelope["error"]["message"].as_str().expect("message");
        assert!(
            message.contains("key"),
            "the refusal names the key field: {message}"
        );
    }
    assert_eq!(
        ledger_lines(&dir).len(),
        0,
        "no refused capture appends an event"
    );
}

#[test]
fn capture_empty_key_or_empty_text_refused() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);

    let envelope = refusal(
        &dir,
        &["note", "", "--actor", "sasha", "--key", "empty-text"],
    );
    assert_eq!(
        envelope["error"]["label"],
        json!("kaya.capture.empty_text"),
        "empty text carries its own label"
    );

    let envelope = refusal(&dir, &["note", "text", "--actor", "sasha", "--key", ""]);
    assert_eq!(
        envelope["error"]["label"],
        json!("kaya.capture.invalid_key"),
        "an empty key is refused as an invalid key"
    );
    let message = envelope["error"]["message"].as_str().expect("message");
    assert!(message.contains("key"), "the refusal names the key field");

    assert_eq!(
        ledger_lines(&dir).len(),
        0,
        "no refused capture appends an event"
    );
}

#[test]
fn capture_project_off_alphabet_refused() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    for bad_project in ["Bad Project", "proj_1", ""] {
        let envelope = refusal(
            &dir,
            &["note", "text", "--actor", "sasha", "--project", bad_project],
        );
        assert_eq!(
            envelope["error"]["label"],
            json!("kaya.capture.invalid_project"),
            "project {bad_project:?} is refused"
        );
        let message = envelope["error"]["message"].as_str().expect("message");
        assert!(
            message.contains("project"),
            "the refusal names the project field: {message}"
        );
    }
    assert_eq!(
        ledger_lines(&dir).len(),
        0,
        "no refused capture appends an event"
    );
}

#[test]
fn tool_actor_refused() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    let envelope = refusal(
        &dir,
        &[
            "note",
            "a tool cannot author a capture",
            "--actor",
            "ci-runner",
            "--actor-type",
            "tool",
            "--key",
            "tool-note",
        ],
    );
    assert_eq!(
        envelope["error"]["label"],
        json!("kaya.capture.actor_required"),
        "a tool actor type is refused as unlabeled authorship"
    );
    assert_eq!(
        ledger_lines(&dir).len(),
        0,
        "the refused capture appends nothing"
    );
}

#[test]
fn agent_actor_recorded() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    let envelope = capture(
        &dir,
        &[
            "note",
            "agent-authored observation",
            "--actor",
            "atlas",
            "--actor-type",
            "agent",
            "--key",
            "agent-note",
        ],
    );
    assert_eq!(envelope["data"]["outcome"], json!("appended"));
    let event = stored_event(&dir, 0);
    assert_eq!(
        event["actor"],
        json!({ "type": "agent", "name": "atlas" }),
        "agent authorship is carried on the stored event"
    );
}

#[test]
fn missing_actor_refused_appends_nothing() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    let output = run(&dir, &["note", "orphan text"]);
    assert!(
        output.status.code().is_some_and(|code| code != 0),
        "a capture without --actor is refused"
    );
    assert_eq!(
        ledger_lines(&dir).len(),
        0,
        "the refused capture appends nothing"
    );
}

#[test]
fn hex_digest_run_redacted() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    let digest = "a3f8c92be1d74056ab09fc83e7d21b64c5a09f38d1e64b7c9a03fd28e51b6470";
    let text = format!("webhook signed with {digest} per docs");
    let envelope = capture(
        &dir,
        &[
            "note",
            &text,
            "--actor",
            "sasha",
            "--key",
            "webhook-signing",
        ],
    );
    assert_eq!(envelope["data"]["outcome"], json!("appended"));
    assert_eq!(
        stored_event(&dir, 0)["data"]["text"],
        json!("webhook signed with [redacted] per docs"),
        "an unlabeled hex run of 32+ characters with a hex letter is secret by default"
    );
    assert!(!ledger_text(&dir).contains(digest));
}

#[test]
fn dsa_private_key_armour_redacted() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    let header = "-----BEGIN DSA PRIVATE KEY-----";
    let text = format!("pasted {header} MIIBSwIBAAKBgQ");
    let envelope = capture(&dir, &["note", &text, "--actor", "sasha"]);
    assert_eq!(envelope["data"]["outcome"], json!("appended"));
    assert_eq!(
        stored_event(&dir, 0)["data"]["text"],
        json!("pasted [redacted] MIIBSwIBAAKBgQ"),
        "any -----BEGIN [A-Z ]*PRIVATE KEY----- armour is redacted, DSA included"
    );
    assert!(!ledger_text(&dir).contains(header));
}

#[test]
fn corrupt_ledger_line_refuses_capture() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    let path = dir.path().join(".kaya").join("ledger.jsonl");
    std::fs::write(&path, "{not json at all\n").unwrap();
    let output = run(&dir, &["note", "text", "--actor", "sasha"]);
    assert!(
        output.status.code().is_some_and(|code| code != 0),
        "capture refuses a ledger with an interior malformed line instead of serving a subset"
    );
    assert_eq!(ledger_lines(&dir).len(), 1, "the refusal appends nothing");
}
