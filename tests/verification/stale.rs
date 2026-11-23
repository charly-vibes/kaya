//! `--fail-on-stale-authored`: enforced and advisory staleness, and the
//! index-mode parity of the stale report.

use serde_json::{Value, json};

use crate::common::*;
use crate::facts_fixture::*;
use crate::snapshot::{assert_tree_unchanged, snapshot_tree};
use crate::tampering::*;

fn stale_entries(envelope: &Value) -> Vec<Value> {
    envelope["data"]["stale"]
        .as_array()
        .unwrap_or_else(|| panic!("check reports data.stale as an array: {envelope}"))
        .clone()
}

/// The seven record facts every executed `check` reports.

#[test]
fn check_stale_advisory_by_default() {
    let (fixture, cited) = record_with_a_stale_authored_block();

    let before = snapshot_tree(fixture.root());
    let output = fixture.output(&["check"]);
    let after = snapshot_tree(fixture.root());

    assert_eq!(
        exit_code(&output),
        0,
        "without --fail-on-stale-authored a stale block is advisory: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    assert_eq!(
        envelope["ok"],
        json!(true),
        "the executed check reports ok:true; the exit code carries the verdict: {envelope}"
    );
    assert!(
        drift_entries(&envelope).is_empty(),
        "the committed record itself matches its projections: {envelope}"
    );
    assert_eq!(
        stale_entries(&envelope),
        vec![json!({
            "path": INDEX,
            "block": "authored-1",
            "evidence": [cited.clone()],
        })],
        "the report names the stale document, block, and evidence: {envelope}"
    );
    assert_tree_unchanged(&before, &after);
}

#[test]
fn check_stale_enforced() {
    let (fixture, cited) = record_with_a_stale_authored_block();
    let authored = authored_bytes(fixture.root());

    let before = snapshot_tree(fixture.root());
    let output = fixture.output(&["check", "--fail-on-stale-authored"]);
    let after = snapshot_tree(fixture.root());

    assert_eq!(
        exit_code(&output),
        1,
        "--fail-on-stale-authored turns the stale report into a violation: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    assert_eq!(
        envelope["ok"],
        json!(true),
        "the executed check reports ok:true; the exit code carries the verdict: {envelope}"
    );
    assert!(
        drift_entries(&envelope).is_empty(),
        "the committed record itself matches its projections: {envelope}"
    );
    assert_eq!(
        stale_entries(&envelope),
        vec![json!({
            "path": INDEX,
            "block": "authored-1",
            "evidence": [cited.clone()],
        })],
        "enforcement names the document, block, and changed evidence ids: {envelope}"
    );
    assert_eq!(
        authored_bytes(fixture.root()),
        authored,
        "enforcement reports without rewriting the authored bytes"
    );
    assert_tree_unchanged(&before, &after);
}

#[test]
fn check_stale_violation() {
    let (fixture, cited) = record_with_a_stale_authored_block();
    let authored = authored_bytes(fixture.root());

    let before = snapshot_tree(fixture.root());
    let output = fixture.output(&["check", "--fail-on-stale-authored"]);
    let after = snapshot_tree(fixture.root());

    assert_eq!(
        exit_code(&output),
        1,
        "enforced staleness is a violation: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    assert_eq!(
        envelope["ok"],
        json!(true),
        "the executed check reports ok:true; the exit code carries the verdict: {envelope}"
    );
    let stale = stale_entries(&envelope);
    assert_eq!(stale.len(), 1, "one stale authored block: {envelope}");
    assert_eq!(
        stale[0]["path"],
        json!(INDEX),
        "the violation names the document: {envelope}"
    );
    assert_eq!(
        stale[0]["block"],
        json!("authored-1"),
        "the violation names the block: {envelope}"
    );
    assert_eq!(
        stale[0]["evidence"],
        json!([cited.clone()]),
        "the violation names the changed evidence ids: {envelope}"
    );
    assert_eq!(
        authored_bytes(fixture.root()),
        authored,
        "the violation is reported without rewriting the authored bytes"
    );
    assert_tree_unchanged(&before, &after);
}

#[test]
fn check_index_stale_parity() {
    let (fixture, cited) = record_with_a_stale_authored_block();
    let root = fixture.root();

    // The correction and the document that cites the corrected event are
    // staged, then the worktree copies go back to `HEAD`: the index is the
    // only place the stale citation lives, exactly as a partial `git add`
    // leaves it.
    git(root, &["add", "--all"]);
    restore_worktree_from_head(root, LEDGER);
    restore_worktree_from_head(root, INDEX);

    assert_eq!(
        worktree_bytes(root, INDEX),
        head_bytes(root, INDEX),
        "the worktree copy is HEAD's clean bytes"
    );
    assert_ne!(
        index_bytes(root, LEDGER),
        head_bytes(root, LEDGER),
        "the index carries the correction the citation is judged against"
    );

    let output = fixture.output(&["check", "--index", "--fail-on-stale-authored"]);

    assert_eq!(
        exit_code(&output),
        1,
        "staleness staged into the index is a violation: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    assert_eq!(
        envelope["ok"],
        json!(true),
        "the executed check reports ok:true; the exit code carries the verdict: {envelope}"
    );
    assert_eq!(
        envelope["envelope_kind"],
        json!("check"),
        "check emits its own envelope kind: {envelope}"
    );
    assert!(
        drift_entries(&envelope).is_empty(),
        "the staged tree matches its own projections, so only staleness fails: {envelope}"
    );
    assert_eq!(
        stale_entries(&envelope),
        vec![json!({
            "path": INDEX,
            "block": "authored-1",
            "evidence": [cited.clone()],
        })],
        "index mode names the same document, block, and evidence as the worktree: {envelope}"
    );

    // A worktree-only citation with clean staged copies is invisible to an
    // index check: the commit this tree would record does not hold it, so the
    // stale block is the worktree's, not the index's.
    let (noisy, noisy_cited) = record_with_a_stale_authored_block();
    let root = noisy.root();

    assert_eq!(
        index_bytes(root, LEDGER),
        head_bytes(root, LEDGER),
        "the staged ledger is HEAD's clean bytes"
    );
    assert_eq!(
        index_bytes(root, INDEX),
        head_bytes(root, INDEX),
        "the staged document is HEAD's clean bytes"
    );

    let worktree = noisy.output(&["check", "--fail-on-stale-authored"]);
    assert_eq!(
        exit_code(&worktree),
        1,
        "the worktree run reports the unstaged citation as stale: {}",
        stderr(&worktree)
    );
    assert_eq!(
        stale_entries(&parse_envelope(&worktree)),
        vec![json!({
            "path": INDEX,
            "block": "authored-1",
            "evidence": [noisy_cited],
        })],
        "the worktree run names the document, block, and evidence"
    );

    let output = noisy.output(&["check", "--index", "--fail-on-stale-authored"]);
    assert_eq!(
        exit_code(&output),
        0,
        "unstaged staleness does not fail an index check: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    assert_check_envelope(&envelope);
    assert!(
        stale_entries(&envelope).is_empty(),
        "index mode reports no staleness from worktree bytes: {envelope}"
    );
}

/// A worktree-only citation with clean staged copies is invisible to an index
/// check: the commit this tree would record does not hold it, so the stale
/// block is the worktree's, not the index's.
#[test]
fn check_index_ignores_worktree_only_staleness() {
    let (noisy, noisy_cited) = record_with_a_stale_authored_block();
    let root = noisy.root();

    assert_eq!(
        index_bytes(root, LEDGER),
        head_bytes(root, LEDGER),
        "the staged ledger is HEAD's clean bytes"
    );
    assert_eq!(
        index_bytes(root, INDEX),
        head_bytes(root, INDEX),
        "the staged document is HEAD's clean bytes"
    );

    let worktree = noisy.output(&["check", "--fail-on-stale-authored"]);
    assert_eq!(
        exit_code(&worktree),
        1,
        "the worktree run reports the unstaged citation as stale: {}",
        stderr(&worktree)
    );
    assert_eq!(
        stale_entries(&parse_envelope(&worktree)),
        vec![json!({
            "path": INDEX,
            "block": "authored-1",
            "evidence": [noisy_cited],
        })],
        "the worktree run names the document, block, and evidence"
    );

    let output = noisy.output(&["check", "--index", "--fail-on-stale-authored"]);
    assert_eq!(
        exit_code(&output),
        0,
        "unstaged staleness does not fail an index check: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    assert_check_envelope(&envelope);
    assert!(
        stale_entries(&envelope).is_empty(),
        "index mode reports no staleness from worktree bytes: {envelope}"
    );
}
