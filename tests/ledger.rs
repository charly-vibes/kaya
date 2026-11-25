//! Contract tests for the append-only ledger.

use std::fs;
use std::io::Write as _;

use kaya::event::{Actor, ActorType, Event, Provenance, SCHEMA_V0, Subject, to_line};
use kaya::ledger::{Anchor, AppendOutcome, Ledger, Recovery, resolve_effective, sha256_hex};
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
            r#type: "Run".to_owned(),
            id: "kaya/Run/RUN-001".to_owned(),
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
    assert_eq!(
        after_first,
        to_line(&first).expect("event data is finite").into_bytes()
    );

    assert_eq!(ledger.append(&second).unwrap(), AppendOutcome::Appended);
    let after_second = fs::read(&path).unwrap();
    assert_eq!(&after_second[..after_first.len()], &after_first[..]);
    assert_eq!(
        after_second,
        [
            after_first,
            to_line(&second).expect("event data is finite").into_bytes()
        ]
        .concat()
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

/// Pin the documented replay policy (kaya-sh6): a semantically-equal replay
/// whose stored line was hand-edited after storage (here: reordered keys and
/// a dropped trailing newline, both accepted by the parser) refuses with
/// id_reuse instead of deduping — the stored bytes are the record.
#[test]
fn cosmetically_different_replay_of_hand_edited_line_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let path = ledger_path(&dir);
    let mut ledger = Ledger::new(&path);
    let event = experiment_event(FIRST, "experiment.run.completed", json!({"passed": 3}));
    assert_eq!(ledger.append(&event).unwrap(), AppendOutcome::Appended);

    // Hand-edit the stored line: re-serialize the parsed JSON with sorted
    // keys and no trailing newline. serde_json parses it to the same Event,
    // but the bytes differ from the canonical line.
    let line = to_line(&event).expect("finite event serializes");
    let parsed: serde_json::Value =
        serde_json::from_str(line.trim_end()).expect("canonical line parses");
    let hand_edited = serde_json::to_string(&parsed).expect("hand edit serializes");
    assert_ne!(hand_edited.as_bytes(), line.as_bytes());
    fs::write(&path, hand_edited.as_bytes()).unwrap();

    let err = ledger.append(&event).unwrap_err();
    assert_eq!(err.label, "kaya.ledger.id_reuse");
    assert!(err.message.contains(FIRST), "{}", err.message);
    // Recovery newline-terminates the hand-edited (unterminated) line before
    // the refusal; the content bytes are otherwise untouched.
    let mut stored = hand_edited.into_bytes();
    stored.push(b'\n');
    assert_eq!(fs::read(&path).unwrap(), stored);
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
        [
            original_bytes,
            to_line(&correction)
                .expect("event data is finite")
                .into_bytes()
        ]
        .concat()
    );

    let (events, recovery) = ledger.read_recovered().unwrap();
    assert_eq!(
        recovery,
        Recovery {
            truncated_bytes: 0,
            appended_newline: false,
        }
    );
    assert_eq!(events, vec![original.clone(), correction.clone()]);

    let effective = resolve_effective(&events);
    assert_eq!(effective.len(), 1);
    assert_eq!(effective[0].id, original.id);
    assert_eq!(effective[0].data, json!({"passed": 3}));
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
    assert_eq!(anchor.byte_len, bytes.len());
    assert_eq!(anchor.line_digests.len(), 2);

    let (events, recovery) = ledger.read_recovered().unwrap();
    assert_eq!(
        recovery,
        Recovery {
            truncated_bytes: 0,
            appended_newline: false,
        }
    );
    assert_eq!(events.len(), 2);
    assert_eq!(Anchor::of(&fs::read(&path).unwrap()), anchor);
    let (verified, recovery) = ledger.read_verified(&anchor).unwrap();
    assert_eq!(verified.len(), 2);
    assert_eq!(recovery, Recovery::default());
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

    let err = ledger.read_verified(&anchor).unwrap_err();
    assert_eq!(err.label, "kaya.ledger.digest_mismatch");
    assert!(err.message.contains("line 2"), "{}", err.message);
    assert_eq!(err.to_string(), format!("{}: {}", err.label, err.message));

    // The unverified path still parses the bytes; verification is what refuses.
    assert_eq!(ledger.read_recovered().unwrap().0.len(), 3);
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
            truncated_bytes: partial.len(),
            appended_newline: false,
        }
    );
    assert_eq!(fs::read(&path).unwrap(), clean);

    let (events, recovery) = ledger.read_recovered().unwrap();
    assert_eq!(recovery, Recovery::default());
    assert_eq!(events.len(), 2);

    let third = experiment_event(THIRD, "experiment.run.completed", json!({"passed": 5}));
    assert_eq!(ledger.append(&third).unwrap(), AppendOutcome::Appended);
    let after = fs::read(&path).unwrap();
    assert_eq!(&after[..clean.len()], &clean[..]);
    assert_eq!(
        after,
        [
            clean,
            to_line(&third).expect("event data is finite").into_bytes()
        ]
        .concat()
    );
}

#[test]
fn empty_ledger_digest_is_sha256_of_no_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let path = ledger_path(&dir);
    fs::write(&path, b"").unwrap();
    let ledger = Ledger::open(&path).unwrap();

    let (events, recovery) = ledger.read_recovered().unwrap();
    assert!(events.is_empty());
    assert_eq!(recovery, Recovery::default());

    let anchor = Anchor::of(&fs::read(&path).unwrap());
    assert_eq!(anchor.digest, EMPTY_SHA256);
    assert_eq!(anchor.digest, sha256_hex(b""));
    assert_eq!(anchor.byte_len, 0);
    assert!(anchor.line_digests.is_empty());
    let (events, recovery) = ledger.read_verified(&anchor).unwrap();
    assert!(events.is_empty());
    assert_eq!(recovery, Recovery::default());
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
    assert_eq!(
        fs::read(&path).unwrap(),
        to_line(&event).expect("event data is finite").into_bytes()
    );
}

#[test]
fn resolve_effective_excludes_correction_events_and_keeps_superseded_targets() {
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
    let effective = resolve_effective(&events);
    let ids: Vec<&str> = effective.iter().map(|event| event.id.as_str()).collect();
    assert_eq!(ids, vec![FIRST, THIRD]);
}

#[test]
fn verified_read_accepts_growth_after_the_anchor() {
    let dir = tempfile::tempdir().unwrap();
    let path = ledger_path(&dir);
    let mut ledger = Ledger::new(&path);
    let first = experiment_event(FIRST, "experiment.run.completed", json!({"passed": 3}));
    let second = experiment_event(SECOND, "experiment.run.completed", json!({"passed": 4}));

    ledger.append(&first).unwrap();
    let anchor = Anchor::of(&fs::read(&path).unwrap());
    assert_eq!(
        anchor.byte_len,
        to_line(&first).expect("event data is finite").len()
    );

    ledger.append(&second).unwrap();
    let (events, recovery) = ledger.read_verified(&anchor).unwrap();
    assert_eq!(events, vec![first.clone(), second]);
    assert_eq!(recovery, Recovery::default());

    let bytes = fs::read(&path).unwrap();
    assert_eq!(
        &bytes[..anchor.byte_len],
        to_line(&first).expect("event data is finite").as_bytes()
    );
}

#[test]
fn verified_read_reports_a_crash_tail_after_the_anchor() {
    let dir = tempfile::tempdir().unwrap();
    let path = ledger_path(&dir);
    let mut ledger = Ledger::new(&path);
    let first = experiment_event(FIRST, "experiment.run.completed", json!({"passed": 3}));
    ledger.append(&first).unwrap();
    let anchored = fs::read(&path).unwrap();
    let anchor = Anchor::of(&anchored);

    let partial = br#"{"id":"01JAV3Z0Y6W4Z5X8B7C6D5E4"#;
    {
        let mut file = fs::OpenOptions::new().append(true).open(&path).unwrap();
        file.write_all(partial).unwrap();
    }

    let (events, recovery) = ledger.read_verified(&anchor).unwrap();
    assert_eq!(events, vec![first]);
    assert_eq!(
        recovery,
        Recovery {
            truncated_bytes: partial.len(),
            appended_newline: false,
        }
    );
    assert_eq!(fs::read(&path).unwrap(), anchored);
}

#[test]
fn complete_unterminated_line_is_preserved_by_appending_a_newline() {
    let dir = tempfile::tempdir().unwrap();
    let path = ledger_path(&dir);
    let mut ledger = Ledger::new(&path);
    let first = experiment_event(FIRST, "experiment.run.completed", json!({"passed": 3}));
    let second = experiment_event(SECOND, "experiment.run.completed", json!({"passed": 4}));
    ledger.append(&first).unwrap();
    ledger.append(&second).unwrap();

    let clean = fs::read(&path).unwrap();
    let unterminated = &clean[..clean.len() - 1];
    fs::write(&path, unterminated).unwrap();

    let (events, recovery) = ledger.read_recovered().unwrap();
    assert_eq!(events, vec![first, second]);
    assert_eq!(
        recovery,
        Recovery {
            truncated_bytes: 0,
            appended_newline: true,
        }
    );
    assert_eq!(fs::read(&path).unwrap(), clean);
}

#[test]
fn verified_read_reports_an_appended_newline_on_growth() {
    let dir = tempfile::tempdir().unwrap();
    let path = ledger_path(&dir);
    let mut ledger = Ledger::new(&path);
    let first = experiment_event(FIRST, "experiment.run.completed", json!({"passed": 3}));
    let second = experiment_event(SECOND, "experiment.run.completed", json!({"passed": 4}));
    ledger.append(&first).unwrap();
    let anchored = fs::read(&path).unwrap();
    let anchor = Anchor::of(&anchored);

    let second_line = to_line(&second).expect("event data is finite");
    let mut unterminated = [anchored.clone(), second_line.clone().into_bytes()].concat();
    assert_eq!(unterminated.pop(), Some(b'\n'));
    fs::write(&path, &unterminated).unwrap();

    let (events, recovery) = ledger.read_verified(&anchor).unwrap();
    assert_eq!(events, vec![first, second.clone()]);
    assert_eq!(
        recovery,
        Recovery {
            truncated_bytes: 0,
            appended_newline: true,
        }
    );
    assert_eq!(
        fs::read(&path).unwrap(),
        [anchored, second_line.into_bytes()].concat()
    );
}

#[test]
fn duplicate_identical_lines_are_refused_naming_both_lines() {
    let dir = tempfile::tempdir().unwrap();
    let path = ledger_path(&dir);
    let mut ledger = Ledger::new(&path);
    let event = experiment_event(FIRST, "experiment.run.completed", json!({"passed": 3}));
    ledger.append(&event).unwrap();

    let line = fs::read(&path).unwrap();
    fs::write(&path, [line.clone(), line].concat()).unwrap();

    let err = ledger.read_recovered().unwrap_err();
    assert_eq!(err.label, "kaya.ledger.id_reuse");
    assert!(err.message.contains(FIRST), "{}", err.message);
    assert!(err.message.contains("lines 1 and 2"), "{}", err.message);
}

#[test]
fn duplicate_ids_with_changed_bytes_are_refused_on_read() {
    let dir = tempfile::tempdir().unwrap();
    let path = ledger_path(&dir);
    let mut ledger = Ledger::new(&path);
    let original = experiment_event(FIRST, "experiment.run.completed", json!({"passed": 3}));
    let mut changed = original.clone();
    changed.data = json!({"passed": 9});
    ledger.append(&original).unwrap();

    fs::write(
        &path,
        [
            to_line(&original)
                .expect("event data is finite")
                .into_bytes(),
            to_line(&changed)
                .expect("event data is finite")
                .into_bytes(),
        ]
        .concat(),
    )
    .unwrap();

    let err = ledger.read_recovered().unwrap_err();
    assert_eq!(err.label, "kaya.ledger.id_reuse");
    assert!(err.message.contains(FIRST), "{}", err.message);
    assert!(err.message.contains("lines 1 and 2"), "{}", err.message);
}

#[test]
fn resolve_effective_applies_a_run_counts_correction() {
    let dir = tempfile::tempdir().unwrap();
    let path = ledger_path(&dir);
    let mut ledger = Ledger::new(&path);
    let original = experiment_event(
        FIRST,
        "experiment.run.completed",
        json!({"passed": 3, "failed": 1, "ignored": 0, "total": 4}),
    );
    let correction = experiment_event(
        SECOND,
        "experiment.run.corrected",
        json!({"supersedes": FIRST, "passed": 4, "failed": 0, "ignored": 0, "total": 4}),
    );

    ledger.append(&original).unwrap();
    let original_line = to_line(&original)
        .expect("event data is finite")
        .into_bytes();
    let before = fs::read(&path).unwrap();
    ledger.append(&correction).unwrap();
    let after = fs::read(&path).unwrap();
    assert_eq!(&after[..before.len()], &before[..]);
    assert_eq!(&after[..original_line.len()], &original_line[..]);

    let (events, _) = ledger.read_recovered().unwrap();
    let effective = resolve_effective(&events);
    assert_eq!(effective.len(), 1);

    let event = &effective[0];
    assert_eq!(event.id, FIRST);
    assert_eq!(event.r#type, "experiment.run.completed");
    assert_eq!(event.subject, original.subject);
    assert_eq!(event.timestamp, original.timestamp);
    assert_eq!(
        event.data,
        json!({"passed": 4, "failed": 0, "ignored": 0, "total": 4})
    );
    assert!(event.data.get("supersedes").is_none());
    assert_eq!(fs::read(&path).unwrap(), after);
}

#[test]
fn resolve_effective_lets_later_corrections_win() {
    let dir = tempfile::tempdir().unwrap();
    let path = ledger_path(&dir);
    let mut ledger = Ledger::new(&path);
    let original = experiment_event(FIRST, "experiment.run.completed", json!({"passed": 3}));
    let first_correction = experiment_event(
        SECOND,
        "experiment.run.corrected",
        json!({"supersedes": FIRST, "passed": 4}),
    );
    let second_correction = experiment_event(
        THIRD,
        "experiment.run.corrected",
        json!({"supersedes": [FIRST], "passed": 5}),
    );
    ledger.append(&original).unwrap();
    ledger.append(&first_correction).unwrap();
    ledger.append(&second_correction).unwrap();

    let (events, _) = ledger.read_recovered().unwrap();
    let effective = resolve_effective(&events);
    assert_eq!(effective.len(), 1);
    assert_eq!(effective[0].id, FIRST);
    assert_eq!(effective[0].data, json!({"passed": 5}));
}

#[test]
fn stale_anchor_for_a_shorter_ledger_is_refused_naming_the_missing_line() {
    let dir = tempfile::tempdir().unwrap();
    let path = ledger_path(&dir);
    let mut ledger = Ledger::new(&path);
    let first = experiment_event(FIRST, "experiment.run.completed", json!({"passed": 3}));
    let second = experiment_event(SECOND, "experiment.run.completed", json!({"passed": 4}));
    ledger.append(&first).unwrap();
    ledger.append(&second).unwrap();

    let anchor = Anchor::of(&fs::read(&path).unwrap());
    fs::write(&path, to_line(&first).expect("event data is finite")).unwrap();

    let err = ledger.read_verified(&anchor).unwrap_err();
    assert_eq!(err.label, "kaya.ledger.digest_mismatch");
    assert!(err.message.contains("line 2"), "{}", err.message);
}

#[test]
fn non_line_aligned_anchor_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let path = ledger_path(&dir);
    let mut ledger = Ledger::new(&path);
    let first = experiment_event(FIRST, "experiment.run.completed", json!({"passed": 3}));
    ledger.append(&first).unwrap();

    let bytes = fs::read(&path).unwrap();
    let anchor = Anchor::of(&bytes[..bytes.len() - 2]);
    assert!(anchor.line_digests.is_empty());

    let err = ledger.read_verified(&anchor).unwrap_err();
    assert_eq!(err.label, "kaya.ledger.digest_mismatch");
    assert!(err.message.contains("line 1"), "{}", err.message);
}

#[test]
fn anchor_byte_len_round_trips_through_verification() {
    let dir = tempfile::tempdir().unwrap();
    let path = ledger_path(&dir);
    let mut ledger = Ledger::new(&path);
    let first = experiment_event(FIRST, "experiment.run.completed", json!({"passed": 3}));
    let second = experiment_event(SECOND, "experiment.run.completed", json!({"passed": 4}));
    ledger.append(&first).unwrap();
    ledger.append(&second).unwrap();

    let bytes = fs::read(&path).unwrap();
    let anchor = Anchor::of(&bytes);
    assert_eq!(anchor.byte_len, bytes.len());
    assert_eq!(anchor.line_digests.len(), 2);
    assert_eq!(anchor, Anchor::of(&bytes));

    let (events, recovery) = ledger.read_verified(&anchor).unwrap();
    assert_eq!(events, vec![first, second]);
    assert_eq!(recovery, Recovery::default());
}

#[test]
fn committed_ledger_diff_shows_only_additions() {
    use std::process::Command;

    let dir = tempfile::tempdir().unwrap();
    let path = ledger_path(&dir);
    let git = |args: &[&str]| {
        let out = Command::new("git")
            .arg("-C")
            .arg(dir.path())
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_SYSTEM", "/dev/null")
            .env("GIT_AUTHOR_NAME", "kaya test")
            .env("GIT_AUTHOR_EMAIL", "test@example.invalid")
            .env("GIT_COMMITTER_NAME", "kaya test")
            .env("GIT_COMMITTER_EMAIL", "test@example.invalid")
            .env("GIT_AUTHOR_DATE", "2026-11-16T00:00:00+00:00")
            .env("GIT_COMMITTER_DATE", "2026-11-16T00:00:00+00:00")
            .args(args)
            .output()
            .expect("git runs");
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).into_owned()
    };

    git(&["init", "--quiet"]);
    let mut ledger = Ledger::new(&path);
    let first = experiment_event(FIRST, "experiment.run.completed", json!({"passed": 3}));
    ledger.append(&first).unwrap();
    git(&["add", "ledger.jsonl"]);
    git(&["commit", "--quiet", "--message", "seed the ledger"]);

    let second = experiment_event(SECOND, "experiment.run.completed", json!({"passed": 4}));
    ledger.append(&second).unwrap();

    let diff = git(&["diff", "--unified=0", "--", "ledger.jsonl"]);
    let removed: Vec<&str> = diff
        .lines()
        .filter(|line| line.starts_with('-') && !line.starts_with("---"))
        .collect();
    assert!(
        removed.is_empty(),
        "append-only diff has no removals: {removed:?}"
    );
    assert!(
        diff.lines().any(|line| line.starts_with('+')),
        "the appended line appears in the diff: {diff}"
    );
}

const FOURTH: &str = "01JAV3Z0Y6W4Z5X8B7C6D5E4F4";

/// Assert the ledger refuses `event` with the supersedes label and no byte change.
fn assert_supersedes_refused(ledger: &mut Ledger, path: &std::path::Path, event: &Event) {
    let before = fs::read(path).unwrap_or_default();
    let err = ledger.append(event).unwrap_err();
    assert_eq!(err.label, "kaya.ledger.supersedes_invalid");
    assert!(err.message.contains(&event.id), "{}", err.message);
    assert_eq!(fs::read(path).unwrap_or_default(), before);
}

#[test]
fn dangling_supersedes_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let path = ledger_path(&dir);
    let mut ledger = Ledger::new(&path);
    let first = experiment_event(FIRST, "experiment.run.completed", json!({"passed": 3}));
    assert_eq!(ledger.append(&first).unwrap(), AppendOutcome::Appended);

    let dangling = experiment_event(
        SECOND,
        "experiment.run.corrected",
        json!({"passed": 4, "supersedes": THIRD}),
    );
    assert_supersedes_refused(&mut ledger, &path, &dangling);
}

#[test]
fn self_referencing_supersedes_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let path = ledger_path(&dir);
    let mut ledger = Ledger::new(&path);
    let first = experiment_event(FIRST, "experiment.run.completed", json!({"passed": 3}));
    assert_eq!(ledger.append(&first).unwrap(), AppendOutcome::Appended);

    let self_referencing = experiment_event(
        SECOND,
        "experiment.run.corrected",
        json!({"passed": 4, "supersedes": SECOND}),
    );
    assert_supersedes_refused(&mut ledger, &path, &self_referencing);
}

#[test]
fn mutual_supersedes_cycle_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let path = ledger_path(&dir);
    // A forward-referencing stored line stands in for a pre-validation
    // ledger: appending B{supersedes: A} closes A -> B -> A, which the
    // chain walk must refuse (byte-identical mutual cycles cannot be
    // appended, since A naming B would be id reuse).
    let a = experiment_event(
        FIRST,
        "experiment.run.completed",
        json!({"passed": 3, "supersedes": SECOND}),
    );
    fs::write(&path, to_line(&a).expect("event data is finite")).unwrap();
    let mut ledger = Ledger::new(&path);
    let b = experiment_event(
        SECOND,
        "experiment.run.corrected",
        json!({"passed": 4, "supersedes": FIRST}),
    );
    assert_supersedes_refused(&mut ledger, &path, &b);
    assert_eq!(
        fs::read_to_string(&path)
            .expect("ledger reads")
            .lines()
            .count(),
        1,
        "refusal leaves only the stored line"
    );
}

#[test]
fn three_event_supersedes_cycle_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let path = ledger_path(&dir);
    // A forward-referencing stored line (as a pre-validation ledger could
    // hold) lets a fresh append close a genuine 3-cycle: C -> B -> A -> C.
    let a = experiment_event(
        FIRST,
        "experiment.run.completed",
        json!({"passed": 3, "supersedes": THIRD}),
    );
    fs::write(&path, to_line(&a).expect("event data is finite")).unwrap();
    let mut ledger = Ledger::new(&path);
    let b = experiment_event(
        SECOND,
        "experiment.run.corrected",
        json!({"passed": 4, "supersedes": FIRST}),
    );
    assert_eq!(ledger.append(&b).unwrap(), AppendOutcome::Appended);
    let cycle = experiment_event(
        THIRD,
        "experiment.run.corrected",
        json!({"passed": 8, "supersedes": SECOND}),
    );
    assert_supersedes_refused(&mut ledger, &path, &cycle);
}

#[test]
fn supersedes_object_shape_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let path = ledger_path(&dir);
    let mut ledger = Ledger::new(&path);
    let first = experiment_event(FIRST, "experiment.run.completed", json!({"passed": 3}));
    assert_eq!(ledger.append(&first).unwrap(), AppendOutcome::Appended);

    let malformed = experiment_event(
        SECOND,
        "experiment.run.corrected",
        json!({"passed": 4, "supersedes": {"id": FIRST}}),
    );
    assert_supersedes_refused(&mut ledger, &path, &malformed);
}

#[test]
fn supersedes_empty_array_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let path = ledger_path(&dir);
    let mut ledger = Ledger::new(&path);
    let first = experiment_event(FIRST, "experiment.run.completed", json!({"passed": 3}));
    assert_eq!(ledger.append(&first).unwrap(), AppendOutcome::Appended);

    let empty_array = experiment_event(
        SECOND,
        "experiment.run.corrected",
        json!({"passed": 4, "supersedes": []}),
    );
    assert_supersedes_refused(&mut ledger, &path, &empty_array);
}

#[test]
fn supersedes_array_with_non_string_entry_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let path = ledger_path(&dir);
    let mut ledger = Ledger::new(&path);
    let first = experiment_event(FIRST, "experiment.run.completed", json!({"passed": 3}));
    assert_eq!(ledger.append(&first).unwrap(), AppendOutcome::Appended);

    let mixed = experiment_event(
        SECOND,
        "experiment.run.corrected",
        json!({"passed": 4, "supersedes": [FIRST, 7]}),
    );
    assert_supersedes_refused(&mut ledger, &path, &mixed);

    let empty_string_entry = experiment_event(
        THIRD,
        "experiment.run.corrected",
        json!({"passed": 5, "supersedes": [FIRST, ""]}),
    );
    assert_supersedes_refused(&mut ledger, &path, &empty_string_entry);
}

#[test]
fn valid_supersedes_chains_and_diamonds_still_append_and_resolve() {
    let dir = tempfile::tempdir().unwrap();
    let path = ledger_path(&dir);
    let mut ledger = Ledger::new(&path);
    let original = experiment_event(FIRST, "experiment.run.completed", json!({"passed": 3}));
    let chain = experiment_event(
        SECOND,
        "experiment.run.corrected",
        json!({"passed": 4, "supersedes": FIRST}),
    );
    let diamond = experiment_event(
        THIRD,
        "experiment.run.corrected",
        json!({"passed": 5, "supersedes": [SECOND, FOURTH]}),
    );
    let other = experiment_event(FOURTH, "experiment.run.completed", json!({"count": 9}));

    assert_eq!(ledger.append(&original).unwrap(), AppendOutcome::Appended);
    assert_eq!(ledger.append(&other).unwrap(), AppendOutcome::Appended);
    assert_eq!(ledger.append(&chain).unwrap(), AppendOutcome::Appended);
    assert_eq!(ledger.append(&diamond).unwrap(), AppendOutcome::Appended);

    let (events, _) = ledger.read_recovered().unwrap();
    assert_eq!(events.len(), 4);
    let effective = resolve_effective(&events);
    assert_eq!(effective.len(), 2);
    let first_effective = effective.iter().find(|e| e.id == FIRST).unwrap();
    assert_eq!(first_effective.data["passed"], 5);
}

/// Lock the ledger from a second in-process handle and return the handle.
///
/// A fresh `open` yields its own file description, so its exclusive advisory
/// lock conflicts with `Ledger`'s operations but not with anything already
/// open. Dropping the handle releases the lock.
fn hold_exclusive_lock(path: &std::path::Path) -> std::fs::File {
    use std::fs::TryLockError;

    let handle = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .unwrap();
    match handle.try_lock() {
        Ok(()) => handle,
        Err(TryLockError::WouldBlock) => panic!("second handle should acquire the lock"),
        Err(TryLockError::Error(err)) => panic!("locking failed: {err}"),
    }
}

#[test]
fn complete_final_line_missing_only_its_newline_gains_exactly_that_newline() {
    let dir = tempfile::tempdir().unwrap();
    let path = ledger_path(&dir);
    let mut ledger = Ledger::new(&path);
    let first = experiment_event(FIRST, "experiment.run.completed", json!({"passed": 3}));
    let second = experiment_event(SECOND, "experiment.run.completed", json!({"passed": 4}));
    ledger.append(&first).unwrap();
    ledger.append(&second).unwrap();

    let clean = fs::read(&path).unwrap();
    let mut unterminated = clean.clone();
    unterminated.pop();
    fs::write(&path, &unterminated).unwrap();

    let (events, recovery) = ledger.read_recovered().unwrap();
    assert_eq!(events, vec![first, second]);
    assert_eq!(
        recovery,
        Recovery {
            truncated_bytes: 0,
            appended_newline: true,
        }
    );
    assert_eq!(fs::read(&path).unwrap(), clean);
}

#[test]
fn read_recovered_under_contention_refuses_with_locked_label() {
    let dir = tempfile::tempdir().unwrap();
    let path = ledger_path(&dir);
    let mut ledger = Ledger::new(&path);
    let first = experiment_event(FIRST, "experiment.run.completed", json!({"passed": 3}));
    ledger.append(&first).unwrap();
    let before = fs::read(&path).unwrap();

    let _holder = hold_exclusive_lock(&path);
    let err = ledger.read_recovered().unwrap_err();
    assert_eq!(err.label, "kaya.ledger.locked");
    assert!(
        err.message.contains("held by another writer"),
        "message should name the holder: {}",
        err.message
    );
    assert!(err.message.contains(path.to_str().unwrap()));
    assert_eq!(fs::read(&path).unwrap(), before);
}

#[test]
fn append_under_contention_refuses_with_locked_label_then_succeeds_after_release() {
    let dir = tempfile::tempdir().unwrap();
    let path = ledger_path(&dir);
    let mut ledger = Ledger::new(&path);
    let first = experiment_event(FIRST, "experiment.run.completed", json!({"passed": 3}));
    ledger.append(&first).unwrap();
    let before = fs::read(&path).unwrap();

    let holder = hold_exclusive_lock(&path);
    let second = experiment_event(SECOND, "experiment.run.completed", json!({"passed": 4}));
    let err = ledger.append(&second).unwrap_err();
    assert_eq!(err.label, "kaya.ledger.locked");
    assert!(
        err.message.contains("held by another writer"),
        "message should name the holder: {}",
        err.message
    );
    assert_eq!(fs::read(&path).unwrap(), before);

    drop(holder);
    // Closing the holder releases its flock, but under parallel test load the
    // release can briefly stay visible as contention (the descriptor number
    // is recycled by a sibling thread microseconds before the retry). Poll
    // briefly for the release instead of asserting on the first attempt;
    // anything other than `kaya.ledger.locked`, or contention lasting beyond
    // the bound, still fails the test.
    let appended = {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        loop {
            match ledger.append(&second) {
                Ok(outcome) => break outcome,
                Err(err)
                    if err.label == "kaya.ledger.locked"
                        && std::time::Instant::now() < deadline =>
                {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
                Err(err) => panic!("append still refused after lock release: {err}"),
            }
        }
    };
    assert_eq!(appended, AppendOutcome::Appended);
    let expected = [
        before,
        to_line(&second).expect("event data is finite").into_bytes(),
    ]
    .concat();
    assert_eq!(fs::read(&path).unwrap(), expected);
}
