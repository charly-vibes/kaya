//! `--index` mode: staged drift, staged ledgers, and unmerged entries.

use serde_json::json;

use crate::common::*;
use crate::tampering::*;

#[test]
fn check_index_only_drift_fails() {
    let fixture = Fixture::new();
    let root = fixture.root();

    // Stage the drifted bytes, then put the worktree back to `HEAD`: the
    // index is the only place the drift lives, exactly as a partial `git add`
    // leaves it.
    corrupt_index_block(root);
    stage(root, INDEX);
    restore_worktree_from_head(root, INDEX);

    assert_eq!(
        worktree_bytes(root, INDEX),
        head_bytes(root, INDEX),
        "the worktree copy is HEAD's clean bytes"
    );
    assert_ne!(
        index_bytes(root, INDEX),
        head_bytes(root, INDEX),
        "the index carries the drifted bytes"
    );

    // The default mode reads the worktree, which a `git commit` would not use
    // for this path, so it sees nothing.
    let worktree = fixture.output(&["check"]);
    assert_eq!(
        exit_code(&worktree),
        0,
        "the clean worktree copy is no drift: {}",
        stderr(&worktree)
    );
    assert!(
        drift_entries(&parse_envelope(&worktree)).is_empty(),
        "the worktree check reports no drift"
    );

    let output = fixture.output(&["check", "--index"]);

    assert_eq!(
        exit_code(&output),
        1,
        "drift staged into the index exits 1: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    assert_check_envelope(&envelope);
    let drift = drift_entries(&envelope);
    assert_eq!(drift.len(), 1, "one drifted block: {envelope}");
    assert_changed(&drift[0], INDEX, INDEX_BLOCK);
}

#[test]
fn check_worktree_noise_ignored() {
    let fixture = Fixture::new();
    let root = fixture.root();

    // Drift the worktree without staging it: the index still holds HEAD's
    // bytes, so a commit of this tree would be clean.
    corrupt_index_block(root);
    assert_ne!(
        worktree_bytes(root, INDEX),
        head_bytes(root, INDEX),
        "the worktree copy is drifted"
    );
    assert_eq!(
        index_bytes(root, INDEX),
        head_bytes(root, INDEX),
        "the staged copy is HEAD's clean bytes"
    );

    let worktree = fixture.output(&["check"]);
    assert_eq!(
        exit_code(&worktree),
        1,
        "the default mode reports worktree drift: {}",
        stderr(&worktree)
    );
    let drift = drift_entries(&parse_envelope(&worktree));
    assert_eq!(drift.len(), 1, "one drifted block in the worktree");
    assert_changed(&drift[0], INDEX, INDEX_BLOCK);

    let output = fixture.output(&["check", "--index"]);

    assert_eq!(
        exit_code(&output),
        0,
        "unstaged worktree noise does not fail an index check: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    assert_check_envelope(&envelope);
    assert!(
        drift_entries(&envelope).is_empty(),
        "index mode reports no drift from worktree bytes: {envelope}"
    );
}

#[test]
fn check_first_commit_verifiable() {
    let fixture = Fixture::staged_first_commit();
    let root = fixture.root();

    // `HEAD` predates the record; the ledger exists only in the index.
    let committed_spec = format!("HEAD:{LEDGER}");
    let committed = git_output(root, &["cat-file", "-e", &committed_spec]);
    assert!(
        !committed.status.success(),
        "the first-commit fixture's HEAD carries no ledger copy"
    );
    assert_eq!(
        index_bytes(root, LEDGER),
        worktree_bytes(root, LEDGER),
        "the staged ledger is the record the worktree holds"
    );
    assert!(
        !index_bytes(root, LEDGER).is_empty(),
        "the staged ledger records events"
    );

    // Without `--index` there is no committed history to verify, so the
    // record is refused rather than silently trusted.
    let worktree = fixture.output(&["check"]);
    assert_eq!(
        exit_code(&worktree),
        2,
        "a ledger HEAD never recorded is refused by the default mode: {}",
        stderr(&worktree)
    );
    assert_eq!(
        parse_envelope(&worktree)["error"]["label"],
        json!("kaya.ledger.uncommitted_history"),
        "the default mode keeps its committed-history refusal"
    );

    let output = fixture.output(&["check", "--index"]);

    assert_eq!(
        exit_code(&output),
        0,
        "the staged first record verifies against empty committed history: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    assert_check_envelope(&envelope);
    assert!(
        drift_entries(&envelope).is_empty(),
        "the staged first record matches its projections: {envelope}"
    );
}

#[test]
fn check_unreadable_index_path_fails() {
    let fixture = Fixture::new();
    let root = fixture.root();

    // Two stages and no stage 0: `git show :docs/research/index.md` fails the
    // way an unresolved merge leaves the index, while the worktree file stays
    // a clean copy of HEAD's bytes.
    make_index_unmerged(root, INDEX);
    assert_eq!(
        worktree_bytes(root, INDEX),
        head_bytes(root, INDEX),
        "the worktree copy stays clean"
    );

    let output = fixture.output(&["check", "--index"]);

    assert_eq!(
        exit_code(&output),
        2,
        "an unreadable index entry is an I/O failure: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    assert_eq!(
        envelope["ok"],
        json!(false),
        "an unreadable index entry is an error envelope: {envelope}"
    );
    assert_eq!(
        envelope["error"]["label"],
        json!("kaya.verification.check_io"),
        "index read failures use the check_io label: {envelope}"
    );
    let message = envelope["error"]["message"]
        .as_str()
        .unwrap_or_else(|| panic!("the I/O failure carries a message: {envelope}"));
    assert!(
        message.contains(INDEX),
        "the failure names the unreadable path: {message}"
    );
    assert!(
        envelope["data"]["drift"].is_null() || envelope["data"]["drift"] == json!([]),
        "no drift finding is reported for an unreadable path: {envelope}"
    );

    // A worktree fallback would have found the clean copy and reported no
    // failure, so the exit code already proves the worktree was not read; the
    // bytes prove the run left it alone.
    assert_eq!(
        worktree_bytes(root, INDEX),
        head_bytes(root, INDEX),
        "the failed check leaves the worktree copy untouched"
    );
}

#[test]
fn check_index_reads() {
    // Index bytes win when they are the drifted side: the clean worktree must
    // not mask staged drift.
    let staged = Fixture::new();
    corrupt_index_block(staged.root());
    stage(staged.root(), INDEX);
    restore_worktree_from_head(staged.root(), INDEX);

    let output = staged.output(&["check", "--index"]);
    assert_eq!(
        exit_code(&output),
        1,
        "staged drift is found even though the worktree is clean: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    assert_check_envelope(&envelope);
    let drift = drift_entries(&envelope);
    assert_eq!(drift.len(), 1, "one drifted block: {envelope}");
    assert_changed(&drift[0], INDEX, INDEX_BLOCK);

    // Index bytes win when the worktree is the drifted side: unstaged bytes
    // must not manufacture drift.
    let noisy = Fixture::new();
    corrupt_index_block(noisy.root());

    let output = noisy.output(&["check", "--index"]);
    assert_eq!(
        exit_code(&output),
        0,
        "worktree drift the index does not hold is ignored: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    assert_check_envelope(&envelope);
    assert!(
        drift_entries(&envelope).is_empty(),
        "index mode reports no finding from worktree bytes: {envelope}"
    );
}

#[test]
fn check_index_reads_the_ledger_from_the_index() {
    let fixture = Fixture::new();
    let root = fixture.root();

    // Tamper with the worktree ledger only: a commit would record the index's
    // committed bytes, which still equal `HEAD`.
    corrupt_ledger_history(root);
    assert_eq!(
        index_bytes(root, LEDGER),
        head_bytes(root, LEDGER),
        "the staged ledger is HEAD's clean bytes"
    );
    assert_ne!(
        worktree_bytes(root, LEDGER),
        head_bytes(root, LEDGER),
        "the worktree ledger no longer extends HEAD"
    );

    // The default mode reads the tampered worktree ledger and refuses it
    // before reporting any drift.
    let worktree = fixture.output(&["check"]);
    assert_eq!(
        exit_code(&worktree),
        2,
        "the tampered worktree ledger is refused: {}",
        stderr(&worktree)
    );
    assert_eq!(
        parse_envelope(&worktree)["error"]["label"],
        json!("kaya.ledger.uncommitted_history"),
        "the default mode keeps its committed-history refusal"
    );

    // Index mode verifies the staged ledger against HEAD, so the worktree's
    // unverified bytes are irrelevant and the record is clean.
    let output = fixture.output(&["check", "--index"]);

    assert_eq!(
        exit_code(&output),
        0,
        "the staged ledger verifies even though the worktree ledger is tampered: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    assert_check_envelope(&envelope);
    assert!(
        drift_entries(&envelope).is_empty(),
        "index mode reports no drift from the tampered worktree ledger: {envelope}"
    );
}

#[test]
fn check_index_reports_stray_managed_document() {
    let fixture = Fixture::new();
    let root = fixture.root();

    // The stray managed document exists only in the index and the worktree:
    // both sides a commit and the worktree check would see agree.
    add_stray_managed_document(root);
    stage(root, STRAY);

    let output = fixture.output(&["check", "--index"]);

    assert_eq!(
        exit_code(&output),
        1,
        "an extra managed document exits 1 in index mode: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    assert_check_envelope(&envelope);
    let drift = drift_entries(&envelope);
    assert_eq!(
        drift.len(),
        1,
        "the staged managed document that the projection does not emit is the only finding: \
         {envelope}"
    );
    assert_document_finding(&drift[0], STRAY, "extra");
}
