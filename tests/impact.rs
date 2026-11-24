//! Integration tests for the documentation impact gate: change-set mapping
//! over document-IR dependency edges, closed dispositions, mechanically
//! derived dispositions, human-only review events, stale authored blocks as
//! unmet obligations, and the deterministic report and exit contract
//! (`kaya impact`), plus the human disposition recorder (`kaya disposition`).
//!
//! Every test drives the real binary in a disposable git repository seeded
//! with the committed model record, an authored research index citing seed
//! evidence, and one declared doctest, then stages a change set, appends or
//! records ledger state, and reads the one JSON envelope each invocation
//! prints.

use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use assert_cmd::Command as AssertCommand;
use serde_json::{Value, json};
use tempfile::TempDir;

use kaya::impact::{ChangeSet, Policy, Reason, map_impacts};
use kaya::projection::ir::{DocumentIr, Record, RecordKind};

const LEDGER: &str = ".kaya/ledger.jsonl";
const SEED: &str = ".kaya/seeds/add-core-research-record.jsonl";
const INDEX: &str = "docs/research/index.md";
const DOC: &str = "docs/research/index.md";
const RULES: &str = "impact-rules.toml";
const GENERATED_BEGIN: &str = "<!-- kaya:begin generated:orientation.research-map -->";
const AUTHORED_BEGIN: &str = "<!-- kaya:authored -->";
const AUTHORED_END: &str = "<!-- kaya:end authored -->";
const PROSE: &str = "Human interpretation of the compiled record.";

const RECORD_DOCTEST: &str = "docs/research/index.md#smoke";
const RECORD_AUTHORED: &str = "docs/research/index.md#authored";
const RECORD_GENERATED: &str = "docs/research/index.md#orientation.research-map";

/// A green doctest contract: `printf` output matching its expectation.
const GREEN_DOCTEST: &str = "\
```doctest shell #smoke
$ printf kaya-smoke
kaya-smoke
```
";

/// A failing doctest contract: the expectation cannot match `printf`.
const FAILING_DOCTEST: &str = "\
```doctest shell #smoke
$ printf kaya-smoke
goodbye
```
";

/// A repository-maintained policy with no rules: edges decide alone.
const EMPTY_POLICY: &str = "# Impact rules: none yet; dependency edges decide.\n";

struct Fixture {
    dir: TempDir,
    cited: String,
    second: String,
}

impl Fixture {
    /// A committed record: the model seed committed before `build` (the
    /// ledger's committed bytes are the prefix build verifies against), then
    /// the compiled research and publication trees. The authored index cites
    /// the first seed event and declares one green doctest.
    fn new() -> Self {
        Self::with_doctest(GREEN_DOCTEST)
    }

    /// The same committed record with a doctest whose expectation cannot
    /// match, for gate-failure paths.
    fn with_failing_doctest() -> Self {
        Self::with_doctest(FAILING_DOCTEST)
    }

    fn with_doctest(doctest: &str) -> Self {
        let dir = tempfile::tempdir().expect("temp workspace");
        let root = dir.path();

        init_repository(root);
        run_ok(root, &["init"]);
        let seed = manifest_dir().join(SEED);
        let seed = seed.to_str().expect("the seed path is UTF-8");
        run_ok(root, &["import", "model", "--file", seed]);

        commit(root, "record the model seed");

        let events = ledger_events(root);
        let cited = events[0]["id"]
            .as_str()
            .expect("a seed event id")
            .to_owned();
        let second = events[1]["id"]
            .as_str()
            .expect("a seed event id")
            .to_owned();
        seed_authored_index(root, &cited, doctest);
        fs::write(root.join(RULES), EMPTY_POLICY).expect("seed the impact policy");

        run_ok(root, &["build"]);
        run_ok(root, &["publish"]);
        let index = fs::read_to_string(root.join(INDEX)).expect("the built index");
        assert!(
            index.contains(GENERATED_BEGIN) && index.contains(doctest),
            "the fixture index carries the managed block and the doctest: {index}"
        );

        commit(root, "commit the compiled record");
        Self { dir, cited, second }
    }

    fn root(&self) -> &Path {
        self.dir.path()
    }
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// A git repository with a fixed identity and no user configuration.
fn init_repository(root: &Path) {
    git(root, &["init", "--quiet"]);
    git(root, &["config", "user.name", "kaya impact"]);
    git(root, &["config", "user.email", "impact@example.invalid"]);
}

/// Seed the research index: a foreign managed block, an authored region
/// citing `cited`, and one declared doctest.
fn seed_authored_index(root: &Path, cited: &str, doctest: &str) {
    let path = root.join(INDEX);
    fs::create_dir_all(path.parent().expect("the index has a parent")).expect("research dir");
    let mut document = String::from("# Research index\n\n");
    document.push_str("<!-- other-tool:begin -->\nforeign\n<!-- other-tool:end -->\n\n");
    document.push_str(AUTHORED_BEGIN);
    document.push('\n');
    document.push_str(PROSE);
    document.push('\n');
    document.push_str(&format!("<!-- kaya:cites {cited} -->\n"));
    document.push_str(doctest);
    document.push_str(AUTHORED_END);
    document.push('\n');
    fs::write(&path, document).expect("seed the authored research index");
}

fn kaya(root: &Path, args: &[&str]) -> AssertCommand {
    let mut command = AssertCommand::cargo_bin("kaya").expect("kaya binary is built");
    command.current_dir(root);
    command.args(args);
    command
}

fn run(root: &Path, args: &[&str]) -> Output {
    kaya(root, args).output().expect("kaya runs")
}

#[track_caller]
fn run_ok(root: &Path, args: &[&str]) -> Value {
    let output = run(root, args);
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

/// Run the impact gate, asserting the `fail` exit contract (nonzero, here 2).
#[track_caller]
fn gate_failing(root: &Path, args: &[&str]) -> Value {
    let output = run(root, args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "a failed gate exits 2: {}",
        stderr(&output)
    );
    parse_envelope(&output)
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

/// Assert a refusal envelope and return its `(label, message)`.
#[track_caller]
fn assert_error(output: &Output, label_prefix: &str) -> (String, String) {
    assert_ne!(
        output.status.code(),
        Some(0),
        "a refusal exits nonzero: {}",
        stderr(output)
    );
    let envelope = parse_envelope(output);
    assert_eq!(
        envelope["ok"],
        json!(false),
        "a refusal is an error: {envelope}"
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

fn ledger_events(root: &Path) -> Vec<Value> {
    let text = fs::read_to_string(root.join(LEDGER)).expect("the ledger is readable");
    text.lines()
        .map(|line| {
            serde_json::from_str(line)
                .unwrap_or_else(|error| panic!("a ledger line is one JSON event ({error}): {line}"))
        })
        .collect()
}

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

/// Append a correction superseding `targets`, mirroring the staleness
/// fixture: the target's own record re-emitted with a fresh id and a
/// `data.supersedes` payload.
fn append_correction(root: &Path, targets: &[String]) {
    let events = ledger_events(root);
    let target = events
        .iter()
        .find(|event| targets.iter().any(|target| event["id"] == json!(target)))
        .unwrap_or_else(|| panic!("the cited id is in the ledger: {targets:?}"))
        .clone();
    let mut correction = target;
    correction["id"] = json!("evt-impact-correction-0000000000000");
    correction["timestamp"] = json!("2026-11-24T00:00:00Z");
    correction["data"] = json!({
        "supersedes": targets,
        "attributes": { "title": "corrected by the impact fixture" },
    });
    append_event(root, &correction);
}

/// Record one human disposition through the CLI, requiring success.
#[track_caller]
fn disposition(root: &Path, record: &str, kind: &str, extra: &[&str]) -> Value {
    let mut args = vec!["disposition", record, kind, "--actor", "alice"];
    args.extend(extra);
    run_ok(root, &args)
}

fn git(root: &Path, args: &[&str]) {
    let output = git_command(root).args(args).output().expect("git runs");
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Stage `paths` (the staged change set the gate reads).
fn stage(root: &Path, paths: &[&str]) {
    let mut args = vec!["add"];
    args.extend(paths);
    git(root, &args);
}

fn git_command(root: &Path) -> Command {
    let mut command = Command::new("git");
    command
        .arg("-C")
        .arg(root)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .env("GIT_AUTHOR_NAME", "kaya impact")
        .env("GIT_AUTHOR_EMAIL", "impact@example.invalid")
        .env("GIT_COMMITTER_NAME", "kaya impact")
        .env("GIT_COMMITTER_EMAIL", "impact@example.invalid")
        .env("GIT_AUTHOR_DATE", "2026-11-24T00:00:00+00:00")
        .env("GIT_COMMITTER_DATE", "2026-11-24T00:00:00+00:00");
    command
}

/// One deterministic commit.
fn commit(root: &Path, message: &str) {
    git(root, &["add", "--all"]);
    git(root, &["commit", "--quiet", "--message", message]);
}

/// Replace the fixture's authored prose with `replacement` in the index.
fn edit_authored_prose(root: &Path, replacement: &str) {
    let document = fs::read_to_string(root.join(INDEX)).expect("the index is readable");
    let updated = document.replacen(PROSE, replacement, 1);
    assert_ne!(updated, document, "the authored prose carries the edit");
    fs::write(root.join(INDEX), updated).expect("write the edited index");
}

/// Add a second cited id to the authored block's citation marker.
fn cite_second_event(root: &Path, second: &str) {
    let document = fs::read_to_string(root.join(INDEX)).expect("the index is readable");
    let marker = "<!-- kaya:cites ";
    let start = document
        .find(marker)
        .unwrap_or_else(|| panic!("the authored block cites evidence: {document}"));
    let close = start
        + document[start..]
            .find(" -->")
            .expect("the citation marker closes");
    let cited = &document[start + marker.len()..close];
    let updated = format!(
        "{}{cited} {second}{}",
        &document[..start + marker.len()],
        &document[close..]
    );
    fs::write(root.join(INDEX), updated).expect("write the widened citation");
}

/// The gate's report payload.
fn report(envelope: &Value) -> &Value {
    envelope
        .get("data")
        .filter(|data| !data.is_null())
        .unwrap_or_else(|| panic!("the gate reports data: {envelope}"))
}

/// The impact set as `(record, identified_by)` pairs.
fn impacts(envelope: &Value) -> Vec<(String, Value)> {
    report(envelope)["impacts"]
        .as_array()
        .unwrap_or_else(|| panic!("the report lists impacts: {envelope}"))
        .iter()
        .map(|impact| {
            (
                impact["record"].as_str().expect("a record id").to_owned(),
                impact["identified_by"].clone(),
            )
        })
        .collect()
}

#[track_caller]
fn assert_impacted(envelope: &Value, record: &str, identified_by: Value) {
    let hits = impacts(envelope)
        .into_iter()
        .filter(|(name, _)| name == record)
        .collect::<Vec<_>>();
    assert_eq!(
        hits.len(),
        1,
        "the impact set names {record} exactly once: {:?}",
        impacts(envelope)
    );
    assert_eq!(
        hits[0].1,
        identified_by,
        "{record} names what identified it: {:?}",
        impacts(envelope)
    );
}

#[track_caller]
fn assert_not_impacted(envelope: &Value, record: &str) {
    assert!(
        !impacts(envelope).iter().any(|(name, _)| name == record),
        "{record} is not in the impact set: {:?}",
        impacts(envelope)
    );
}

fn disposition_of(envelope: &Value, record: &str) -> String {
    report(envelope)["dispositions"][record]
        .as_str()
        .unwrap_or_else(|| panic!("the report carries a disposition for {record}: {envelope}"))
        .to_owned()
}

fn unmet(envelope: &Value) -> Vec<(String, String)> {
    report(envelope)["unmet"]
        .as_array()
        .expect("the report lists unmet obligations")
        .iter()
        .map(|entry| {
            (
                entry["record"].as_str().expect("a record id").to_owned(),
                entry["reason"].as_str().expect("a reason").to_owned(),
            )
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Mapping: dependency edges
// ---------------------------------------------------------------------------

/// A staged path that records depend on pulls every record of that document
/// into the impact set, each naming the edge that identified it; records of
/// untouched documents stay out.
#[test]
fn dependents_found() {
    let fixture = Fixture::new();
    let root = fixture.root();

    edit_authored_prose(root, "Fresh human interpretation after the change.");
    stage(root, &[INDEX]);

    let envelope = gate_failing(root, &["impact", "--staged"]);
    let edge = json!({ "edge": format!("path:{DOC}") });
    assert_impacted(&envelope, RECORD_AUTHORED, edge.clone());
    assert_impacted(&envelope, RECORD_DOCTEST, edge.clone());
    assert_impacted(&envelope, RECORD_GENERATED, edge);

    // Records of untouched documents are never named.
    let experiment_record = ledger_events(root)
        .iter()
        .find(|event| event["subject"]["type"] == json!("Experiment"))
        .map(|event| {
            event["subject"]["id"]
                .as_str()
                .expect("an experiment id")
                .to_owned()
        })
        .expect("the seed records an experiment");
    assert_not_impacted(
        &envelope,
        &format!(
            "docs/research/experiments/kaya-Experiment-{experiment_record}.md#orientation.research-map"
        ),
    );
}

/// A change set touching a path no record depends on yields an empty impact
/// set, and the report says so with a passing result.
#[test]
fn unaffected_excluded() {
    let fixture = Fixture::new();
    let root = fixture.root();

    fs::write(root.join("NOTES.md"), "scratch\n").expect("seed notes");
    stage(root, &["NOTES.md"]);

    let output = run(root, &["impact", "--staged"]);
    assert_eq!(output.status.code(), Some(0), "a passing gate exits 0");
    let envelope = parse_envelope(&output);
    assert!(impacts(&envelope).is_empty(), "the impact set is empty");
    assert_eq!(report(&envelope)["result"], json!("pass"));
}

// ---------------------------------------------------------------------------
// Dispositions
// ---------------------------------------------------------------------------

/// Three impacts with derived dispositions on two and none on the third:
/// the gate fails naming exactly the unmet record.
#[test]
fn impacts_accounted() {
    let fixture = Fixture::new();
    let root = fixture.root();

    disposition(root, RECORD_DOCTEST, "reviewed", &[]);
    edit_authored_prose(root, "Partially dispositioned interpretation.");
    stage(root, &[INDEX]);

    let envelope = gate_failing(root, &["impact", "--staged"]);
    assert_eq!(report(&envelope)["result"], json!("fail"));
    assert_eq!(
        unmet(&envelope),
        vec![(
            RECORD_GENERATED.to_owned(),
            "missing_disposition".to_owned()
        )],
        "the unmet list names exactly the undispositioned record"
    );
    assert_eq!(disposition_of(&envelope, RECORD_AUTHORED), "updated");
    assert_eq!(disposition_of(&envelope, RECORD_DOCTEST), "reviewed");
}

/// Derived dispositions need no recorded events: a green doctest run
/// establishes `validated`, regenerated-different bytes establish `updated`.
#[test]
fn dispositions_derived() {
    let fixture = Fixture::new();
    let root = fixture.root();

    let before = ledger_events(root).len();
    edit_authored_prose(root, "Regenerated interpretation, bytes differing.");
    stage(root, &[INDEX]);

    let output = run(root, &["impact", "--staged"]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "the unmet generated record fails the gate"
    );
    let envelope = parse_envelope(&output);
    assert_eq!(disposition_of(&envelope, RECORD_DOCTEST), "validated");
    assert_eq!(disposition_of(&envelope, RECORD_AUTHORED), "updated");
    assert_eq!(
        ledger_events(root).len(),
        before,
        "derived dispositions record no events"
    );
}

// ---------------------------------------------------------------------------
// Human dispositions
// ---------------------------------------------------------------------------

/// A `waived` disposition recorded with a tool actor is refused with a
/// labeled error and appends nothing.
#[test]
fn human_actor_required() {
    let fixture = Fixture::new();
    let root = fixture.root();

    let before = ledger_events(root).len();
    let output = run(
        root,
        &[
            "disposition",
            RECORD_AUTHORED,
            "waived",
            "--rationale",
            "accepted risk",
            "--actor",
            "robocop",
            "--actor-type",
            "tool",
        ],
    );
    let (label, _message) = assert_error(&output, "kaya.impact");
    assert!(
        label.contains("human"),
        "the refusal names the human-actor requirement: {label}"
    );
    assert_eq!(
        ledger_events(root).len(),
        before,
        "a refused disposition appends no event"
    );
}

// ---------------------------------------------------------------------------
// Stale authored blocks
// ---------------------------------------------------------------------------

/// A stale authored block is an unmet obligation regardless of any other
/// disposition, until the scoped acknowledgment is recorded.
#[test]
fn stale_unmet() {
    let fixture = Fixture::new();
    let root = fixture.root();

    disposition(root, RECORD_AUTHORED, "reviewed", &[]);
    append_correction(root, std::slice::from_ref(&fixture.cited));

    let envelope = gate_failing(root, &["impact", "--staged"]);
    assert_eq!(report(&envelope)["result"], json!("fail"));
    assert_eq!(
        unmet(&envelope),
        vec![(RECORD_AUTHORED.to_owned(), "stale_block".to_owned())],
        "staleness is unmet even under a reviewed disposition"
    );

    run_ok(
        root,
        &[
            "acknowledge",
            INDEX,
            "--block",
            "authored-1",
            "--events",
            &fixture.cited,
            "--actor",
            "alice",
        ],
    );

    let output = run(root, &["impact", "--staged"]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "the acknowledgment clears the obligation"
    );
    let envelope = parse_envelope(&output);
    assert_eq!(report(&envelope)["result"], json!("pass"));
}

/// A waiver binds the record's dependency digest and lapses when the
/// dependencies change again.
#[test]
fn waiver_lapses_when_dependencies_change() {
    let fixture = Fixture::new();
    let root = fixture.root();

    disposition(
        root,
        RECORD_AUTHORED,
        "waived",
        &["--rationale", "accepted risk"],
    );
    let output = run(root, &["impact", "--staged"]);
    assert_eq!(output.status.code(), Some(0), "a fresh waiver holds");
    let envelope = parse_envelope(&output);
    assert_eq!(
        report(&envelope)["waivers"][RECORD_AUTHORED]["lapsed"],
        json!(false),
        "the report lists the holding waiver"
    );

    cite_second_event(root, &fixture.second);
    stage(root, &[INDEX]);

    let envelope = gate_failing(root, &["impact", "--staged"]);
    assert_eq!(
        report(&envelope)["waivers"][RECORD_AUTHORED]["lapsed"],
        json!(true),
        "changed dependencies lapse the waiver"
    );
    let unmet = unmet(&envelope);
    assert!(
        unmet.contains(&(RECORD_AUTHORED.to_owned(), "lapsed_waiver".to_owned())),
        "a lapsed waiver is an unmet obligation again: {unmet:?}"
    );
    assert!(
        !unmet
            .iter()
            .any(|(record, reason)| record == RECORD_AUTHORED && reason != "lapsed_waiver"),
        "the lapsed waiver replaces every derived disposition: {unmet:?}"
    );
}

// ---------------------------------------------------------------------------
// Gate result contract
// ---------------------------------------------------------------------------

/// The exit code is zero on a green state and nonzero exactly on fail.
#[test]
fn exit_contract() {
    let fixture = Fixture::new();
    let root = fixture.root();

    let green = run(root, &["impact", "--staged"]);
    assert_eq!(green.status.code(), Some(0), "a green gate exits zero");
    assert_eq!(report(&parse_envelope(&green))["result"], json!("pass"));

    edit_authored_prose(root, "Broken interpretation without dispositions.");
    stage(root, &[INDEX]);

    let broken = run(root, &["impact", "--staged"]);
    assert_eq!(
        broken.status.code(),
        Some(2),
        "exactly a fail exits nonzero"
    );
    assert_eq!(report(&parse_envelope(&broken))["result"], json!("fail"));
}

/// Two gate runs on the same change set and ledger state print
/// byte-identical reports.
#[test]
fn report_deterministic() {
    let fixture = Fixture::new();
    let root = fixture.root();

    edit_authored_prose(root, "Deterministic interpretation.");
    stage(root, &[INDEX]);

    let first = run(root, &["impact", "--staged"]);
    let second = run(root, &["impact", "--staged"]);
    assert_eq!(
        first.stdout, second.stdout,
        "the reports are byte-identical on the same state"
    );
}

/// A failing doctest run in the impact set fails the gate, naming the record
/// and the failure, with a nonzero exit.
#[test]
fn failed_doctest_blocks() {
    let fixture = Fixture::with_failing_doctest();
    let root = fixture.root();

    disposition(root, RECORD_GENERATED, "reviewed", &[]);
    edit_authored_prose(root, "Interpretation beside a failing contract.");
    stage(root, &[INDEX]);

    let envelope = gate_failing(root, &["impact", "--staged"]);
    assert_eq!(report(&envelope)["result"], json!("fail"));
    let checks = report(&envelope)["checks"]
        .as_array()
        .expect("the report lists checks")
        .iter()
        .find(|check| check["record"] == json!(RECORD_DOCTEST))
        .expect("the doctest record carries a check")
        .clone();
    assert_eq!(checks["outcome"], json!("failed"));
    assert_eq!(checks["reason"], json!("mismatch"));
    assert_eq!(
        unmet(&envelope),
        vec![(RECORD_DOCTEST.to_owned(), "failed_check".to_owned())],
        "the failed check is the only unmet obligation"
    );
}

/// Every impact dispositioned and every mandatory check green: the gate
/// passes and exits zero.
#[test]
fn complete_dispositions_pass() {
    let fixture = Fixture::new();
    let root = fixture.root();

    disposition(root, RECORD_GENERATED, "reviewed", &[]);
    edit_authored_prose(root, "Fully dispositioned interpretation.");
    stage(root, &[INDEX]);

    let output = run(root, &["impact", "--staged"]);
    assert_eq!(output.status.code(), Some(0), "a complete state passes");
    let envelope = parse_envelope(&output);
    assert_eq!(report(&envelope)["result"], json!("pass"));
    assert!(unmet(&envelope).is_empty(), "no obligation is unmet");
}

/// The committed change between a base rev and HEAD gates the same way the
/// staged change does: edited authored prose derives `updated` against the
/// base's bytes, and the unedited generated record still owes a disposition.
#[test]
fn range_gate_maps_committed_changes() {
    let fixture = Fixture::new();
    let root = fixture.root();

    edit_authored_prose(root, "Committed interpretation.");
    commit(root, "commit the prose edit");
    disposition(root, RECORD_GENERATED, "reviewed", &[]);
    commit(root, "commit the disposition");

    let output = run(root, &["impact", "--range", "HEAD~2.."]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "the committed edit gates: {}",
        stderr(&output)
    );
    let envelope = parse_envelope(&output);
    assert_eq!(
        report(&envelope)["dispositions"][RECORD_AUTHORED],
        json!("updated"),
        "committed bytes differing from the base establish updated"
    );
    assert_eq!(
        report(&envelope)["dispositions"][RECORD_DOCTEST],
        json!("validated"),
        "the green mandatory check establishes validated with no recorded event"
    );
    assert!(
        unmet(&envelope).is_empty(),
        "updated and validated dispositions need no recorded event: {:?}",
        unmet(&envelope)
    );

    disposition(root, RECORD_DOCTEST, "reviewed", &[]);
    commit(root, "commit the last disposition");
    let output = run(root, &["impact", "--range", "HEAD~3"]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "a bare base rev works like a trailing `..`: {}",
        stderr(&output)
    );
    assert_eq!(report(&parse_envelope(&output))["result"], json!("pass"));
}

/// A policy rule that omits its severity is refused, never defaulted into a
/// hard blocker, and an inline `#` comment survives parsing.
#[test]
fn policy_parse_refusals_and_comments() {
    Policy::parse(
        "[[rule]]\nid = \"r\"\nmatch = \"docs/**\"\ntargets = \"docs/**\"\n",
    )
    .expect_err("a rule without a severity is refused");

    let policy = Policy::parse(
        "[[rule]]\nid = \"r\" # the only rule\nmatch = \"NOTES.md\"\ntargets = \"docs/**\"\nseverity = \"advisory\"\n",
    )
    .expect("an inline comment parses");
    assert_eq!(policy.rules.len(), 1, "the comment adds no rule");
    assert_eq!(policy.rules[0].id, "r", "the comment does not corrupt the id");
    assert_eq!(policy.rules[0].matches, "NOTES.md");

    Policy::parse("[[rule]]\nid = \"r\" trailing\nseverity = \"advisory\"\n")
        .expect_err("text after the closing quote that is not a comment is refused");
    Policy::parse("[[rule]]\nmatch = \"x\"\nseverity = \"advisory\"\n")
        .expect_err("a rule without an id is refused");
    Policy::parse("[[rule]]\nid = \"r\"\nmatch = \"x\"\ntargets = \"y\"\nseverity = \"loud\"\n")
        .expect_err("an out-of-vocabulary severity is refused");
}

/// Advisory findings produce a review result without blocking: exit zero,
/// each finding reported with its rule and records.
#[test]
fn advisory_only_review() {
    let fixture = Fixture::new();
    let root = fixture.root();

    fs::write(
        root.join(RULES),
        "[[rule]]\nid = \"notes-are-advisory\"\nmatch = \"NOTES.md\"\ntargets = \"docs/**\"\nseverity = \"advisory\"\n",
    )
    .expect("write the advisory policy");
    fs::write(root.join("NOTES.md"), "scratch\n").expect("seed notes");
    stage(root, &["NOTES.md"]);

    let output = run(root, &["impact", "--staged"]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "advisory findings never block"
    );
    let envelope = parse_envelope(&output);
    assert_eq!(report(&envelope)["result"], json!("review"));
    assert!(
        impacts(&envelope).is_empty(),
        "advisory rules add no mandatory impacts"
    );
    let findings = report(&envelope)["findings"]
        .as_array()
        .expect("the report lists findings");
    assert_eq!(
        findings[0]["rule"],
        json!("notes-are-advisory"),
        "each finding names the rule that fired"
    );
    assert!(
        !findings[0]["records"]
            .as_array()
            .expect("a finding names its records")
            .is_empty(),
        "the advisory finding names the records it affects"
    );
}

// ---------------------------------------------------------------------------
// The gate reads IR structure, not rendered markdown
// ---------------------------------------------------------------------------

/// Impact mapping walks the document IR's dependency and containment edges:
/// a changed path pulls the owning record and, transitively, every composed
/// record containing it, while records without intersecting edges stay out.
#[test]
fn structure_read_from_the_ir_not_from_rendered_markdown() {
    let doctest = Record {
        id: "d.md#doctest-1".to_owned(),
        kind: RecordKind::Doctest {
            runner: "shell".to_owned(),
        },
        content_hash: "hash".to_owned(),
        provenance: Vec::new(),
        cites: Vec::new(),
        artifacts: Vec::new(),
        contains: Vec::new(),
    };
    let composed = Record {
        id: "pub/page".to_owned(),
        kind: RecordKind::Composed,
        content_hash: "hash".to_owned(),
        provenance: Vec::new(),
        cites: Vec::new(),
        artifacts: Vec::new(),
        contains: vec!["d.md#doctest-1".to_owned()],
    };
    let other = Record {
        id: "other.md#authored-1".to_owned(),
        kind: RecordKind::AuthoredBlock,
        content_hash: "hash".to_owned(),
        provenance: Vec::new(),
        cites: Vec::new(),
        artifacts: Vec::new(),
        contains: Vec::new(),
    };
    let mut ir = DocumentIr::default();
    ir.records.insert(doctest.id.clone(), doctest);
    ir.records.insert(composed.id.clone(), composed);
    ir.records.insert(other.id.clone(), other);

    let changes = ChangeSet {
        paths: ["d.md".to_owned()].into_iter().collect(),
        event_ids: Default::default(),
    };

    let mapped = map_impacts(&ir, &changes, &Policy::default());
    assert_eq!(
        mapped.impacts.get("d.md#doctest-1"),
        Some(&Reason::Edge("path:d.md".to_owned())),
        "the changed path identifies its owning record"
    );
    assert_eq!(
        mapped.impacts.get("pub/page"),
        Some(&Reason::Edge("contains:d.md#doctest-1".to_owned())),
        "containment pulls the composed dependent in transitively"
    );
    assert!(
        !mapped.impacts.contains_key("other.md#authored-1"),
        "a record without intersecting edges is never named"
    );
}
