//! The facts census: what an executed `check` reports about the record.

use serde_json::{Value, json};

use crate::common::*;
use crate::facts_fixture::*;

#[test]
fn check_facts_reported() {
    let fixture = Fixture::new();
    let root = fixture.root();

    let output = fixture.output(&["check"]);

    assert_eq!(
        exit_code(&output),
        0,
        "a clean record exits 0: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    assert_check_envelope(&envelope);
    let facts = record_facts(&envelope);

    // The report is a second reader of the record, not a second parser: its
    // count and digest describe exactly the verified ledger bytes.
    assert_facts_match_ledger(facts, root, &envelope);

    // The seed records no capture observation, so there is no frontier yet,
    // and every event is a model event.
    assert_eq!(
        facts["last_capture_commit"],
        Value::Null,
        "a record without capture events reports a null frontier: {envelope}"
    );
    let events = ledger_events(root);
    let counts = source_counts(facts);
    assert_eq!(
        counts["model"],
        events.len() as u64,
        "every seed event is a model event: {envelope}"
    );
    assert_eq!(
        counts["wai"], 0,
        "the seed imports no wai artifact observations: {envelope}"
    );
    assert_eq!(
        counts["git"], 0,
        "the seed imports no commit observations: {envelope}"
    );
    assert_eq!(
        counts["test-runner"], 0,
        "the seed imports no run observations: {envelope}"
    );
    assert_eq!(
        counts.values().sum::<u64>(),
        events.len() as u64,
        "the per-source counts partition every event: {envelope}"
    );
    assert!(
        !counts.contains_key("charly"),
        "the seed's human actor is never a source key: {envelope}"
    );

    // Both kind lists are one vocabulary: dotted `family.kind` names.
    let emitted = dotted_kinds(facts, "emitted_kinds");
    assert!(
        !emitted.is_empty(),
        "the projection emits at least the research map: {envelope}"
    );
    let omitted = dotted_kinds(facts, "omitted_kinds");
    assert!(
        omitted.contains(&"understanding.decision-record".to_owned()),
        "the seed holds no decision, so that kind is omitted for no-records: {envelope}"
    );
}

#[test]
fn check_capture_frontier_survives_rewrite() {
    let fixture = Fixture::new();
    let root = fixture.root();
    let captured = head_sha(root);

    // Record the frontier, then rewrite that very commit: the observation is
    // a fact about the record, not a promise that git still reaches the sha.
    run_ok(root, &["import", "git", "--limit", "1"]);
    run_ok(root, &["build"]);
    run_ok(root, &["publish"]);
    git(root, &["add", "--all"]);
    git(root, &["commit", "--amend", "--no-edit"]);

    let rewritten = head_sha(root);
    assert_ne!(
        rewritten, captured,
        "the amend rewrites the captured commit into a new sha"
    );
    let reachable = String::from_utf8(git_stdout(root, &["rev-list", "--all"]))
        .expect("the reachable shas are UTF-8");
    assert!(
        !reachable.split_whitespace().any(|sha| sha == captured),
        "the recorded sha is no longer reachable from any ref: {reachable}"
    );

    let output = fixture.output(&["check"]);

    assert_eq!(
        exit_code(&output),
        0,
        "the rewritten record still verifies against its projections: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    assert_check_envelope(&envelope);
    let facts = record_facts(&envelope);
    assert_eq!(
        facts["last_capture_commit"],
        json!(captured),
        "the frontier is reported as recorded even though the commit is unreachable: {envelope}"
    );
}

#[test]
fn check_sources_and_kinds_canonical() {
    let fixture = Fixture::new();
    let root = fixture.root();

    record_with_two_sources(root, "3");

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

    let counts = source_counts(facts);
    assert_eq!(
        counts["model"], 13,
        "the committed seed's model events count under model: {envelope}"
    );
    assert!(
        counts["wai"] >= 1,
        "the imported research note counts under wai: {envelope}"
    );
    assert!(
        counts["git"] >= 1,
        "the imported commits count under git: {envelope}"
    );
    assert_eq!(
        counts["test-runner"], 0,
        "no run report was imported, so test-runner is zero: {envelope}"
    );
    assert_eq!(
        counts["turu"], 0,
        "no turu bundle was imported, so turu is zero: {envelope}"
    );
    assert!(
        !counts.contains_key("charly"),
        "the seed's human actor name never becomes a source key: {envelope}"
    );
    assert_eq!(
        counts.values().sum::<u64>(),
        facts["events"]
            .as_u64()
            .unwrap_or_else(|| panic!("check reports an integer event count: {envelope}")),
        "the canonical counts partition every event: {envelope}"
    );

    let emitted = dotted_kinds(facts, "emitted_kinds");
    assert!(
        !emitted.is_empty(),
        "the projection emits at least the research map: {envelope}"
    );
    let omitted = dotted_kinds(facts, "omitted_kinds");
    assert!(
        omitted.contains(&"understanding.decision-record".to_owned()),
        "the seed holds no decision, so that kind is omitted for no-records: {envelope}"
    );
}
