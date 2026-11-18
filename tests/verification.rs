//! Integration tests for `kaya check`, the read-only record verifier.
//!
//! Every test builds a disposable repository, imports the committed model
//! seed, commits the ledger, compiles the record with `kaya build` and
//! `kaya publish`, and commits the generated documents. One artifact is then
//! tampered with and `check` must report the drift from the worktree without
//! writing: a changed block body is `changed`, an absent generated document is
//! `missing`, a managed document the projection does not emit is `extra`, a
//! tampered publication page is one `changed` block finding, an absent
//! publication export is a `missing` document, a worktree ledger that no
//! longer extends `HEAD` is refused before any finding, and the exit code
//! carries the verdict (0 clean, 1 drift, 2 usage or I/O).
//!
//! The fixtures follow `tests/acceptance.rs` and `tests/cli.rs`: the built
//! binary runs in a `tempfile::TempDir`, envelopes are parsed as JSON, and
//! commits use a fixed identity and dates.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use assert_cmd::Command as AssertCommand;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tempfile::TempDir;

const LEDGER: &str = ".kaya/ledger.jsonl";
const SEED: &str = ".kaya/seeds/add-core-research-record.jsonl";
const INDEX: &str = "docs/research/index.md";
const INDEX_BLOCK: &str = "orientation.research-map";
const INDEX_BEGIN: &str = "<!-- kaya:begin generated:orientation.research-map -->";
const INDEX_END: &str = "<!-- kaya:end generated:orientation.research-map -->";
const STRAY: &str = "docs/research/experiments/stray.md";
const PUBLICATION_OVERVIEW: &str = "docs/public/overview.md";
const PUBLICATION_OVERVIEW_BLOCK: &str = "publication.product.overview";
const PUBLICATION_OVERVIEW_BEGIN: &str =
    "<!-- kaya:begin generated:publication.product.overview -->";
const PUBLICATION_EXPORT: &str = "docs/public/publication.json";

/// Authored prose the fixture seeds before `build` ever runs.
const AUTHORED_PROSE: &str = "Human interpretation of the compiled record.";

/// A foreign tool's managed block: bytes kaya never interprets, writes, or
/// compares, so `check` must leave it out of every finding.
const FOREIGN_BLOCK: &str = "\
<!-- other-tool:begin -->
foreign
<!-- other-tool:end -->
";

/// A committed record in a throwaway repository.
///
/// The ledger is committed before `build` (so git can verify its history) and
/// the generated documents are committed afterwards, once `build` and
/// `publish` have written them (so they are the committed projections `check`
/// compares against). The research index is seeded with authored prose and a
/// foreign managed block first, so every fixture proves `check` preserves
/// bytes kaya does not own.
struct Fixture {
    dir: TempDir,
}

impl Fixture {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("temp workspace");
        let root = dir.path();

        git(root, &["init", "--quiet"]);
        git(root, &["config", "user.name", "kaya verification"]);
        git(
            root,
            &["config", "user.email", "verification@example.invalid"],
        );

        run_ok(root, &["init"]);
        let seed = manifest_dir().join(SEED);
        let seed = seed.to_str().expect("the seed path is UTF-8");
        run_ok(root, &["import", "model", "--file", seed]);

        git(root, &["add", "--all"]);
        git(
            root,
            &["commit", "--quiet", "--message", "record the model seed"],
        );

        seed_authored_index(root);
        run_ok(root, &["build"]);
        run_ok(root, &["publish"]);

        git(root, &["add", "--all"]);
        git(
            root,
            &[
                "commit",
                "--quiet",
                "--message",
                "commit the compiled record",
            ],
        );

        let index = fs::read_to_string(root.join(INDEX)).expect("build writes the research index");
        assert!(
            index.contains(INDEX_BEGIN),
            "the fixture index carries the managed research map block: {index}"
        );
        assert!(
            index.contains(FOREIGN_BLOCK) && index.contains(AUTHORED_PROSE),
            "the projection preserves the seeded authored and foreign bytes: {index}"
        );

        Self { dir }
    }

    fn root(&self) -> &Path {
        self.dir.path()
    }

    fn output(&self, args: &[&str]) -> Output {
        self.output_in(self.root(), args)
    }

    fn output_in(&self, cwd: &Path, args: &[&str]) -> Output {
        let mut command = AssertCommand::cargo_bin("kaya").expect("kaya binary is built");
        command.current_dir(cwd).args(args);
        command.output().expect("kaya runs")
    }
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Run the built binary, requiring exit 0 and an ok envelope.
fn run_ok(workspace: &Path, args: &[&str]) -> Value {
    let mut command = AssertCommand::cargo_bin("kaya").expect("kaya binary is built");
    command.current_dir(workspace).args(args);
    let output = command.output().expect("kaya runs");
    assert!(
        output.status.success(),
        "kaya {args:?} exited with {:?}: {}",
        output.status.code(),
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    assert_eq!(
        envelope["ok"],
        json!(true),
        "kaya {args:?} returned an error envelope: {envelope}"
    );
    envelope
}

/// One deterministic commit: fixed identity, fixed dates, no user config.
fn git(workspace: &Path, args: &[&str]) {
    let output = Command::new("git")
        .arg("-C")
        .arg(workspace)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .env("GIT_AUTHOR_NAME", "kaya verification")
        .env("GIT_AUTHOR_EMAIL", "verification@example.invalid")
        .env("GIT_COMMITTER_NAME", "kaya verification")
        .env("GIT_COMMITTER_EMAIL", "verification@example.invalid")
        .env("GIT_AUTHOR_DATE", "2026-11-18T00:00:00+00:00")
        .env("GIT_COMMITTER_DATE", "2026-11-18T00:00:00+00:00")
        .args(args)
        .output()
        .expect("git runs");
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn parse_envelope(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|err| {
        panic!(
            "expected one JSON envelope on stdout ({err}); stderr: {}",
            stderr(output)
        )
    })
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn exit_code(output: &Output) -> i32 {
    output
        .status
        .code()
        .unwrap_or_else(|| panic!("kaya exited without a code: {}", stderr(output)))
}

/// The `check` success envelope: `ok: true`, its own kind, empty staleness.
fn assert_check_envelope(envelope: &Value) {
    assert_eq!(
        envelope["ok"],
        json!(true),
        "an executed check reports ok:true; the exit code carries the verdict: {envelope}"
    );
    assert_eq!(
        envelope["envelope_kind"],
        json!("check"),
        "check emits its own envelope kind: {envelope}"
    );
    assert!(
        envelope.get("error").is_none(),
        "an executed check omits the error field: {envelope}"
    );
    let stale = envelope["data"]["stale"]
        .as_array()
        .unwrap_or_else(|| panic!("check reports data.stale as an array: {envelope}"));
    assert!(
        stale.is_empty(),
        "this slice reports no staleness findings: {envelope}"
    );
}

fn drift_entries(envelope: &Value) -> Vec<Value> {
    envelope["data"]["drift"]
        .as_array()
        .unwrap_or_else(|| panic!("check reports data.drift as an array: {envelope}"))
        .clone()
}

/// A block-body finding names the document, the block, and `changed`.
fn assert_changed(entry: &Value, path: &str, block: &str) {
    assert_eq!(entry["path"], json!(path), "drift path: {entry}");
    assert_eq!(
        entry["block"],
        json!(block),
        "a block-body finding names its block: {entry}"
    );
    assert_eq!(entry["kind"], json!("changed"), "drift kind: {entry}");
}

/// A document-level finding names the document and its violation kind; the
/// `block` field is present as a string or null.
fn assert_document_finding(entry: &Value, path: &str, kind: &str) {
    assert_eq!(entry["path"], json!(path), "drift path: {entry}");
    assert_eq!(entry["kind"], json!(kind), "drift kind: {entry}");
    match entry.get("block") {
        Some(value) if value.is_null() || value.is_string() => {}
        other => {
            panic!("a drift entry carries `block` as a string or null, got {other:?}: {entry}")
        }
    }
}

/// Seed the research index with authored prose and a foreign managed block
/// before the projection appends its own block.
fn seed_authored_index(root: &Path) {
    let path = root.join(INDEX);
    fs::create_dir_all(path.parent().expect("the index has a parent"))
        .expect("create the research directory");
    let mut document = String::from("# Research index\n\n");
    document.push_str(FOREIGN_BLOCK);
    document.push_str("\n<!-- kaya:authored -->\n");
    document.push_str(AUTHORED_PROSE);
    document.push_str("\n<!-- kaya:end authored -->\n");
    fs::write(&path, document).expect("seed the authored research index");
}

/// Rewrite the first body byte of the generated block starting at `begin` in
/// `relative`, leaving the surrounding bytes untouched.
fn corrupt_generated_block(root: &Path, relative: &str, begin: &str) {
    let path = root.join(relative);
    let text = fs::read_to_string(&path).expect("the generated document is readable");
    let after_marker = text
        .find(begin)
        .unwrap_or_else(|| panic!("{relative} carries the generated marker {begin}"))
        + begin.len();
    let body = text[after_marker..]
        .find('\n')
        .map(|offset| after_marker + offset + 1)
        .expect("the managed block marker ends its line");
    let mut bytes = text.into_bytes();
    bytes[body] = if bytes[body] == b'#' { b'!' } else { b'#' };
    fs::write(&path, bytes).expect("write the drifted document");
}

/// Rewrite the first body byte of the index's managed research-map block.
fn corrupt_index_block(root: &Path) {
    corrupt_generated_block(root, INDEX, INDEX_BEGIN);
}

/// Remove the publication export the declarations require.
fn remove_publication_export(root: &Path) {
    fs::remove_file(root.join(PUBLICATION_EXPORT)).expect("remove the publication export");
}

/// Delete the generated document the projection requires.
fn remove_generated_index(root: &Path) {
    fs::remove_file(root.join(INDEX)).expect("remove the generated index");
}

/// Add a managed document that the projection would not emit.
fn add_stray_managed_document(root: &Path) {
    let path = root.join(STRAY);
    fs::create_dir_all(path.parent().expect("stray path has a parent"))
        .expect("create the stray document directory");
    let document = format!("{INDEX_BEGIN}\n### Stray research map\n{INDEX_END}\n");
    fs::write(&path, document).expect("write the stray managed document");
}

/// Rewrite one byte inside the first committed ledger line, keeping it valid
/// JSON while breaking the prefix relation with `HEAD`.
fn corrupt_ledger_history(root: &Path) {
    let path = root.join(LEDGER);
    let text = fs::read_to_string(&path).expect("the committed ledger is readable");
    let (first, rest) = text
        .split_once('\n')
        .expect("the committed ledger holds a line");
    let tampered = first.replace("\"charly\"", "\"charlx\"");
    assert_ne!(
        first, tampered,
        "the seed's first ledger line names its actor: {first}"
    );
    fs::write(&path, format!("{tampered}\n{rest}")).expect("rewrite the ledger worktree bytes");
}

/// Every directory path and every regular file's digest under `root`.
struct TreeSnapshot {
    directories: BTreeSet<PathBuf>,
    files: BTreeMap<PathBuf, String>,
}

fn snapshot_tree(root: &Path) -> TreeSnapshot {
    let mut snapshot = TreeSnapshot {
        directories: BTreeSet::new(),
        files: BTreeMap::new(),
    };
    collect_tree(root, root, &mut snapshot);
    snapshot
}

fn collect_tree(root: &Path, dir: &Path, snapshot: &mut TreeSnapshot) {
    let entries = fs::read_dir(dir).unwrap_or_else(|err| panic!("read {}: {err}", dir.display()));
    for entry in entries {
        let entry = entry.unwrap_or_else(|err| panic!("read an entry of {}: {err}", dir.display()));
        let path = entry.path();
        let file_type = entry.file_type().expect("tree entry type");
        let relative = path
            .strip_prefix(root)
            .expect("path below the fixture root")
            .to_path_buf();
        if file_type.is_dir() {
            snapshot.directories.insert(relative);
            collect_tree(root, &path, snapshot);
        } else if file_type.is_file() {
            // Git spawns auto-maintenance asynchronously; its transient lock
            // files appear and vanish without any check involvement.
            if path
                .extension()
                .is_some_and(|extension| extension == "lock")
            {
                continue;
            }
            let bytes =
                fs::read(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
            snapshot.files.insert(relative, sha256_hex(&bytes));
        }
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn assert_tree_unchanged(before: &TreeSnapshot, after: &TreeSnapshot) {
    let created = after
        .directories
        .difference(&before.directories)
        .collect::<Vec<_>>();
    assert!(created.is_empty(), "check created directories: {created:?}");
    let removed = before
        .directories
        .difference(&after.directories)
        .collect::<Vec<_>>();
    assert!(removed.is_empty(), "check removed directories: {removed:?}");

    let created = after
        .files
        .keys()
        .filter(|path| !before.files.contains_key(*path))
        .collect::<Vec<_>>();
    assert!(created.is_empty(), "check created files: {created:?}");
    let removed = before
        .files
        .keys()
        .filter(|path| !after.files.contains_key(*path))
        .collect::<Vec<_>>();
    assert!(removed.is_empty(), "check removed files: {removed:?}");
    let changed = before
        .files
        .iter()
        .filter(|(path, digest)| after.files.get(*path) != Some(*digest))
        .map(|(path, _)| path)
        .collect::<Vec<_>>();
    assert!(changed.is_empty(), "check rewrote files: {changed:?}");
}

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
