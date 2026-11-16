//! Contract tests for the append-only ledger.

use std::fs;
use std::io::Write as _;

use kaya::event::{Actor, ActorType, Event, Provenance, SCHEMA_V0, Subject, to_line};
use kaya::ledger::{Anchor, AppendOutcome, Ledger, Recovery, resolve, sha256_hex};
use serde_json::{Value, json};
use tempfile::TempDir;

const EMPTY_SHA256: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

const FIRST: &str = "01JAV3Z0Y6W4Z5X8B7C6D5E4F1";
const SECOND: &str = "01JAV3Z0Y6W4Z5X8B7C6D5E4F2";
const THIRD: &str = "01JAV3Z0Y6W4Z5X8B7C6D5E4F3";

fn experiment_event(id: &str, event_type: &str, data: Value) -> Event {
    Event {
        id: id.to_owned(),
        schema: SCHEMA_V0.to_owned(),
        r#type: event_type.to_owned(),
        timestamp: "2026-11-16T10:00:00Z".to_owned(),
        actor: Actor {
            r#type: ActorType::Tool,
            name: "cargo-test".to_owned(),
        },
        subject: Subject {
            r#type: "Experiment".to_owned(),
            id: "kaya/Experiment/EXP-001".to_owned(),
        },
        data,
        provenance: Provenance {
            tool: "kaya".to_owned(),
            version: "0.1.0".to_owned(),
            reference: "commit:abc123".to_owned(),
        },
        artifacts: vec!["reports/run.json".to_owned()],
    }
}

fn ledger_path(dir: &TempDir) -> std::path::PathBuf {
    dir.path().join("ledger.jsonl")
}

#[test]
fn appends_only_extend_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = ledger_path(&dir);
    let mut ledger = Ledger::new(&path);
    assert_eq!(ledger.path(), path.as_path());

    let first = experiment_event(FIRST, "experiment.run.completed", json!({"passed": 3}));
    let second = experiment_event(SECOND, "experiment.run.completed", json!({"passed": 4}));

    assert_eq!(ledger.append(&first).unwrap(), AppendOutcome::Appended);
    let after_first = fs::read(&path).unwrap();
    assert_eq!(after_first, to_line(&first).into_bytes());

    assert_eq!(ledger.append(&second).unwrap(), AppendOutcome::Appended);
    let after_second = fs::read(&path).unwrap();
    assert_eq!(&after_second[..after_first.len()], &after_first[..]);
    assert_eq!(
        after_second,
        [after_first, to_line(&second).into_bytes()].concat()
    );
}

#[test]
fn identical_replay_is_a_byte_identical_no_op() {
    let dir = tempfile::tempdir().unwrap();
    let path = ledger_path(&dir);
    let mut ledger = Ledger::new(&path);
    let event = experiment_event(FIRST, "experiment.run.completed", json!({"passed": 3}));

    assert_eq!(ledger.append(&event).unwrap(), AppendOutcome::Appended);
    let before = fs::read(&path).unwrap();
    assert_eq!(ledger.append(&event).unwrap(), AppendOutcome::Duplicate);
    assert_eq!(fs::read(&path).unwrap(), before);

    let mut reopened = Ledger::open(&path).unwrap();
    assert_eq!(reopened.append(&event).unwrap(), AppendOutcome::Duplicate);
    assert_eq!(fs::read(&path).unwrap(), before);
}

#[test]
fn id_reuse_with_changed_bytes_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let path = ledger_path(&dir);
    let mut ledger = Ledger::new(&path);
    let event = experiment_event(FIRST, "experiment.run.completed", json!({"passed": 3}));

    assert_eq!(ledger.append(&event).unwrap(), AppendOutcome::Appended);
    let before = fs::read(&path).unwrap();

    let mut changed = event.clone();
    changed.data = json!({"passed": 4});
    let err = ledger.append(&changed).unwrap_err();
    assert_eq!(err.label, "kaya.ledger.id_reuse");
    assert!(err.message.contains(FIRST), "{}", err.message);
    assert_eq!(fs::read(&path).unwrap(), before);
}

#[test]
fn corrections_supersede_without_rewriting_the_original_line() {
    let dir = tempfile::tempdir().unwrap();
    let path = ledger_path(&dir);
    let mut ledger = Ledger::new(&path);
    let original = experiment_event(FIRST, "experiment.run.completed", json!({"passed": 3}));
    let correction = experiment_event(
        SECOND,
        "experiment.run.corrected",
        json!({"passed": 3, "supersedes": FIRST}),
    );

    assert_eq!(ledger.append(&original).unwrap(), AppendOutcome::Appended);
    let original_bytes = fs::read(&path).unwrap();

    assert_eq!(ledger.append(&correction).unwrap(), AppendOutcome::Appended);
    let after = fs::read(&path).unwrap();
    assert_eq!(&after[..original_bytes.len()], &original_bytes[..]);
    assert_eq!(
        after,
        [original_bytes, to_line(&correction).into_bytes()].concat()
    );

    let (events, recovery) = ledger.read_recovered().unwrap();
    assert_eq!(recovery, Recovery { truncated_bytes: 0 });
    assert_eq!(events, vec![original, correction.clone()]);

    let resolved = resolve(&events);
    assert_eq!(resolved.len(), 1);
    assert_eq!(resolved[0].id, correction.id);
    assert_eq!(resolved[0].r#type, "experiment.run.corrected");
}

#[test]
fn digests_are_stable_across_reads() {
    let dir = tempfile::tempdir().unwrap();
    let path = ledger_path(&dir);
    let mut ledger = Ledger::new(&path);
    let first = experiment_event(FIRST, "experiment.run.completed", json!({"passed": 3}));
    let second = experiment_event(SECOND, "experiment.run.completed", json!({"passed": 4}));
    ledger.append(&first).unwrap();
    ledger.append(&second).unwrap();

    let bytes = fs::read(&path).unwrap();
    let anchor = Anchor::of(&bytes);
    assert_eq!(anchor, Anchor::of(&fs::read(&path).unwrap()));
    assert_eq!(anchor.digest, sha256_hex(&bytes));
    assert_eq!(anchor.line_digests.len(), 2);

    let (events, recovery) = ledger.read_recovered().unwrap();
    assert_eq!(recovery, Recovery { truncated_bytes: 0 });
    assert_eq!(events.len(), 2);
    assert_eq!(Anchor::of(&fs::read(&path).unwrap()), anchor);
    assert_eq!(ledger.read_verified(Some(&anchor)).unwrap().len(), 2);
    assert_eq!(ledger.read_verified(None).unwrap().len(), 2);
}

#[test]
fn tampered_line_is_refused_naming_the_line_number() {
    let dir = tempfile::tempdir().unwrap();
    let path = ledger_path(&dir);
    let mut ledger = Ledger::new(&path);
    for (id, passed) in [(FIRST, 3), (SECOND, 4), (THIRD, 5)] {
        let event = experiment_event(id, "experiment.run.completed", json!({"passed": passed}));
        ledger.append(&event).unwrap();
    }
    let anchor = Anchor::of(&fs::read(&path).unwrap());

    let text = String::from_utf8(fs::read(&path).unwrap()).unwrap();
    let mut lines: Vec<String> = text.lines().map(str::to_owned).collect();
    assert_eq!(lines.len(), 3);
    let tampered = lines[1].replace("\"passed\":4", "\"passed\":9");
    assert_ne!(tampered, lines[1], "second line carries the marker");
    lines[1] = tampered;
    let mut rewritten = lines.join("\n");
    rewritten.push('\n');
    fs::write(&path, rewritten).unwrap();

    let err = ledger.read_verified(Some(&anchor)).unwrap_err();
    assert_eq!(err.label, "kaya.ledger.digest_mismatch");
    assert!(err.message.contains("line 2"), "{}", err.message);
    assert_eq!(err.to_string(), format!("{}: {}", err.label, err.message));

    // Without an anchor the same bytes still parse; verification is what refuses.
    assert_eq!(ledger.read_verified(None).unwrap().len(), 3);
}

#[test]
fn crash_tail_is_recovered_and_reported() {
    let dir = tempfile::tempdir().unwrap();
    let path = ledger_path(&dir);
    let mut ledger = Ledger::new(&path);
    let first = experiment_event(FIRST, "experiment.run.completed", json!({"passed": 3}));
    let second = experiment_event(SECOND, "experiment.run.completed", json!({"passed": 4}));
    ledger.append(&first).unwrap();
    ledger.append(&second).unwrap();
    let clean = fs::read(&path).unwrap();

    let partial = br#"{"id":"01JAV3Z0Y6W4Z5X8B7C6D5E4"#;
    {
        let mut file = fs::OpenOptions::new().append(true).open(&path).unwrap();
        file.write_all(partial).unwrap();
    }
    assert_eq!(fs::read(&path).unwrap().len(), clean.len() + partial.len());

    let (events, recovery) = ledger.read_recovered().unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(
        recovery,
        Recovery {
            truncated_bytes: partial.len()
        }
    );
    assert_eq!(fs::read(&path).unwrap(), clean);

    let (events, recovery) = ledger.read_recovered().unwrap();
    assert_eq!(recovery, Recovery { truncated_bytes: 0 });
    assert_eq!(events.len(), 2);

    let third = experiment_event(THIRD, "experiment.run.completed", json!({"passed": 5}));
    assert_eq!(ledger.append(&third).unwrap(), AppendOutcome::Appended);
    let after = fs::read(&path).unwrap();
    assert_eq!(&after[..clean.len()], &clean[..]);
    assert_eq!(after, [clean, to_line(&third).into_bytes()].concat());
}

#[test]
fn empty_ledger_digest_is_sha256_of_no_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let path = ledger_path(&dir);
    fs::write(&path, b"").unwrap();
    let ledger = Ledger::open(&path).unwrap();

    let (events, recovery) = ledger.read_recovered().unwrap();
    assert!(events.is_empty());
    assert_eq!(recovery, Recovery { truncated_bytes: 0 });

    let anchor = Anchor::of(&fs::read(&path).unwrap());
    assert_eq!(anchor.digest, EMPTY_SHA256);
    assert_eq!(anchor.digest, sha256_hex(b""));
    assert!(anchor.line_digests.is_empty());
    assert!(ledger.read_verified(Some(&anchor)).unwrap().is_empty());
}

#[test]
fn opening_a_missing_ledger_is_a_labeled_error() {
    let dir = tempfile::tempdir().unwrap();
    let path = ledger_path(&dir);

    let err = Ledger::open(&path).unwrap_err();
    assert_eq!(err.label, "kaya.ledger.missing");
    assert!(err.message.contains("ledger.jsonl"), "{}", err.message);

    let err = Ledger::new(&path).read_recovered().unwrap_err();
    assert_eq!(err.label, "kaya.ledger.missing");
}

#[test]
fn first_append_creates_the_ledger() {
    let dir = tempfile::tempdir().unwrap();
    let path = ledger_path(&dir);
    assert!(!path.exists());

    let mut ledger = Ledger::new(&path);
    let event = experiment_event(FIRST, "experiment.run.completed", json!({"passed": 3}));
    assert_eq!(ledger.append(&event).unwrap(), AppendOutcome::Appended);
    assert_eq!(fs::read(&path).unwrap(), to_line(&event).into_bytes());
}

#[test]
fn resolve_drops_only_events_superseded_later() {
    let dir = tempfile::tempdir().unwrap();
    let path = ledger_path(&dir);
    let mut ledger = Ledger::new(&path);
    let original = experiment_event(FIRST, "experiment.run.completed", json!({"passed": 3}));
    let correction = experiment_event(
        SECOND,
        "experiment.run.corrected",
        json!({"supersedes": [FIRST], "passed": 3}),
    );
    let unrelated = experiment_event(THIRD, "experiment.run.completed", json!({"passed": 5}));
    ledger.append(&original).unwrap();
    ledger.append(&correction).unwrap();
    ledger.append(&unrelated).unwrap();

    let (events, _) = ledger.read_recovered().unwrap();
    assert_eq!(events.len(), 3);
    let resolved = resolve(&events);
    let ids: Vec<&str> = resolved.iter().map(|event| event.id.as_str()).collect();
    assert_eq!(ids, vec![SECOND, THIRD]);
}
