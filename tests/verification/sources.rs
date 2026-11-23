//! Canonical source and kind vocabularies in the facts census.

use std::fs;
use std::path::Path;
use std::process::Output;

use serde_json::json;

use crate::common::*;
use crate::facts_fixture::*;
use crate::snapshot::sha256_hex;
use crate::tampering::worktree_bytes;

/// A record holding three capture sources: a wai artifact, the one recent
/// commit, and a completed libtest run, committed with its projections.
/// Returns the commit the capture frontier must name.
fn record_with_captured_sources(root: &Path) -> String {
    let captured = head_sha(root);
    add_wai_artifact(root);
    run_ok(root, &["import", "wai"]);
    run_ok(root, &["import", "git", "--limit", "1"]);
    let report = write_libtest_report(root);
    let report = report.to_str().expect("the report path is UTF-8");
    run_ok(root, &["import", "test", "--report", report]);
    run_ok(root, &["build"]);
    run_ok(root, &["publish"]);
    git(root, &["add", "--all"]);
    git(
        root,
        &[
            "commit",
            "--quiet",
            "--message",
            "record imported observations",
        ],
    );
    captured
}

#[test]
fn check_facts_test_runner_source() {
    let fixture = Fixture::new();
    let root = fixture.root();
    let captured = record_with_captured_sources(root);

    let output = fixture.output(&["check"]);

    assert_eq!(
        exit_code(&output),
        0,
        "the extended record still verifies against its projections: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    assert_check_envelope(&envelope);
    let facts = record_facts(&envelope);

    // The report describes the verified ledger without parsing it again.
    assert_facts_match_ledger(facts, root, &envelope);

    // A record with more than one source still carries a capture frontier.
    assert_eq!(
        facts["last_capture_commit"],
        json!(captured),
        "the frontier names the one imported commit: {envelope}"
    );
    let counts = source_counts(facts);
    assert!(
        counts["test-runner"] >= 1,
        "the imported run counts under test-runner: {envelope}"
    );
    assert!(
        counts["wai"] >= 1,
        "the imported research note counts under wai: {envelope}"
    );
    assert!(
        counts["git"] >= 1,
        "the imported commit counts under git: {envelope}"
    );
    assert!(
        counts["model"] >= 1,
        "the seed's model events count under model: {envelope}"
    );
    assert_eq!(
        counts.values().sum::<u64>(),
        ledger_events(root).len() as u64,
        "the canonical counts partition every event: {envelope}"
    );
    assert!(
        !counts.contains_key("charly"),
        "the seed's human actor name never becomes a source key: {envelope}"
    );

    let emitted = dotted_kinds(facts, "emitted_kinds");
    let omitted = dotted_kinds(facts, "omitted_kinds");
    assert!(
        !emitted.is_empty(),
        "the projection emits at least the research map: {envelope}"
    );
    assert!(
        omitted.contains(&"understanding.decision-record".to_owned()),
        "the seed holds no decision, so that kind is omitted for no-records: {envelope}"
    );
}

#[test]
fn check_facts_turu_source() {
    let fixture = Fixture::new();
    let root = fixture.root();

    // A record holding one turu whisper-bundle entry beside the seed's model
    // events: the entry is observed, never interpreted, and counted under the
    // turu source alone.
    record_with_turu_source(root);

    let output = fixture.output(&["check"]);

    assert_eq!(
        exit_code(&output),
        0,
        "the record with a turu entry still verifies against its projections: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    assert_check_envelope(&envelope);
    let facts = record_facts(&envelope);

    let counts = source_counts(facts);
    assert_eq!(
        counts["turu"], 1,
        "the imported whisper entry counts under turu: {envelope}"
    );
    assert_eq!(
        counts["wai"], 0,
        "the turu entry is not a wai artifact: {envelope}"
    );
    assert_eq!(
        counts["git"], 0,
        "the turu entry is not a commit observation: {envelope}"
    );
    assert_eq!(
        counts["test-runner"], 0,
        "the turu entry is not a run observation: {envelope}"
    );
    assert_eq!(
        counts.values().sum::<u64>(),
        facts["events"]
            .as_u64()
            .unwrap_or_else(|| panic!("check reports an integer event count: {envelope}")),
        "the canonical counts partition every event: {envelope}"
    );
}

/// The fact set the dated-commits record must report.
struct ParityExpectations {
    frontier: String,
    timestamp: String,
    digest: String,
    events: usize,
}

/// A record with two commits whose log order and author dates disagree, plus
/// a wai artifact: the tip carries the earlier author date, so only a
/// frontier ordered by record time names the commit the record observed
/// later.
fn dated_commit_record(root: &Path) -> ParityExpectations {
    let notes = root.join("notes");
    fs::create_dir_all(&notes).expect("create the notes directory");

    fs::write(notes.join("observed-later.md"), "# observed later\n")
        .expect("write the later-dated note");
    git(root, &["add", "--all"]);
    commit_dated(
        root,
        "observe the later-dated commit",
        "2026-11-19T00:00:00+00:00",
        "2026-11-19T00:00:00+00:00",
    );
    let later_dated = head_sha(root);

    fs::write(notes.join("observed-earlier.md"), "# observed earlier\n")
        .expect("write the earlier-dated note");
    git(root, &["add", "--all"]);
    commit_dated(
        root,
        "observe the earlier-dated commit",
        "2026-11-17T00:00:00+00:00",
        "2026-11-21T00:00:00+00:00",
    );
    let later_logged = head_sha(root);
    assert_ne!(
        later_dated, later_logged,
        "the two dated commits are distinct"
    );

    record_with_two_sources(root, "2");
    let events = ledger_events(root);
    let imported = events
        .iter()
        .filter(|event| event["type"] == json!("capture.commit.observed"))
        .collect::<Vec<_>>();
    assert_eq!(
        imported.len(),
        2,
        "the limit imports both dated commits: {imported:?}"
    );
    ParityExpectations {
        // The later author date wins, not the later log entry the import read
        // last.
        frontier: later_dated,
        timestamp: newest_timestamp(&events),
        digest: sha256_hex(&worktree_bytes(root, LEDGER)),
        events: events.len(),
    }
}

/// Assert one check run reports the expected fact set over the record.
fn assert_facts_parity(output: &Output, expected: &ParityExpectations) {
    assert_eq!(
        exit_code(output),
        0,
        "the same clean record exits 0 in both modes: {}",
        stderr(output)
    );
    let envelope = parse_envelope(output);
    assert_check_envelope(&envelope);
    let facts = record_facts(&envelope);

    // Frontier, canonical counts, and dotted kinds are present together.
    assert_eq!(
        facts["last_capture_commit"],
        json!(expected.frontier),
        "the frontier names the newest recorded capture sha: {envelope}"
    );
    assert_eq!(
        facts["newest_event_at"],
        json!(expected.timestamp),
        "the newest event timestamp is the greatest recorded: {envelope}"
    );
    assert_eq!(
        facts["ledger_digest"],
        json!(expected.digest),
        "the digest covers the verified ledger bytes: {envelope}"
    );
    let counts = source_counts(facts);
    assert_eq!(
        counts.values().sum::<u64>(),
        expected.events as u64,
        "the per-source counts partition every event: {envelope}"
    );
    assert!(
        counts["wai"] >= 1 && counts["git"] >= 1,
        "both capture sources are counted: {envelope}"
    );
    let emitted = dotted_kinds(facts, "emitted_kinds");
    let omitted = dotted_kinds(facts, "omitted_kinds");
    assert!(
        !emitted.is_empty() && !omitted.is_empty(),
        "both kind lists are reported: {envelope}"
    );
}

#[test]
fn check_facts_property() {
    let fixture = Fixture::new();
    let root = fixture.root();
    let expected = dated_commit_record(root);

    // The same committed bytes read through the worktree and through the
    // index must yield one fact set: the facts describe the verified record,
    // not the read path.
    let worktree = fixture.output(&["check"]);
    let index = fixture.output(&["check", "--index"]);
    for output in [&worktree, &index] {
        assert_facts_parity(output, &expected);
    }

    assert_eq!(
        parse_envelope(&worktree)["data"],
        parse_envelope(&index)["data"],
        "worktree and index modes report identical facts for identical bytes"
    );
}
