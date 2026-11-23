//! Worktree drift: tampered bytes, missing documents, and refusals.

use std::fs;

use serde_json::json;

use crate::common::*;
use crate::snapshot::{assert_tree_unchanged, snapshot_tree};
use crate::tampering::*;

#[test]
fn check_reports_drift_without_writing() {
    let fixture = Fixture::new();
    corrupt_index_block(fixture.root());

    let before = snapshot_tree(fixture.root());
    let output = fixture.output(&["check"]);
    let after = snapshot_tree(fixture.root());

    assert_eq!(exit_code(&output), 1, "drift exits 1: {}", stderr(&output));
    let envelope = parse_envelope(&output);
    assert_check_envelope(&envelope);
    let drift = drift_entries(&envelope);
    assert_eq!(drift.len(), 1, "one drifted block: {envelope}");
    assert_changed(&drift[0], INDEX, INDEX_BLOCK);
    assert_tree_unchanged(&before, &after);
}

#[test]
fn check_refuses_unverifiable_ledger() {
    let fixture = Fixture::new();
    corrupt_ledger_history(fixture.root());

    let output = fixture.output(&["check"]);

    assert_eq!(
        exit_code(&output),
        2,
        "an unverifiable ledger is refused with exit 2: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    assert_eq!(
        envelope["ok"],
        json!(false),
        "a refusal is an error envelope: {envelope}"
    );
    assert_eq!(
        envelope["error"]["label"],
        json!("kaya.ledger.uncommitted_history"),
        "the ledger refusal reuses the committed-history label: {envelope}"
    );
    let drift = envelope["data"].get("drift");
    assert!(
        drift.is_none() || drift == Some(&json!([])),
        "an unverifiable ledger is refused before reporting any finding: {envelope}"
    );
}

#[test]
fn check_reports_drifted_block() {
    let fixture = Fixture::new();
    corrupt_index_block(fixture.root());

    let output = fixture.output(&["check"]);

    assert_eq!(
        exit_code(&output),
        1,
        "a drifted block exits 1: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    assert_check_envelope(&envelope);
    let drift = drift_entries(&envelope);
    assert_eq!(drift.len(), 1, "one drifted block: {envelope}");
    assert_changed(&drift[0], INDEX, INDEX_BLOCK);
}

#[test]
fn check_clean_record_exits_zero() {
    let fixture = Fixture::new();

    let output = fixture.output(&["check"]);

    assert_eq!(
        exit_code(&output),
        0,
        "a clean record exits 0: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    assert_check_envelope(&envelope);
    assert!(
        drift_entries(&envelope).is_empty(),
        "a clean record reports an empty drift list: {envelope}"
    );

    // Authored prose and a foreign tool's block are not kaya's to compare:
    // the clean verdict proves they are not drift, and the bytes prove the
    // read-only check left them exactly as `build` wrote around them.
    let index = fs::read_to_string(fixture.root().join(INDEX)).expect("the index is readable");
    assert!(
        index.contains(FOREIGN_BLOCK),
        "the foreign managed block survives: {index}"
    );
    assert!(
        index.contains(AUTHORED_PROSE),
        "the authored prose survives: {index}"
    );
}

#[test]
fn check_reports_missing_document() {
    let fixture = Fixture::new();
    remove_generated_index(fixture.root());

    let output = fixture.output(&["check"]);

    assert_eq!(
        exit_code(&output),
        1,
        "a missing generated document exits 1: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    assert_check_envelope(&envelope);
    let drift = drift_entries(&envelope);
    assert_eq!(drift.len(), 1, "one missing document: {envelope}");
    assert_document_finding(&drift[0], INDEX, "missing");
}

#[test]
fn check_usage_error_exits_two() {
    let fixture = Fixture::new();

    let output = fixture.output(&["check", "--expect-digest", "not-a-digest"]);

    assert_eq!(
        exit_code(&output),
        2,
        "an invalid flag value is a usage failure: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    assert_eq!(
        envelope["ok"],
        json!(false),
        "a usage failure is an error envelope: {envelope}"
    );
    let label = envelope["error"]["label"]
        .as_str()
        .unwrap_or_else(|| panic!("a usage failure carries an error label: {envelope}"));
    assert!(
        label.starts_with("kaya.verification."),
        "usage failures use the verification namespace: {envelope}"
    );
}

#[test]
fn check_paths_follow_root() {
    let fixture = Fixture::new();
    let nested = fixture.root().join("nested");
    fs::create_dir(&nested).expect("create the invocation subdirectory");
    corrupt_index_block(fixture.root());

    let root = fixture.root().to_str().expect("the root path is UTF-8");
    let output = fixture.output_in(&nested, &["check", "--root", root]);

    assert_eq!(
        exit_code(&output),
        1,
        "the drifted record is found through --root: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    assert_check_envelope(&envelope);
    let drift = drift_entries(&envelope);
    assert_eq!(drift.len(), 1, "one drifted block: {envelope}");
    assert_changed(&drift[0], INDEX, INDEX_BLOCK);
}

#[test]
fn check_has_no_side_effects() {
    let fixture = Fixture::new();
    corrupt_index_block(fixture.root());

    let before = snapshot_tree(fixture.root());
    let output = fixture.output(&["check"]);
    let after = snapshot_tree(fixture.root());

    assert_eq!(
        exit_code(&output),
        1,
        "the drifted record is still reported: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    assert_check_envelope(&envelope);
    let drift = drift_entries(&envelope);
    assert_eq!(drift.len(), 1, "one drifted block: {envelope}");
    assert_changed(&drift[0], INDEX, INDEX_BLOCK);
    assert_tree_unchanged(&before, &after);
}

#[test]
fn check_drift_exit() {
    let fixture = Fixture::new();
    add_stray_managed_document(fixture.root());

    let output = fixture.output(&["check"]);

    assert_eq!(
        exit_code(&output),
        1,
        "an extra managed document exits 1: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    assert_check_envelope(&envelope);
    let drift = drift_entries(&envelope);
    assert_eq!(
        drift.len(),
        1,
        "the stray managed document is the only finding: {envelope}"
    );
    assert_document_finding(&drift[0], STRAY, "extra");
}

#[test]
fn check_reports_publication_drift() {
    let fixture = Fixture::new();
    corrupt_generated_block(
        fixture.root(),
        PUBLICATION_OVERVIEW,
        PUBLICATION_OVERVIEW_BEGIN,
    );

    let output = fixture.output(&["check"]);

    assert_eq!(
        exit_code(&output),
        1,
        "a drifted publication page exits 1: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    assert_check_envelope(&envelope);
    let drift = drift_entries(&envelope);
    assert_eq!(
        drift.len(),
        1,
        "one drifted publication block is one finding: {envelope}"
    );
    assert_changed(&drift[0], PUBLICATION_OVERVIEW, PUBLICATION_OVERVIEW_BLOCK);
}

#[test]
fn check_reports_missing_publication_export() {
    let fixture = Fixture::new();
    remove_publication_export(fixture.root());

    let output = fixture.output(&["check"]);

    assert_eq!(
        exit_code(&output),
        1,
        "a missing publication export exits 1: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    assert_check_envelope(&envelope);
    let drift = drift_entries(&envelope);
    assert_eq!(drift.len(), 1, "one missing export: {envelope}");
    let entry = &drift[0];
    assert_document_finding(entry, PUBLICATION_EXPORT, "missing");
    assert!(
        entry["block"].is_null(),
        "a document-level finding carries a null block: {entry}"
    );
}
