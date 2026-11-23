//! `--require-kind` and `--require-source`: named required violations.

use serde_json::{Value, json};

use crate::common::*;
use crate::facts_fixture::*;

fn required_entries(envelope: &Value) -> Vec<Value> {
    envelope["data"]["required"]
        .as_array()
        .unwrap_or_else(|| panic!("check reports data.required as an array: {envelope}"))
        .clone()
}

/// The `data.stale` entries: each stale document path, its authored block, and
/// the evidence ids the block cites.

#[test]
fn check_required_source_turu_absent() {
    let fixture = Fixture::new();

    let output = fixture.output(&["check", "--require-source", "turu"]);

    assert_eq!(
        exit_code(&output),
        1,
        "a required turu source with no events is a violation: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    assert_check_envelope(&envelope);
    assert!(
        drift_entries(&envelope).is_empty(),
        "the committed record itself matches its projections: {envelope}"
    );
    assert_eq!(
        required_entries(&envelope),
        vec![json!({
            "area": "source",
            "name": "turu",
            "reason": "no-events",
        })],
        "the report names the absent turu source and its reason: {envelope}"
    );
}

#[test]
fn check_required_source_turu_satisfied() {
    let fixture = Fixture::new();
    let root = fixture.root();

    record_with_turu_source(root);

    let output = fixture.output(&["check", "--require-source", "turu"]);

    assert_eq!(
        exit_code(&output),
        0,
        "a satisfied turu requirement is not a violation: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    assert_check_envelope(&envelope);
    assert!(
        required_entries(&envelope).is_empty(),
        "the satisfied requirement reports no unmet item: {envelope}"
    );
}

#[test]
fn check_required_kind_omitted() {
    let fixture = Fixture::new();

    let output = fixture.output(&["check", "--require-kind", "understanding.decision-record"]);

    assert_eq!(
        exit_code(&output),
        1,
        "a required kind the projection omits is a violation: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    assert_check_envelope(&envelope);
    assert!(
        drift_entries(&envelope).is_empty(),
        "the committed record itself matches its projections: {envelope}"
    );
    let omitted = dotted_kinds(record_facts(&envelope), "omitted_kinds");
    assert!(
        omitted.contains(&"understanding.decision-record".to_owned()),
        "the seed holds no decision, so the required kind is omitted for no-records: {envelope}"
    );
    assert_eq!(
        required_entries(&envelope),
        vec![json!({
            "area": "kind",
            "name": "understanding.decision-record",
            "reason": "no-records",
        })],
        "the report names the omitted kind and its reason: {envelope}"
    );
}

#[test]
fn check_required_absent_kind() {
    let fixture = Fixture::new();

    // `bogus.kind` is a well-formed dotted name no planner ever emits or
    // omits, so it is neither satisfied nor a family with no records.
    let output = fixture.output(&["check", "--require-kind", "bogus.kind"]);

    assert_eq!(
        exit_code(&output),
        1,
        "a required kind outside both vocabularies is a violation: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    assert_check_envelope(&envelope);
    assert!(
        drift_entries(&envelope).is_empty(),
        "the committed record itself matches its projections: {envelope}"
    );
    let facts = record_facts(&envelope);
    assert!(
        !dotted_kinds(facts, "emitted_kinds").contains(&"bogus.kind".to_owned())
            && !dotted_kinds(facts, "omitted_kinds").contains(&"bogus.kind".to_owned()),
        "the requested kind is in neither vocabulary, so no-records cannot apply: {envelope}"
    );
    assert_eq!(
        required_entries(&envelope),
        vec![json!({
            "area": "kind",
            "name": "bogus.kind",
            "reason": "absent",
        })],
        "a kind no planner names is absent rather than omitted: {envelope}"
    );
}

#[test]
fn check_required_source_absent() {
    let fixture = Fixture::new();

    let output = fixture.output(&["check", "--require-source", "test-runner"]);

    assert_eq!(
        exit_code(&output),
        1,
        "a required source with no events is a violation: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    assert_check_envelope(&envelope);
    assert!(
        drift_entries(&envelope).is_empty(),
        "the committed record itself matches its projections: {envelope}"
    );
    assert_eq!(
        required_entries(&envelope),
        vec![json!({
            "area": "source",
            "name": "test-runner",
            "reason": "no-events",
        })],
        "the report names the absent source and its reason: {envelope}"
    );

    // A requirement outside the canonical vocabulary is a usage failure, not
    // a finding: the check never runs, so it reports nothing to enforce.
    let unknown = fixture.output(&["check", "--require-source", "charly"]);
    assert_eq!(
        exit_code(&unknown),
        2,
        "an unknown source is refused before any comparison: {}",
        stderr(&unknown)
    );
    let envelope = parse_envelope(&unknown);
    assert_eq!(
        envelope["ok"],
        json!(false),
        "a usage failure is an error envelope: {envelope}"
    );
    let label = envelope["error"]["label"]
        .as_str()
        .unwrap_or_else(|| panic!("a usage failure names its label: {envelope}"));
    assert!(
        label.starts_with("kaya.verification."),
        "usage failures stay in the verification namespace: {envelope}"
    );
    assert!(
        envelope["data"].is_null(),
        "a usage failure reports no findings: {envelope}"
    );

    // A kind that is not a dotted `family.kind` name is refused the same way.
    let malformed = fixture.output(&["check", "--require-kind", "decision-record"]);
    assert_eq!(
        exit_code(&malformed),
        2,
        "a malformed required kind is refused before any comparison: {}",
        stderr(&malformed)
    );
    let envelope = parse_envelope(&malformed);
    assert_eq!(
        envelope["ok"],
        json!(false),
        "a usage failure is an error envelope: {envelope}"
    );
    let label = envelope["error"]["label"]
        .as_str()
        .unwrap_or_else(|| panic!("a usage failure names its label: {envelope}"));
    assert!(
        label.starts_with("kaya.verification."),
        "usage failures stay in the verification namespace: {envelope}"
    );
    assert!(
        envelope["data"].is_null(),
        "a usage failure reports no findings: {envelope}"
    );
}

#[test]
fn check_required_items() {
    let fixture = Fixture::new();

    let output = fixture.output(&[
        "check",
        "--require-kind",
        "understanding.decision-record",
        "--require-source",
        "test-runner",
    ]);

    assert_eq!(
        exit_code(&output),
        1,
        "every absent requirement is a violation: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    assert_check_envelope(&envelope);
    assert!(
        drift_entries(&envelope).is_empty(),
        "the record is clean; only the requirements fail: {envelope}"
    );
    let required = required_entries(&envelope);
    assert_eq!(
        required.len(),
        2,
        "both flags are reported, one entry each: {envelope}"
    );
    assert!(
        required.contains(&json!({
            "area": "kind",
            "name": "understanding.decision-record",
            "reason": "no-records",
        })),
        "the absent required kind is named with its reason: {envelope}"
    );
    assert!(
        required.contains(&json!({
            "area": "source",
            "name": "test-runner",
            "reason": "no-events",
        })),
        "the absent required source is named with its reason: {envelope}"
    );
}

#[test]
fn check_required_satisfied_and_repeated() {
    let fixture = Fixture::new();

    // The seed emits `orientation.research-map` and holds model events, so
    // asking twice for the same satisfied sources stays one clean verdict:
    // a requirement names a defect, never a count.
    let output = fixture.output(&[
        "check",
        "--require-kind",
        "orientation.research-map",
        "--require-source",
        "model",
        "--require-source",
        "model",
    ]);

    assert_eq!(
        exit_code(&output),
        0,
        "satisfied requirements are no violation: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    assert_check_envelope(&envelope);
    assert!(
        drift_entries(&envelope).is_empty(),
        "the committed record itself matches its projections: {envelope}"
    );
    assert!(
        dotted_kinds(record_facts(&envelope), "emitted_kinds")
            .contains(&"orientation.research-map".to_owned()),
        "the required kind is emitted by the research index: {envelope}"
    );
    assert_eq!(
        required_entries(&envelope),
        Vec::<Value>::new(),
        "satisfied and repeated requirements report nothing: {envelope}"
    );
}
