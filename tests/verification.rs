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
//! The `--index` fixtures do the same over the git index: the ledger and every
//! compared document are read from `:<path>`, so staged drift is reported even
//! when the worktree is clean, worktree noise is ignored, a staged ledger in a
//! repository whose `HEAD` predates it is verifiable, and an unmerged index
//! entry is an I/O failure rather than a worktree fallback.
//!
//! The fixtures follow `tests/acceptance.rs` and `tests/cli.rs`: the built
//! binary runs in a `tempfile::TempDir`, envelopes are parsed as JSON, and
//! commits use a fixed identity and dates.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

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
    fn staged_first_commit() -> Self {
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

/// A git repository with a fixed identity and no user configuration.
fn init_repository(root: &Path) {
    git(root, &["init", "--quiet"]);
    git(root, &["config", "user.name", "kaya verification"]);
    git(
        root,
        &["config", "user.email", "verification@example.invalid"],
    );
}

/// Create the ledger and import the committed model seed, leaving every byte
/// uncommitted for the caller to stage.
fn create_record(root: &Path) {
    run_ok(root, &["init"]);
    let seed = manifest_dir().join(SEED);
    let seed = seed.to_str().expect("the seed path is UTF-8");
    run_ok(root, &["import", "model", "--file", seed]);
}

/// Seed the authored index, then compile and publish the projections.
///
/// `allow_uncommitted` compiles a ledger git cannot verify against `HEAD`,
/// which is exactly the first-commit fixture's state.
fn compile_projection(root: &Path, allow_uncommitted: bool) {
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

/// A git command with a deterministic identity, dates, and no user config.
fn git_command(workspace: &Path) -> Command {
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
fn git_output(workspace: &Path, args: &[&str]) -> Output {
    git_command(workspace)
        .args(args)
        .output()
        .expect("git runs")
}

/// Run one git command with `input` on stdin, capturing its output.
fn git_with_stdin(workspace: &Path, args: &[&str], input: &[u8]) -> Output {
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
fn git(workspace: &Path, args: &[&str]) {
    let output = git_output(workspace, args);
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// The bytes one git command writes to stdout, requiring exit 0.
fn git_stdout(workspace: &Path, args: &[&str]) -> Vec<u8> {
    let output = git_output(workspace, args);
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

/// The commit `HEAD` names.
fn head_sha(workspace: &Path) -> String {
    String::from_utf8(git_stdout(workspace, &["rev-parse", "HEAD"]))
        .expect("the commit sha is UTF-8")
        .trim()
        .to_owned()
}

/// Commit the staged tree stamped with explicit author and committer dates, so
/// a fixture can put a later author date on an earlier log entry.
fn commit_dated(workspace: &Path, message: &str, author: &str, committer: &str) {
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

/// Stage `relative` exactly as the worktree holds it now.
fn stage(root: &Path, relative: &str) {
    git(root, &["add", "--", relative]);
}

/// Restore the worktree copy of `relative` to the bytes `HEAD` records.
fn restore_worktree_from_head(root: &Path, relative: &str) {
    let spec = format!("HEAD:{relative}");
    let bytes = git_stdout(root, &["show", &spec]);
    fs::write(root.join(relative), bytes).expect("restore the worktree document from HEAD");
}

/// The worktree bytes of `relative`.
fn worktree_bytes(root: &Path, relative: &str) -> Vec<u8> {
    fs::read(root.join(relative)).unwrap_or_else(|error| {
        panic!("the worktree copy of {relative} is readable: {error}");
    })
}

/// The staged (index) bytes of `relative`, as `git show :<path>` reports them.
fn index_bytes(root: &Path, relative: &str) -> Vec<u8> {
    let spec = format!(":{relative}");
    git_stdout(root, &["show", &spec])
}

/// The committed `HEAD` bytes of `relative`.
fn head_bytes(root: &Path, relative: &str) -> Vec<u8> {
    let spec = format!("HEAD:{relative}");
    git_stdout(root, &["show", &spec])
}

/// Put `relative` in the index as two unmerged stages, leaving the worktree
/// copy exactly as it was.
fn make_index_unmerged(root: &Path, relative: &str) {
    let spec = format!(":{relative}");
    let sha = String::from_utf8(git_stdout(root, &["rev-parse", &spec]))
        .expect("the blob id is UTF-8")
        .trim()
        .to_owned();

    git(root, &["update-index", "--force-remove", "--", relative]);
    let info = format!("100644 {sha} 1\t{relative}\n100644 {sha} 2\t{relative}\n");
    let output = git_with_stdin(root, &["update-index", "--index-info"], info.as_bytes());
    assert!(
        output.status.success(),
        "the index records two unmerged stages for {relative}: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let unmerged = git_output(root, &["show", &spec]);
    assert!(
        !unmerged.status.success(),
        "an unmerged {relative} cannot be read from the index: {}",
        String::from_utf8_lossy(&unmerged.stdout)
    );
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

/// A dated wai research artifact; the adapter records one observation per
/// markdown file under `.wai/projects/<project>/<kind>/`.
const WAI_NOTE: &str = ".wai/projects/kaya/research/2026-11-18-note.md";

/// The canonical `events_by_source` keys: capture family and provenance, never
/// the actor name.
const SOURCES: [&str; 4] = ["wai", "git", "test-runner", "model"];

/// Write one human-authored research note into the wai tree.
fn add_wai_artifact(root: &Path) {
    let path = root.join(WAI_NOTE);
    fs::create_dir_all(path.parent().expect("the wai artifact has a parent"))
        .expect("create the wai research directory");
    fs::write(
        &path,
        "# facts fixture note\n\nA human note the wai adapter only observes.\n",
    )
    .expect("write the wai artifact");
}

/// A valid libtest JSONL report: suite and bare-string records are ignored, so
/// the terminal test events are what the adapter tallies.
const LIBTEST_REPORT: &str = concat!(
    "{\"type\":\"suite\",\"event\":\"started\",\"test_count\":4}\n",
    "{\"type\":\"test\",\"event\":\"started\",\"name\":\"passes_one\"}\n",
    "{\"type\":\"test\",\"event\":\"ok\",\"name\":\"passes_one\"}\n",
    "{\"type\":\"test\",\"event\":\"started\",\"name\":\"passes_two\"}\n",
    "{\"type\":\"test\",\"event\":\"ok\",\"name\":\"passes_two\"}\n",
    "{\"type\":\"test\",\"event\":\"started\",\"name\":\"fails\"}\n",
    "{\"type\":\"test\",\"event\":\"failed\",\"name\":\"fails\"}\n",
    "{\"type\":\"test\",\"event\":\"ignored\",\"name\":\"skipped\"}\n",
    "\"a bare JSON string is not a test event\"\n",
    "{\"type\":\"suite\",\"event\":\"ok\"}\n",
);

/// Where a fixture writes the run report it imports.
const LIBTEST_REPORT_PATH: &str = "cargo-test-results.jsonl";

/// Write the libtest report under `root` and return its path.
fn write_libtest_report(root: &Path) -> PathBuf {
    let path = root.join(LIBTEST_REPORT_PATH);
    fs::write(&path, LIBTEST_REPORT).expect("write the libtest report");
    path
}

/// A record with two capture sources: import the wai tree and `limit` recent
/// commits, then rebuild, publish, and commit the grown ledger, so `check`
/// reads a record whose committed history it can verify.
fn record_with_two_sources(root: &Path, limit: &str) {
    add_wai_artifact(root);
    run_ok(root, &["import", "wai"]);
    run_ok(root, &["import", "git", "--limit", limit]);
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
}

/// Every event the worktree ledger records, in file order.
fn ledger_events(root: &Path) -> Vec<Value> {
    let text = fs::read_to_string(root.join(LEDGER)).expect("the ledger is readable");
    text.lines()
        .map(|line| {
            serde_json::from_str(line)
                .unwrap_or_else(|error| panic!("a ledger line is one JSON event ({error}): {line}"))
        })
        .collect()
}

/// The greatest event timestamp the ledger records.
fn newest_timestamp(events: &[Value]) -> String {
    events
        .iter()
        .map(|event| {
            event["timestamp"]
                .as_str()
                .unwrap_or_else(|| panic!("every event carries a timestamp: {event}"))
        })
        .max()
        .expect("the ledger records events")
        .to_owned()
}

/// The seven record facts every executed `check` reports.
fn record_facts(envelope: &Value) -> &Value {
    const FIELDS: [&str; 7] = [
        "events",
        "ledger_digest",
        "last_capture_commit",
        "newest_event_at",
        "events_by_source",
        "emitted_kinds",
        "omitted_kinds",
    ];
    let data = envelope["data"]
        .as_object()
        .unwrap_or_else(|| panic!("check reports data as an object: {envelope}"));
    for field in FIELDS {
        assert!(
            data.contains_key(field),
            "check reports data.{field}: {envelope}"
        );
    }
    &envelope["data"]
}

/// The canonical source counts, requiring exactly the four source keys.
fn source_counts(facts: &Value) -> BTreeMap<String, u64> {
    let counts = facts["events_by_source"]
        .as_object()
        .unwrap_or_else(|| panic!("check reports data.events_by_source as an object: {facts}"));
    let keys = counts.keys().cloned().collect::<BTreeSet<_>>();
    let expected = SOURCES
        .iter()
        .map(|source| (*source).to_owned())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        keys, expected,
        "events_by_source holds exactly the canonical sources {SOURCES:?}: {facts}"
    );
    counts
        .iter()
        .map(|(source, count)| {
            let count = count
                .as_u64()
                .unwrap_or_else(|| panic!("the {source} source count is an integer: {facts}"));
            (source.clone(), count)
        })
        .collect()
}

/// A dotted `family.kind` list: strings, strictly sorted (hence unique), with a
/// non-empty family and kind on either side of a dot.
fn dotted_kinds(facts: &Value, field: &str) -> Vec<String> {
    let list = facts[field]
        .as_array()
        .unwrap_or_else(|| panic!("check reports data.{field} as an array: {facts}"));
    let names = list
        .iter()
        .map(|name| {
            name.as_str()
                .unwrap_or_else(|| panic!("data.{field} holds strings: {facts}"))
                .to_owned()
        })
        .collect::<Vec<_>>();
    for pair in names.windows(2) {
        assert!(
            pair[0] < pair[1],
            "data.{field} is sorted and deduplicated: {names:?}"
        );
    }
    for name in &names {
        let (family, kind) = name.split_once('.').unwrap_or_else(|| {
            panic!("data.{field} names dotted family.kind values, got \"{name}\": {facts}")
        });
        assert!(
            !family.is_empty() && !kind.is_empty() && !name.split('.').any(str::is_empty),
            "data.{field} names dotted family.kind values, got \"{name}\": {facts}"
        );
    }
    names
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
    let events = ledger_events(root);
    assert_eq!(
        facts["events"].as_u64(),
        Some(events.len() as u64),
        "the event count matches the ledger: {envelope}"
    );
    assert_eq!(
        facts["ledger_digest"],
        json!(sha256_hex(&worktree_bytes(root, LEDGER))),
        "the digest is sha256 over the verified ledger bytes: {envelope}"
    );
    assert_eq!(
        facts["newest_event_at"],
        json!(newest_timestamp(&events)),
        "the newest event timestamp is the greatest recorded: {envelope}"
    );

    // The seed records no capture observation, so there is no frontier yet,
    // and every event is a model event.
    assert_eq!(
        facts["last_capture_commit"],
        Value::Null,
        "a record without capture events reports a null frontier: {envelope}"
    );
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
fn check_facts_test_runner_source() {
    let fixture = Fixture::new();
    let root = fixture.root();
    let captured = head_sha(root);

    // A record holding three capture sources: a wai artifact, the one recent
    // commit, and a completed libtest run.
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
    let events = ledger_events(root);
    assert_eq!(
        facts["events"].as_u64(),
        Some(events.len() as u64),
        "the event count matches the ledger: {envelope}"
    );
    assert_eq!(
        facts["ledger_digest"],
        json!(sha256_hex(&worktree_bytes(root, LEDGER))),
        "the digest is sha256 over the verified ledger bytes: {envelope}"
    );
    assert_eq!(
        facts["newest_event_at"],
        json!(newest_timestamp(&events)),
        "the newest event timestamp is the greatest recorded: {envelope}"
    );

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
        events.len() as u64,
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

#[test]
fn check_facts_property() {
    let fixture = Fixture::new();
    let root = fixture.root();

    let notes = root.join("notes");
    fs::create_dir_all(&notes).expect("create the notes directory");

    // Two commits whose log order and author dates disagree: the tip carries
    // the earlier author date, so only a frontier ordered by record time names
    // the commit the record observed later.
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
    let expected_timestamp = newest_timestamp(&events);
    // The later author date wins, not the later log entry the import read last.
    let expected_frontier = later_dated;
    let expected_digest = sha256_hex(&worktree_bytes(root, LEDGER));

    // The same committed bytes read through the worktree and through the
    // index must yield one fact set: the facts describe the verified record,
    // not the read path.
    let worktree = fixture.output(&["check"]);
    let index = fixture.output(&["check", "--index"]);
    for output in [&worktree, &index] {
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
            json!(expected_frontier),
            "the frontier names the newest recorded capture sha: {envelope}"
        );
        assert_eq!(
            facts["newest_event_at"],
            json!(expected_timestamp),
            "the newest event timestamp is the greatest recorded: {envelope}"
        );
        assert_eq!(
            facts["ledger_digest"],
            json!(expected_digest),
            "the digest covers the verified ledger bytes: {envelope}"
        );
        let counts = source_counts(facts);
        assert_eq!(
            counts.values().sum::<u64>(),
            events.len() as u64,
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

    assert_eq!(
        parse_envelope(&worktree)["data"],
        parse_envelope(&index)["data"],
        "worktree and index modes report identical facts for identical bytes"
    );
}
