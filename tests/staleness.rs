//! Integration tests for authored staleness and the `kaya acknowledge` command.
//!
//! Every test builds a disposable repository: `kaya init`, the committed model
//! seed, an authored research index carrying one human block and one foreign
//! tool's managed block, `kaya build`, `kaya publish`, and one commit. A test
//! then cites ledger evidence from that authored block, appends a correction
//! or an ordinary observation directly to `.kaya/ledger.jsonl` (always
//! extending `HEAD` on a line boundary), and observes `build`, `check`, and
//! `acknowledge`.
//!
//! The contract under test has three faces:
//!
//! - A `<!-- kaya:cites … -->` marker inside an authored block declares the
//!   evidence the block rests on. Every id must exist in the ledger: an
//!   unknown id fails `build` with `kaya.projection.citation_marker` naming the
//!   document, the marker, and the id, and makes `check` exit 2 with the same
//!   label and no findings. A marker outside any authored block is ordinary
//!   prose: inert, and byte-preserved.
//! - A cited id ages its block only when a later event supersedes it and
//!   changes its effective data. A newly appended event that supersedes
//!   nothing never ages a block, and every changed cited id is named.
//! - `kaya acknowledge <document> --block <name> --events <id>… --actor <name>`
//!   appends exactly one `verification.acknowledged` event with a human actor.
//!   It clears staleness only for that document, block, and ids, records the
//!   correction each id was reviewed at, leaves authored bytes untouched, adds
//!   nothing to the research graph or the generated documents, refuses a
//!   non-human actor, an unknown document (including one outside the working
//!   directory), an unknown block, or an unknown event id without appending,
//!   accepts ids that exist but are no longer cited as inert, and repeats as a
//!   byte-identical no-op until a newer correction changes a reviewed id. It
//!   never rebuilds.

use std::collections::BTreeMap;
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use assert_cmd::Command as AssertCommand;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tempfile::TempDir;

const LEDGER: &str = ".kaya/ledger.jsonl";
const SEED: &str = ".kaya/seeds/add-core-research-record.jsonl";
const INDEX: &str = "docs/research/index.md";
const INDEX_BEGIN: &str = "<!-- kaya:begin generated:orientation.research-map -->";

/// The human-owned region of the research index.
const AUTHORED_BEGIN: &str = "<!-- kaya:authored -->";
const AUTHORED_END: &str = "<!-- kaya:end authored -->";

/// Authored prose the fixture seeds before `build` ever runs.
const AUTHORED_PROSE: &str = "Human interpretation of the compiled record.";

/// A foreign tool's managed block: bytes kaya never interprets or writes.
const FOREIGN_BLOCK: &str = "\
<!-- other-tool:begin -->
foreign
<!-- other-tool:end -->
";

/// The ordinal the fixture's single authored block receives.
const BLOCK: &str = "authored-1";

/// An id no ledger contains.
const UNKNOWN_EVENT: &str = "evt-000000000000000000000000";

/// A fresh, non-superseding observation appended to the fixture ledger.
const NEW_EVIDENCE: &str = "evt-feedfacefeedfacefeedface";

/// The correction event appended to the fixture ledger.
const CORRECTION: &str = "evt-feedfacefeedfacefeedfac1";

/// A second correction appended after the first is acknowledged.
const SECOND_CORRECTION: &str = "evt-feedfacefeedfacefeedfac2";

struct Fixture {
    dir: TempDir,
}

impl Fixture {
    /// A committed record: the model seed is committed before `build`, and the
    /// compiled research and publication trees are committed afterwards.
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

        compile_projection(root);
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

    fn root(&self) -> &Path {
        self.dir.path()
    }

    fn output(&self, args: &[&str]) -> Output {
        let mut command = AssertCommand::cargo_bin("kaya").expect("kaya binary is built");
        command.current_dir(self.root()).args(args);
        command.output().expect("kaya runs")
    }
}

/// A git repository with a fixed identity and no user configuration.
fn init_repository(root: &Path) {
    git(root, &["init", "--quiet"]);
    git(root, &["config", "user.name", "kaya staleness"]);
    git(root, &["config", "user.email", "staleness@example.invalid"]);
}

/// Create the ledger and import the committed model seed.
fn create_record(root: &Path) {
    run_ok(root, &["init"]);
    let seed = manifest_dir().join(SEED);
    let seed = seed.to_str().expect("the seed path is UTF-8");
    run_ok(root, &["import", "model", "--file", seed]);
}

/// Seed the authored index, then compile and publish the projections.
fn compile_projection(root: &Path) {
    seed_authored_index(root);
    run_ok(root, &["build"]);
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

/// Seed the research index with authored prose and a foreign managed block.
fn seed_authored_index(root: &Path) {
    let path = root.join(INDEX);
    fs::create_dir_all(path.parent().expect("the index has a parent"))
        .expect("create the research directory");
    let mut document = String::from("# Research index\n\n");
    document.push_str(FOREIGN_BLOCK);
    document.push('\n');
    document.push_str(AUTHORED_BEGIN);
    document.push('\n');
    document.push_str(AUTHORED_PROSE);
    document.push('\n');
    document.push_str(AUTHORED_END);
    document.push('\n');
    fs::write(&path, document).expect("seed the authored research index");
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

/// A git command with a deterministic identity and no user configuration.
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
        .env("GIT_AUTHOR_NAME", "kaya staleness")
        .env("GIT_AUTHOR_EMAIL", "staleness@example.invalid")
        .env("GIT_COMMITTER_NAME", "kaya staleness")
        .env("GIT_COMMITTER_EMAIL", "staleness@example.invalid")
        .env("GIT_AUTHOR_DATE", "2026-11-18T00:00:00+00:00")
        .env("GIT_COMMITTER_DATE", "2026-11-18T00:00:00+00:00");
    command
}

/// Run one git command, requiring success.
fn git(workspace: &Path, args: &[&str]) {
    let output = git_command(workspace)
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

/// The `data.stale` entries a build or check reports.
fn stale_entries(envelope: &Value) -> Vec<Value> {
    envelope["data"]["stale"]
        .as_array()
        .unwrap_or_else(|| panic!("the envelope reports data.stale as an array: {envelope}"))
        .clone()
}

/// The `data.drift` entries a check reports.
fn drift_entries(envelope: &Value) -> Vec<Value> {
    envelope["data"]["drift"]
        .as_array()
        .unwrap_or_else(|| panic!("check reports data.drift as an array: {envelope}"))
        .clone()
}

/// Assert a failure envelope and return its `(label, message)`.
fn assert_error(envelope: &Value, label_prefix: &str) -> (String, String) {
    assert_eq!(
        envelope["ok"],
        json!(false),
        "a refusal is an error envelope: {envelope}"
    );
    let error = envelope
        .get("error")
        .filter(|error| !error.is_null())
        .unwrap_or_else(|| panic!("a refusal carries an error: {envelope}"));
    let label = error["label"]
        .as_str()
        .unwrap_or_else(|| panic!("a refusal names its label: {envelope}"))
        .to_owned();
    assert!(
        label.starts_with(label_prefix),
        "the refusal label {label:?} belongs to the {label_prefix} namespace: {envelope}"
    );
    let message = error["message"]
        .as_str()
        .unwrap_or_else(|| panic!("a refusal names its cause: {envelope}"))
        .to_owned();
    (label, message)
}

fn read_document(root: &Path) -> String {
    fs::read_to_string(root.join(INDEX)).expect("the research index is readable")
}

/// Cite `ids` from the fixture's authored block with one marker line.
fn cite(root: &Path, ids: &[String]) {
    assert!(!ids.is_empty(), "a citation marker names at least one id");
    let marker = format!("<!-- kaya:cites {} -->", ids.join(" "));
    let document = read_document(root);
    assert!(
        !document.contains(&marker),
        "the authored block does not carry {marker} yet: {document}"
    );
    let cited = format!("{AUTHORED_PROSE}\n{marker}");
    let updated = document.replacen(AUTHORED_PROSE, &cited, 1);
    assert_ne!(updated, document, "the authored prose carries the marker");
    fs::write(root.join(INDEX), updated).expect("cite the evidence from authored prose");
}

/// Put `marker` in the document header, outside every authored block.
fn insert_uncategorized_marker(root: &Path, marker: &str) {
    let document = read_document(root);
    let header = "# Research index\n";
    let updated = document.replacen(header, &format!("{header}\n{marker}\n"), 1);
    assert_ne!(updated, document, "the document header carries the marker");

    let authored = updated
        .find(AUTHORED_BEGIN)
        .unwrap_or_else(|| panic!("the index carries an authored block: {updated}"));
    let placed = updated
        .find(marker)
        .unwrap_or_else(|| panic!("the marker survives insertion: {updated}"));
    assert!(
        placed < authored,
        "the marker sits outside the authored block: {updated}"
    );
    fs::write(root.join(INDEX), updated).expect("write the uncategorized marker");
}

/// The authored bytes of the research index, markers included.
fn authored_bytes(root: &Path) -> Vec<u8> {
    let document = read_document(root);
    let start = document
        .find(AUTHORED_BEGIN)
        .unwrap_or_else(|| panic!("the index carries an authored block: {document}"));
    let rest = &document[start..];
    let end = rest
        .find(AUTHORED_END)
        .unwrap_or_else(|| panic!("the authored block is closed: {document}"))
        + AUTHORED_END.len();
    rest.as_bytes()[..end].to_vec()
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

/// The last event the worktree ledger records.
fn last_ledger_event(root: &Path) -> Value {
    ledger_events(root)
        .pop()
        .expect("the ledger records at least one event")
}

/// The exact bytes of the worktree ledger.
fn ledger_bytes(root: &Path) -> Vec<u8> {
    fs::read(root.join(LEDGER)).expect("the ledger is readable")
}

/// Append one event as a newline-terminated ledger line.
fn append_event(root: &Path, event: &Value) {
    let mut line = serde_json::to_string(event).expect("the event serializes to one JSON line");
    line.push('\n');
    let mut ledger = fs::OpenOptions::new()
        .append(true)
        .open(root.join(LEDGER))
        .expect("the ledger is appended to, never rewritten");
    ledger
        .write_all(line.as_bytes())
        .expect("append the event line");
}

/// A valid observation that supersedes nothing: a newly created Finding.
fn new_evidence_event() -> Value {
    json!({
        "id": NEW_EVIDENCE,
        "schema": "kaya.event/v0",
        "type": "model.entity.created",
        "timestamp": "2026-11-20T00:00:00Z",
        "actor": { "type": "human", "name": "charly" },
        "subject": { "type": "Finding", "id": "kaya/Finding/F-staleness-900" },
        "data": { "attributes": { "title": "newly captured evidence" } },
        "provenance": {
            "tool": "kaya",
            "version": "0.1.0",
            "reference": "kaya/Finding/F-staleness-900",
        },
        "artifacts": [],
    })
}

/// Append one correction that changes the effective data of every id in
/// `targets`, cloning the first target's envelope and taking a later
/// timestamp and a fresh id.
fn append_correction(root: &Path, targets: &[String]) {
    append_correction_event(root, CORRECTION, targets);
}

/// Append a further correction with its own id, after `append_correction` has
/// already claimed `CORRECTION`.
fn append_second_correction(root: &Path, targets: &[String]) {
    append_correction_event(root, SECOND_CORRECTION, targets);
}

fn append_correction_event(root: &Path, id: &str, targets: &[String]) {
    assert!(!targets.is_empty(), "a correction supersedes something");
    let events = ledger_events(root);
    assert!(
        !events.iter().any(|event| event["id"] == json!(id)),
        "the correction id is fresh: {id}"
    );
    let target = events
        .iter()
        .find(|event| targets.iter().any(|target| event["id"] == json!(target)))
        .unwrap_or_else(|| panic!("the cited id is in the ledger: {targets:?}"))
        .clone();

    let mut correction = target;
    correction["id"] = json!(id);
    correction["timestamp"] = json!("2026-11-20T00:00:00Z");
    correction["data"] = json!({
        "supersedes": targets,
        "attributes": { "title": "corrected by the staleness fixture" },
    });
    append_event(root, &correction);
}

/// One deterministic commit.
fn commit(root: &Path, message: &str) {
    git(root, &["add", "--all"]);
    git(root, &["commit", "--quiet", "--message", message]);
}

/// Every file digest under the research and publication projection trees.
fn snapshot_documents(root: &Path) -> BTreeMap<PathBuf, String> {
    let mut snapshot = BTreeMap::new();
    for tree in ["docs/research", "docs/public"] {
        collect_documents(root, &root.join(tree), &mut snapshot);
    }
    snapshot
}

fn collect_documents(root: &Path, dir: &Path, snapshot: &mut BTreeMap<PathBuf, String>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|err| panic!("read {}: {err}", dir.display()));
    for entry in entries {
        let entry = entry.unwrap_or_else(|err| panic!("read an entry of {}: {err}", dir.display()));
        let path = entry.path();
        let file_type = entry.file_type().expect("tree entry type");
        if file_type.is_dir() {
            collect_documents(root, &path, snapshot);
        } else if file_type.is_file() {
            let bytes =
                fs::read(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
            let relative = path
                .strip_prefix(root)
                .expect("path below the fixture root")
                .to_path_buf();
            snapshot.insert(relative, format!("{:x}", Sha256::digest(&bytes)));
        }
    }
}

/// Cite the first seed event from the authored block, rebuild, publish, and
/// commit the cited state.
fn record_citing_first_event(fixture: &Fixture) -> String {
    let root = fixture.root();
    let cited = ledger_events(root)[0]["id"]
        .as_str()
        .expect("the first seed event carries an id")
        .to_owned();
    cite(root, std::slice::from_ref(&cited));
    run_ok(root, &["build"]);
    run_ok(root, &["publish"]);
    commit(root, "cite the first seed event from authored prose");
    cited
}

#[test]
fn citation_known_ids_accepted() {
    let fixture = Fixture::new();
    let root = fixture.root();
    let cited = ledger_events(root)[0]["id"]
        .as_str()
        .expect("the first seed event carries an id")
        .to_owned();

    cite(root, std::slice::from_ref(&cited));
    let authored = authored_bytes(root);

    let built = run_ok(root, &["build"]);
    assert_eq!(
        built["data"]["stale"],
        json!([]),
        "a citation of an admitted event is accepted and stale-free: {built}"
    );
    assert_eq!(
        authored_bytes(root),
        authored,
        "build leaves the authored bytes alone"
    );

    run_ok(root, &["publish"]);
    assert_eq!(
        authored_bytes(root),
        authored,
        "publish leaves the authored bytes alone"
    );

    let output = fixture.output(&["check"]);
    assert_eq!(
        exit_code(&output),
        0,
        "a known citation is not a check failure: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    assert_eq!(envelope["ok"], json!(true));
    assert_eq!(envelope["envelope_kind"], json!("check"));
    assert!(
        drift_entries(&envelope).is_empty(),
        "the committed record matches its projections: {envelope}"
    );
    assert!(
        stale_entries(&envelope).is_empty(),
        "nothing superseded the cited id: {envelope}"
    );
    assert!(
        read_document(root).contains(&format!("<!-- kaya:cites {cited} -->")),
        "the citation marker is preserved"
    );
}

#[test]
fn citation_unknown_id_refused() {
    let fixture = Fixture::new();
    let root = fixture.root();

    cite(root, &[UNKNOWN_EVENT.to_owned()]);
    let authored = authored_bytes(root);
    let documents = snapshot_documents(root);

    let output = fixture.output(&["build"]);
    assert_eq!(
        exit_code(&output),
        1,
        "build refuses an unknown citation: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    let (label, message) = assert_error(&envelope, "kaya.projection.citation_marker");
    assert_eq!(
        label, "kaya.projection.citation_marker",
        "the citation refusal carries its exact label: {envelope}"
    );
    assert!(
        message.contains(INDEX),
        "the refusal names the document: {message}"
    );
    assert!(
        message.contains("kaya:cites"),
        "the refusal names the marker: {message}"
    );
    assert!(
        message.contains(UNKNOWN_EVENT),
        "the refusal names the unknown id: {message}"
    );
    assert_eq!(
        authored_bytes(root),
        authored,
        "a refused build never rewrites authored bytes"
    );
    assert_eq!(
        snapshot_documents(root),
        documents,
        "a refused build leaves the compiled documents as they were"
    );

    let documents = snapshot_documents(root);
    let output = fixture.output(&["check"]);
    assert_eq!(
        exit_code(&output),
        2,
        "an unknown citation is an I/O-class check refusal: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    let (label, message) = assert_error(&envelope, "kaya.projection.citation_marker");
    assert_eq!(
        label, "kaya.projection.citation_marker",
        "check uses the same citation label as build: {envelope}"
    );
    assert!(
        message.contains(INDEX),
        "the refusal names the document: {message}"
    );
    assert!(
        message.contains(UNKNOWN_EVENT),
        "the refusal names the unknown id: {message}"
    );
    assert!(
        envelope["data"].is_null(),
        "a refused check reports no drift: {envelope}"
    );
    assert_eq!(
        snapshot_documents(root),
        documents,
        "a refused check writes nothing"
    );
}

#[test]
fn citation_marker_outside_authored_block_inert() {
    let fixture = Fixture::new();
    let root = fixture.root();
    let marker = format!("<!-- kaya:cites {UNKNOWN_EVENT} -->");

    insert_uncategorized_marker(root, &marker);
    let authored = authored_bytes(root);
    let document = read_document(root);

    let built = run_ok(root, &["build"]);
    assert_eq!(
        built["data"]["stale"],
        json!([]),
        "a marker outside an authored block has no staleness effect: {built}"
    );
    assert_eq!(
        read_document(root),
        document,
        "build leaves the document byte-identical, marker included"
    );
    assert_eq!(
        document.matches(&marker).count(),
        1,
        "the uncategorized marker survives build byte-identically: {document}"
    );
    assert_eq!(
        authored_bytes(root),
        authored,
        "build leaves the authored block alone"
    );

    run_ok(root, &["publish"]);
    assert_eq!(
        read_document(root),
        document,
        "publish leaves the document byte-identical, marker included"
    );

    let output = fixture.output(&["check"]);
    assert_eq!(
        exit_code(&output),
        0,
        "an inert marker is not a check failure: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    assert!(
        drift_entries(&envelope).is_empty(),
        "the uncategorized marker is not managed content: {envelope}"
    );
    assert!(
        stale_entries(&envelope).is_empty(),
        "the uncategorized marker cites nothing: {envelope}"
    );
    assert_eq!(
        read_document(root),
        document,
        "check leaves the document byte-identical, marker included"
    );
}

#[test]
fn new_evidence_does_not_age_a_block() {
    let fixture = Fixture::new();
    let cited = record_citing_first_event(&fixture);
    let root = fixture.root();
    let authored = authored_bytes(root);

    append_event(root, &new_evidence_event());

    let built = run_ok(root, &["build"]);
    assert_eq!(
        built["data"]["stale"],
        json!([]),
        "an appended observation that supersedes nothing ages no block: {built}"
    );
    assert_eq!(
        authored_bytes(root),
        authored,
        "reporting no staleness never rewrites authored bytes"
    );
    assert!(
        ledger_events(root)
            .iter()
            .any(|event| event["id"] == json!(cited)),
        "the cited id is still in the ledger"
    );

    run_ok(root, &["publish"]);
    commit(root, "record the newly captured evidence");

    let output = fixture.output(&["check"]);
    assert_eq!(
        exit_code(&output),
        0,
        "uncorrected evidence ages no block: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    assert!(
        drift_entries(&envelope).is_empty(),
        "the rebuilt record matches its projections: {envelope}"
    );
    assert!(
        stale_entries(&envelope).is_empty(),
        "new evidence alone is not staleness: {envelope}"
    );
}

#[test]
fn correction_ages_a_block() {
    let fixture = Fixture::new();
    let cited = record_citing_first_event(&fixture);
    let root = fixture.root();
    let authored = authored_bytes(root);

    append_correction(root, std::slice::from_ref(&cited));

    let built = run_ok(root, &["build"]);
    assert_eq!(
        built["data"]["stale"],
        json!([{ "block": BLOCK, "evidence": [cited] }]),
        "a correction that changes the cited data ages the block: {built}"
    );
    assert_eq!(
        authored_bytes(root),
        authored,
        "the stale report never rewrites authored bytes"
    );

    run_ok(root, &["publish"]);

    let output = fixture.output(&["check"]);
    assert_eq!(
        exit_code(&output),
        0,
        "a stale block is advisory without the flag: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    assert_eq!(envelope["ok"], json!(true));
    assert!(
        drift_entries(&envelope).is_empty(),
        "the worktree matches the corrected projections: {envelope}"
    );
    assert_eq!(
        stale_entries(&envelope),
        vec![json!({
            "path": INDEX,
            "block": BLOCK,
            "evidence": [cited.clone()],
        })],
        "check names the document, block, and changed id: {envelope}"
    );

    let output = fixture.output(&["check", "--fail-on-stale-authored"]);
    assert_eq!(
        exit_code(&output),
        1,
        "the stale block is a violation under the flag: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    assert_eq!(
        stale_entries(&envelope),
        vec![json!({
            "path": INDEX,
            "block": BLOCK,
            "evidence": [cited.clone()],
        })],
        "enforcement names the same evidence: {envelope}"
    );
    assert_eq!(
        authored_bytes(root),
        authored,
        "enforcement never rewrites authored bytes"
    );
}

#[test]
fn acknowledgment_clears_covered_ids() {
    let fixture = Fixture::new();
    let cited = record_citing_first_event(&fixture);
    let root = fixture.root();
    let authored = authored_bytes(root);

    append_correction(root, std::slice::from_ref(&cited));
    let built = run_ok(root, &["build"]);
    assert_eq!(
        built["data"]["stale"],
        json!([{ "block": BLOCK, "evidence": [cited] }]),
        "the correction ages the block before acknowledgment: {built}"
    );

    let acknowledged = run_ok(
        root,
        &[
            "acknowledge",
            INDEX,
            "--block",
            BLOCK,
            "--events",
            cited.as_str(),
            "--actor",
            "charly",
        ],
    );
    assert_eq!(
        acknowledged["envelope_kind"],
        json!("acknowledge"),
        "the command emits its own envelope kind: {acknowledged}"
    );
    assert_eq!(
        acknowledged["data"]["appended"],
        json!(true),
        "the first invocation appends one event: {acknowledged}"
    );
    let event = &acknowledged["data"]["event"];
    assert_eq!(
        event["type"],
        json!("verification.acknowledged"),
        "the appended event carries the acknowledgment type: {acknowledged}"
    );
    assert_eq!(
        event["subject"],
        json!({ "type": "Document", "id": INDEX }),
        "the subject names the document as the caller named it: {acknowledged}"
    );
    assert_eq!(
        event["data"]["block"],
        json!(BLOCK),
        "the event names the authored block: {acknowledged}"
    );
    assert_eq!(
        event["data"]["events"],
        json!([cited]),
        "the event names exactly the covered ids: {acknowledged}"
    );
    assert_eq!(
        event["actor"],
        json!({ "type": "human", "name": "charly" }),
        "the acknowledgment carries a human actor: {acknowledged}"
    );

    let last = last_ledger_event(root);
    assert_eq!(
        last["type"],
        json!("verification.acknowledged"),
        "the acknowledgment is the last ledger line: {last}"
    );
    assert_eq!(
        last["data"]["events"],
        json!([cited]),
        "the stored event names the covered id: {last}"
    );
    let ledger = ledger_bytes(root);

    let built = run_ok(root, &["build"]);
    assert_eq!(
        built["data"]["stale"],
        json!([]),
        "acknowledging every changed cited id clears the block: {built}"
    );
    assert_eq!(
        authored_bytes(root),
        authored,
        "acknowledgment reporting never rewrites authored bytes"
    );

    run_ok(root, &["publish"]);
    assert_eq!(
        authored_bytes(root),
        authored,
        "publish leaves the acknowledged authored bytes alone"
    );

    let output = fixture.output(&["check"]);
    assert_eq!(
        exit_code(&output),
        0,
        "an acknowledged block is not a violation: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    assert!(
        stale_entries(&envelope).is_empty(),
        "the acknowledgment covers every changed cited id: {envelope}"
    );
    assert!(
        drift_entries(&envelope).is_empty(),
        "the rebuilt record matches its projections: {envelope}"
    );
    assert_eq!(
        ledger_bytes(root),
        ledger,
        "build and check never append to the ledger"
    );
}

#[test]
fn partial_acknowledgment_leaves_block_stale() {
    let fixture = Fixture::new();
    let root = fixture.root();

    let mut ids: Vec<String> = ledger_events(root)
        .iter()
        .take(2)
        .map(|event| {
            event["id"]
                .as_str()
                .expect("the seed event carries an id")
                .to_owned()
        })
        .collect();
    ids.sort();
    cite(root, &ids);
    run_ok(root, &["build"]);
    run_ok(root, &["publish"]);
    commit(root, "cite two seed events from authored prose");

    append_correction(root, &ids);
    let built = run_ok(root, &["build"]);
    assert_eq!(
        built["data"]["stale"],
        json!([{ "block": BLOCK, "evidence": ids }]),
        "one correction ages the block on both cited ids: {built}"
    );
    run_ok(root, &["publish"]);

    let (first, second) = (&ids[0], &ids[1]);
    run_ok(
        root,
        &[
            "acknowledge",
            INDEX,
            "--block",
            BLOCK,
            "--events",
            first.as_str(),
            "--actor",
            "charly",
        ],
    );

    let authored = authored_bytes(root);
    let built = run_ok(root, &["build"]);
    assert_eq!(
        built["data"]["stale"],
        json!([{ "block": BLOCK, "evidence": [second] }]),
        "the unacknowledged changed id keeps the block stale: {built}"
    );
    assert_eq!(
        authored_bytes(root),
        authored,
        "a partial acknowledgment never rewrites authored bytes"
    );

    let output = fixture.output(&["check", "--fail-on-stale-authored"]);
    assert_eq!(
        exit_code(&output),
        1,
        "the partially acknowledged block is still a violation: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    assert_eq!(
        stale_entries(&envelope),
        vec![json!({
            "path": INDEX,
            "block": BLOCK,
            "evidence": [second.clone()],
        })],
        "the report names only the unacknowledged id: {envelope}"
    );
    assert_eq!(
        authored_bytes(root),
        authored,
        "the stale report leaves authored bytes alone"
    );
}

#[test]
fn acknowledgment_is_not_graph_evidence() {
    let fixture = Fixture::new();
    let cited = record_citing_first_event(&fixture);
    let root = fixture.root();

    append_correction(root, std::slice::from_ref(&cited));
    let built_before = run_ok(root, &["build"]);
    run_ok(root, &["publish"]);
    let documents_before = snapshot_documents(root);

    run_ok(
        root,
        &[
            "acknowledge",
            INDEX,
            "--block",
            BLOCK,
            "--events",
            cited.as_str(),
            "--actor",
            "charly",
        ],
    );

    let built_after = run_ok(root, &["build"]);
    run_ok(root, &["publish"]);
    let documents_after = snapshot_documents(root);

    assert_eq!(
        documents_after, documents_before,
        "replaying an acknowledgment regenerates byte-identical documents"
    );
    assert_eq!(
        built_after["data"]["sources"], built_before["data"]["sources"],
        "the acknowledgment adds no graph node or edge to any generated block"
    );
    assert_eq!(
        built_after["data"]["omitted"], built_before["data"]["omitted"],
        "the acknowledgment suppresses no generated content"
    );
    assert_eq!(
        built_after["data"]["stale"],
        json!([]),
        "the acknowledgment still clears the stale report: {built_after}"
    );
}

#[test]
fn acknowledge_tool_actor_refused() {
    let fixture = Fixture::new();
    let cited = record_citing_first_event(&fixture);
    let root = fixture.root();

    append_correction(root, std::slice::from_ref(&cited));
    let built = run_ok(root, &["build"]);
    assert_eq!(
        built["data"]["stale"],
        json!([{ "block": BLOCK, "evidence": [cited] }]),
        "the correction ages the block before the refused acknowledgment: {built}"
    );
    let ledger = ledger_bytes(root);

    for actor_type in ["tool", "agent"] {
        let output = fixture.output(&[
            "acknowledge",
            INDEX,
            "--block",
            BLOCK,
            "--events",
            cited.as_str(),
            "--actor",
            "some-bot",
            "--actor-type",
            actor_type,
        ]);
        assert_eq!(
            exit_code(&output),
            1,
            "a {actor_type} actor cannot acknowledge: {}",
            stderr(&output)
        );
        let envelope = parse_envelope(&output);
        let (_, message) = assert_error(&envelope, "kaya.acknowledge.");
        assert!(
            message.contains(actor_type),
            "the refusal names the actor type: {message}"
        );
        assert_eq!(
            ledger_bytes(root),
            ledger,
            "a refused {actor_type} acknowledgment appends nothing"
        );
    }

    let built = run_ok(root, &["build"]);
    assert_eq!(
        built["data"]["stale"],
        json!([{ "block": BLOCK, "evidence": [cited] }]),
        "the refused acknowledgments cleared nothing: {built}"
    );
}

#[test]
fn acknowledge_unknown_block_refused() {
    let fixture = Fixture::new();
    let cited = record_citing_first_event(&fixture);
    let root = fixture.root();

    append_correction(root, std::slice::from_ref(&cited));
    run_ok(root, &["build"]);
    let ledger = ledger_bytes(root);

    let output = fixture.output(&[
        "acknowledge",
        INDEX,
        "--block",
        "authored-9",
        "--events",
        cited.as_str(),
        "--actor",
        "charly",
    ]);
    assert_eq!(
        exit_code(&output),
        1,
        "an unknown block is refused: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    let (_, message) = assert_error(&envelope, "kaya.acknowledge.");
    assert!(
        message.contains("authored-9"),
        "the refusal names the unknown block: {message}"
    );
    assert_eq!(
        ledger_bytes(root),
        ledger,
        "an unknown block appends nothing"
    );

    let missing_document = "docs/research/absent.md";
    let output = fixture.output(&[
        "acknowledge",
        missing_document,
        "--block",
        BLOCK,
        "--events",
        cited.as_str(),
        "--actor",
        "charly",
    ]);
    assert_eq!(
        exit_code(&output),
        1,
        "an unknown document is refused: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    let (_, message) = assert_error(&envelope, "kaya.acknowledge.");
    assert!(
        message.contains(missing_document),
        "the refusal names the unknown document: {message}"
    );
    assert_eq!(
        ledger_bytes(root),
        ledger,
        "an unknown document appends nothing"
    );

    let built = run_ok(root, &["build"]);
    assert_eq!(
        built["data"]["stale"],
        json!([{ "block": BLOCK, "evidence": [cited] }]),
        "the refused acknowledgments cleared nothing: {built}"
    );
}

#[test]
fn acknowledge_unknown_event_refused() {
    let fixture = Fixture::new();
    let cited = record_citing_first_event(&fixture);
    let root = fixture.root();

    append_correction(root, std::slice::from_ref(&cited));
    run_ok(root, &["build"]);
    let ledger = ledger_bytes(root);

    let output = fixture.output(&[
        "acknowledge",
        INDEX,
        "--block",
        BLOCK,
        "--events",
        UNKNOWN_EVENT,
        "--actor",
        "charly",
    ]);
    assert_eq!(
        exit_code(&output),
        1,
        "an unknown event id is refused: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    let (_, message) = assert_error(&envelope, "kaya.acknowledge.");
    assert!(
        message.contains(UNKNOWN_EVENT),
        "the refusal names the unknown id: {message}"
    );
    assert_eq!(
        ledger_bytes(root),
        ledger,
        "an unknown event id appends nothing"
    );

    let built = run_ok(root, &["build"]);
    assert_eq!(
        built["data"]["stale"],
        json!([{ "block": BLOCK, "evidence": [cited] }]),
        "the refused acknowledgment cleared nothing: {built}"
    );
}

#[test]
fn acknowledge_is_idempotent() {
    let fixture = Fixture::new();
    let cited = record_citing_first_event(&fixture);
    let root = fixture.root();

    append_correction(root, std::slice::from_ref(&cited));
    run_ok(root, &["build"]);

    let args = [
        "acknowledge",
        INDEX,
        "--block",
        BLOCK,
        "--events",
        cited.as_str(),
        "--actor",
        "charly",
    ];
    let first = run_ok(root, &args);
    assert_eq!(
        first["data"]["appended"],
        json!(true),
        "the first invocation appends: {first}"
    );
    let ledger = ledger_bytes(root);

    let second = run_ok(root, &args);
    assert_eq!(
        second["envelope_kind"],
        json!("acknowledge"),
        "a repeated invocation still emits the acknowledge envelope: {second}"
    );
    assert_eq!(
        second["data"]["appended"],
        json!(false),
        "the same acknowledgment twice appends nothing the second time: {second}"
    );
    assert_eq!(
        ledger_bytes(root),
        ledger,
        "a repeated acknowledgment leaves the ledger bytes identical"
    );

    let acknowledgments = ledger_events(root)
        .iter()
        .filter(|event| event["type"] == json!("verification.acknowledged"))
        .count();
    assert_eq!(
        acknowledgments, 1,
        "the ledger records exactly one acknowledgment event"
    );

    let built = run_ok(root, &["build"]);
    assert_eq!(
        built["data"]["stale"],
        json!([]),
        "the single recorded acknowledgment clears the block: {built}"
    );
}

#[test]
fn acknowledge_uncited_event_inert() {
    let fixture = Fixture::new();
    let root = fixture.root();

    let events = ledger_events(root);
    let cited = events[0]["id"]
        .as_str()
        .expect("the first seed event carries an id")
        .to_owned();
    let uncited = events[1]["id"]
        .as_str()
        .expect("the second seed event carries an id")
        .to_owned();
    assert_ne!(cited, uncited, "the fixture records two distinct events");

    cite(root, std::slice::from_ref(&cited));
    run_ok(root, &["build"]);
    run_ok(root, &["publish"]);
    commit(root, "cite the first seed event");

    append_correction(root, std::slice::from_ref(&cited));
    let built = run_ok(root, &["build"]);
    assert_eq!(
        built["data"]["stale"],
        json!([{ "block": BLOCK, "evidence": [cited] }]),
        "the correction ages the block: {built}"
    );

    let acknowledged = run_ok(
        root,
        &[
            "acknowledge",
            INDEX,
            "--block",
            BLOCK,
            "--events",
            uncited.as_str(),
            "--actor",
            "charly",
        ],
    );
    assert_eq!(
        acknowledged["data"]["appended"],
        json!(true),
        "an existing but uncited id is accepted: {acknowledged}"
    );

    let authored = authored_bytes(root);
    let built = run_ok(root, &["build"]);
    assert_eq!(
        built["data"]["stale"],
        json!([{ "block": BLOCK, "evidence": [cited] }]),
        "acknowledging an uncited id has no effect on staleness: {built}"
    );
    assert_eq!(
        authored_bytes(root),
        authored,
        "the inert acknowledgment never rewrites authored bytes"
    );

    run_ok(
        root,
        &[
            "acknowledge",
            INDEX,
            "--block",
            BLOCK,
            "--events",
            cited.as_str(),
            "--actor",
            "charly",
        ],
    );
    let built = run_ok(root, &["build"]);
    assert_eq!(
        built["data"]["stale"],
        json!([]),
        "the changed cited id is cleared once it is acknowledged: {built}"
    );
    assert_eq!(
        authored_bytes(root),
        authored,
        "the authored bytes never change"
    );
}

#[test]
fn acknowledgment_after_a_new_correction_is_stale() {
    let fixture = Fixture::new();
    let cited = record_citing_first_event(&fixture);
    let root = fixture.root();

    append_correction(root, std::slice::from_ref(&cited));
    let built = run_ok(root, &["build"]);
    assert_eq!(
        built["data"]["stale"],
        json!([{ "block": BLOCK, "evidence": [cited] }]),
        "the first correction ages the block: {built}"
    );

    let args = [
        "acknowledge",
        INDEX,
        "--block",
        BLOCK,
        "--events",
        cited.as_str(),
        "--actor",
        "charly",
    ];
    let first = run_ok(root, &args);
    assert_eq!(
        first["data"]["appended"],
        json!(true),
        "the first acknowledgment appends: {first}"
    );
    let reviewed: serde_json::Map<String, Value> =
        [(cited.clone(), json!(CORRECTION))].into_iter().collect();
    assert_eq!(
        first["data"]["event"]["data"]["reviewed"],
        Value::Object(reviewed),
        "the acknowledgment names the correction it reviewed: {first}"
    );

    let built = run_ok(root, &["build"]);
    assert_eq!(
        built["data"]["stale"],
        json!([]),
        "acknowledging every changed cited id clears the block: {built}"
    );

    append_second_correction(root, std::slice::from_ref(&cited));
    let built = run_ok(root, &["build"]);
    assert_eq!(
        built["data"]["stale"],
        json!([{ "block": BLOCK, "evidence": [cited] }]),
        "a newer correction re-raises the acknowledged block: {built}"
    );

    run_ok(root, &["publish"]);
    let output = fixture.output(&["check"]);
    assert_eq!(
        exit_code(&output),
        0,
        "a re-raised block is advisory without the flag: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    assert_eq!(
        stale_entries(&envelope),
        vec![json!({
            "path": INDEX,
            "block": BLOCK,
            "evidence": [cited.clone()],
        })],
        "check names the id changed since its last acknowledgment: {envelope}"
    );

    let second = run_ok(root, &args);
    assert_eq!(
        second["data"]["appended"],
        json!(true),
        "a newer correction makes the same acknowledgment a new review: {second}"
    );
    let reviewed: serde_json::Map<String, Value> = [(cited.clone(), json!(SECOND_CORRECTION))]
        .into_iter()
        .collect();
    assert_eq!(
        second["data"]["event"]["data"]["reviewed"],
        Value::Object(reviewed),
        "the second acknowledgment names the newer correction: {second}"
    );
    let acknowledgments = ledger_events(root)
        .iter()
        .filter(|event| event["type"] == json!("verification.acknowledged"))
        .count();
    assert_eq!(
        acknowledgments, 2,
        "the ledger records a second, version-aware acknowledgment"
    );

    let built = run_ok(root, &["build"]);
    assert_eq!(
        built["data"]["stale"],
        json!([]),
        "the new acknowledgment clears the block again: {built}"
    );
}

#[test]
fn acknowledgment_with_an_absolute_path_clears_staleness() {
    let fixture = Fixture::new();
    let cited = record_citing_first_event(&fixture);
    let root = fixture.root();

    append_correction(root, std::slice::from_ref(&cited));
    let built = run_ok(root, &["build"]);
    assert_eq!(
        built["data"]["stale"],
        json!([{ "block": BLOCK, "evidence": [cited] }]),
        "the correction ages the block before acknowledgment: {built}"
    );

    let absolute = root.join(INDEX);
    let acknowledged = run_ok(
        root,
        &[
            "acknowledge",
            absolute.to_str().expect("the fixture path is UTF-8"),
            "--block",
            BLOCK,
            "--events",
            cited.as_str(),
            "--actor",
            "charly",
        ],
    );
    assert_eq!(
        acknowledged["data"]["appended"],
        json!(true),
        "an absolute path acknowledges the document: {acknowledged}"
    );
    assert_eq!(
        acknowledged["data"]["event"]["subject"],
        json!({ "type": "Document", "id": INDEX }),
        "the event records the build's canonical label: {acknowledged}"
    );

    let built = run_ok(root, &["build"]);
    assert_eq!(
        built["data"]["stale"],
        json!([]),
        "the canonical label covers the block build reports: {built}"
    );
}
