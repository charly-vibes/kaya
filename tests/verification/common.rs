//! Shared fixture, git, and envelope helpers for the `check` suite.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use assert_cmd::Command as AssertCommand;
use serde_json::{Value, json};
use tempfile::TempDir;

use crate::tampering::seed_authored_index;

pub(crate) const LEDGER: &str = ".kaya/ledger.jsonl";
pub(crate) const SEED: &str = ".kaya/seeds/add-core-research-record.jsonl";
pub(crate) const INDEX: &str = "docs/research/index.md";
pub(crate) const INDEX_BLOCK: &str = "orientation.research-map";
pub(crate) const INDEX_BEGIN: &str = "<!-- kaya:begin generated:orientation.research-map -->";
pub(crate) const INDEX_END: &str = "<!-- kaya:end generated:orientation.research-map -->";
pub(crate) const STRAY: &str = "docs/research/experiments/stray.md";
pub(crate) const PUBLICATION_OVERVIEW: &str = "docs/public/overview.md";
pub(crate) const PUBLICATION_OVERVIEW_BLOCK: &str = "publication.product.overview";
pub(crate) const PUBLICATION_OVERVIEW_BEGIN: &str =
    "<!-- kaya:begin generated:publication.product.overview -->";
pub(crate) const PUBLICATION_EXPORT: &str = "docs/public/publication.json";

/// Authored prose the fixture seeds before `build` ever runs.
pub(crate) const AUTHORED_PROSE: &str = "Human interpretation of the compiled record.";

/// The human-owned region of the research index, as `build` and `publish`
/// must leave it.
pub(crate) const AUTHORED_BEGIN: &str = "<!-- kaya:authored -->";
pub(crate) const AUTHORED_END: &str = "<!-- kaya:end authored -->";

/// A foreign tool's managed block: bytes kaya never interprets, writes, or
/// compares, so `check` must leave it out of every finding.
pub(crate) const FOREIGN_BLOCK: &str = "\
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
pub(crate) struct Fixture {
    dir: TempDir,
}

impl Fixture {
    pub(crate) fn new() -> Self {
        let dir = tempfile::tempdir().expect("temp workspace");
        let root = dir.path();

        init_repository(root);
        create_record(root);

        git(root, &["add", "--all"]);
        git(
            root,
            &["commit", "--quiet", "--message", "record the model seed"],
        );

        compile_projection(root, false);
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
        Self { dir }
    }

    /// A first-commit record: the only commit predates the record, and the
    /// ledger with its projections is staged but never committed, so `HEAD`
    /// carries no ledger copy for `check --index` to verify against.
    ///
    /// `build --allow-uncommitted` compiles the projections while the ledger
    /// has no committed history; staging the whole tree afterwards is the
    /// first record a commit would create.
    pub(crate) fn staged_first_commit() -> Self {
        let dir = tempfile::tempdir().expect("temp workspace");
        let root = dir.path();

        init_repository(root);
        fs::write(
            root.join("README.md"),
            "# a repository before its research record\n",
        )
        .expect("write the pre-record commit's file");
        git(root, &["add", "--", "README.md"]);
        git(
            root,
            &["commit", "--quiet", "--message", "open the repository"],
        );

        create_record(root);
        compile_projection(root, true);
        git(root, &["add", "--all"]);
        Self { dir }
    }

    pub(crate) fn root(&self) -> &Path {
        self.dir.path()
    }

    pub(crate) fn output(&self, args: &[&str]) -> Output {
        self.output_in(self.root(), args)
    }

    pub(crate) fn output_in(&self, cwd: &Path, args: &[&str]) -> Output {
        let mut command = AssertCommand::cargo_bin("kaya").expect("kaya binary is built");
        command.current_dir(cwd).args(args);
        command.output().expect("kaya runs")
    }
}

/// A git repository with a fixed identity and no user configuration.
pub(crate) fn init_repository(root: &Path) {
    git(root, &["init", "--quiet"]);
    git(root, &["config", "user.name", "kaya verification"]);
    git(
        root,
        &["config", "user.email", "verification@example.invalid"],
    );
}

/// Create the ledger and import the committed model seed, leaving every byte
/// uncommitted for the caller to stage.
pub(crate) fn create_record(root: &Path) {
    run_ok(root, &["init"]);
    let seed = manifest_dir().join(SEED);
    let seed = seed.to_str().expect("the seed path is UTF-8");
    run_ok(root, &["import", "model", "--file", seed]);
}

/// Seed the authored index, then compile and publish the projections.
///
/// `allow_uncommitted` compiles a ledger git cannot verify against `HEAD`,
/// which is exactly the first-commit fixture's state.
pub(crate) fn compile_projection(root: &Path, allow_uncommitted: bool) {
    seed_authored_index(root);
    let mut build = vec!["build"];
    if allow_uncommitted {
        build.push("--allow-uncommitted");
    }
    run_ok(root, &build);
    run_ok(root, &["publish"]);

    let index = fs::read_to_string(root.join(INDEX)).expect("build writes the research index");
    assert!(
        index.contains(INDEX_BEGIN),
        "the fixture index carries the managed research map block: {index}"
    );
    assert!(
        index.contains(FOREIGN_BLOCK) && index.contains(AUTHORED_PROSE),
        "the projection preserves the seeded authored and foreign bytes: {index}"
    );
}

pub(crate) fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Run the built binary, requiring exit 0 and an ok envelope.
pub(crate) fn run_ok(workspace: &Path, args: &[&str]) -> Value {
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

/// A git command with a deterministic identity, dates, and no user config.
pub(crate) fn git_command(workspace: &Path) -> Command {
    let mut command = Command::new("git");
    command
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
        .env("GIT_COMMITTER_DATE", "2026-11-18T00:00:00+00:00");
    command
}

/// Run one git command and capture its output without requiring success.
pub(crate) fn git_output(workspace: &Path, args: &[&str]) -> Output {
    git_command(workspace)
        .args(args)
        .output()
        .expect("git runs")
}

/// Run one git command with `input` on stdin, capturing its output.
pub(crate) fn git_with_stdin(workspace: &Path, args: &[&str], input: &[u8]) -> Output {
    let mut child = git_command(workspace)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("git runs");
    child
        .stdin
        .take()
        .expect("git stdin is piped")
        .write_all(input)
        .expect("write git stdin");
    child.wait_with_output().expect("git finishes")
}

/// One deterministic commit: fixed identity, fixed dates, no user config.
pub(crate) fn git(workspace: &Path, args: &[&str]) {
    let output = git_output(workspace, args);
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// The bytes one git command writes to stdout, requiring exit 0.
pub(crate) fn git_stdout(workspace: &Path, args: &[&str]) -> Vec<u8> {
    let output = git_output(workspace, args);
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

/// The commit `HEAD` names.
pub(crate) fn head_sha(workspace: &Path) -> String {
    String::from_utf8(git_stdout(workspace, &["rev-parse", "HEAD"]))
        .expect("the commit sha is UTF-8")
        .trim()
        .to_owned()
}

/// Commit the staged tree stamped with explicit author and committer dates, so
/// a fixture can put a later author date on an earlier log entry.
pub(crate) fn commit_dated(workspace: &Path, message: &str, author: &str, committer: &str) {
    let output = git_command(workspace)
        .args(["commit", "--quiet", "--message", message])
        .env("GIT_AUTHOR_DATE", author)
        .env("GIT_COMMITTER_DATE", committer)
        .output()
        .expect("git runs");
    assert!(
        output.status.success(),
        "git commit {message:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

pub(crate) fn parse_envelope(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|err| {
        panic!(
            "expected one JSON envelope on stdout ({err}); stderr: {}",
            stderr(output)
        )
    })
}

pub(crate) fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

pub(crate) fn exit_code(output: &Output) -> i32 {
    output
        .status
        .code()
        .unwrap_or_else(|| panic!("kaya exited without a code: {}", stderr(output)))
}

/// The `check` success envelope: `ok: true`, its own kind, empty staleness.
pub(crate) fn assert_check_envelope(envelope: &Value) {
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

pub(crate) fn drift_entries(envelope: &Value) -> Vec<Value> {
    envelope["data"]["drift"]
        .as_array()
        .unwrap_or_else(|| panic!("check reports data.drift as an array: {envelope}"))
        .clone()
}

/// A block-body finding names the document, the block, and `changed`.
pub(crate) fn assert_changed(entry: &Value, path: &str, block: &str) {
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
pub(crate) fn assert_document_finding(entry: &Value, path: &str, kind: &str) {
    assert_eq!(entry["path"], json!(path), "drift path: {entry}");
    assert_eq!(entry["kind"], json!(kind), "drift kind: {entry}");
    match entry.get("block") {
        Some(value) if value.is_null() || value.is_string() => {}
        other => {
            panic!("a drift entry carries `block` as a string or null, got {other:?}: {entry}")
        }
    }
}
