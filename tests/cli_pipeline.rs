//! Integration tests for the CLI pipeline: `import`, `build`, `promote`,
//! `challenge`, and the `status` proposal counter.
//!
//! Every case drives the real binary in a tempfile project, writes its own
//! fixture tree, and reads the one JSON envelope each invocation prints.

use assert_cmd::Command;
use serde_json::{Value, json};
use std::fs;
use std::process::Command as StdCommand;
use tempfile::TempDir;

const LEDGER: &str = ".kaya/ledger.jsonl";
const INDEX: &str = "docs/research/index.md";

fn kaya(dir: &TempDir) -> Command {
    let mut command = Command::cargo_bin("kaya").expect("kaya binary is built");
    command.current_dir(dir.path());
    command
}

fn run(dir: &TempDir, args: &[&str]) -> std::process::Output {
    kaya(dir).args(args).output().expect("kaya runs")
}

fn import_wai(dir: &TempDir) -> std::process::Output {
    run(dir, &["import", "wai"])
}

fn promote(dir: &TempDir, assertion_id: &str, actor: &str) -> std::process::Output {
    run(dir, &["promote", assertion_id, "--actor", actor])
}

fn challenge(
    dir: &TempDir,
    conclusion_id: &str,
    events: &[&str],
    actor: &str,
) -> std::process::Output {
    let mut args = vec!["challenge", conclusion_id, "--events"];
    args.extend(events.iter().copied());
    args.push("--actor");
    args.push(actor);
    run(dir, &args)
}

fn parse_envelope(output: &std::process::Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|err| {
        panic!(
            "expected one JSON envelope on stdout ({err}); stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

#[track_caller]
fn assert_success(output: &std::process::Output) -> Value {
    assert!(
        output.status.success(),
        "expected success (exit {:?}); stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    let envelope = parse_envelope(output);
    assert_eq!(envelope["ok"], json!(true));
    assert!(
        !envelope
            .as_object()
            .expect("envelope is a JSON object")
            .contains_key("error"),
        "success envelopes omit the error field"
    );
    envelope
}

#[track_caller]
fn assert_failure(output: &std::process::Output) -> Value {
    assert_eq!(
        output.status.code(),
        Some(1),
        "expected exit 1; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let envelope = parse_envelope(output);
    assert_eq!(envelope["ok"], json!(false));
    assert_eq!(envelope["envelope_kind"], json!("error"));
    assert_eq!(envelope["data"], Value::Null);
    envelope
}

fn event_line(event: &Value) -> String {
    format!(
        "{}\n",
        serde_json::to_string(event).expect("event serializes")
    )
}

fn init(dir: &TempDir) {
    assert_success(&run(dir, &["init"]));
}

fn write_wai_design(dir: &TempDir, name: &str) {
    let path = dir
        .path()
        .join(".wai")
        .join("projects")
        .join("kaya")
        .join("designs")
        .join(name);
    fs::create_dir_all(path.parent().expect("design dir")).unwrap();
    fs::write(&path, "# design\n").unwrap();
}

fn write_ledger(dir: &TempDir, lines: &[String]) {
    let path = dir.path().join(LEDGER);
    fs::create_dir_all(path.parent().expect("ledger dir")).unwrap();
    fs::write(&path, lines.concat()).unwrap();
}

fn ledger_lines(dir: &TempDir) -> Vec<String> {
    fs::read_to_string(dir.path().join(LEDGER))
        .expect("ledger is readable")
        .lines()
        .map(str::to_owned)
        .collect()
}

fn assertion_event(id: &str, text: &str, cites: &[&str]) -> Value {
    json!({
        "id": id,
        "schema": "kaya.event/v0",
        "type": "promotion.assertion.recorded",
        "timestamp": "2026-11-16T10:00:00Z",
        "actor": { "type": "human", "name": "alice" },
        "subject": { "type": "assertion", "id": id },
        "data": {
            "text": text,
            "status": "proposed",
            "event_ids": cites,
            "machine_proposed": false,
        },
        "provenance": { "tool": "kaya", "version": "0.1.0", "reference": id },
        "artifacts": [],
    })
}

fn node_event(event_id: &str, kind: &str, node_id: &str, title: &str) -> Value {
    json!({
        "id": event_id,
        "schema": "kaya.event/v0",
        "type": "model.entity.created",
        "timestamp": "2026-11-16T10:00:00Z",
        "actor": { "type": "human", "name": "alice" },
        "subject": { "type": kind, "id": node_id },
        "data": { "attributes": { "title": title } },
        "provenance": { "tool": "kaya", "version": "0.1.0", "reference": event_id },
        "artifacts": [],
    })
}

fn relationship_event(event_id: &str, relationship: &str, from: &str, to: &str) -> Value {
    json!({
        "id": event_id,
        "schema": "kaya.event/v0",
        "type": "model.relationship.asserted",
        "timestamp": "2026-11-16T10:00:00Z",
        "actor": { "type": "human", "name": "alice" },
        "subject": { "type": "Relationship", "id": from },
        "data": { "relationship": relationship, "from": from, "to": to },
        "provenance": { "tool": "kaya", "version": "0.1.0", "reference": event_id },
        "artifacts": [],
    })
}

fn experiment_event(event_id: &str, node_id: &str, title: &str) -> Value {
    node_event(event_id, "Experiment", node_id, title)
}

fn artifact_event(event_id: &str, node_id: &str, kind: &str, path: &str) -> Value {
    json!({
        "id": event_id,
        "schema": "kaya.event/v0",
        "type": "capture.artifact.observed",
        "timestamp": "2026-11-16T10:00:00Z",
        "actor": { "type": "tool", "name": "wai" },
        "subject": { "type": "Artifact", "id": node_id },
        "data": { "kind": kind, "path": path },
        "provenance": {
            "tool": "wai",
            "version": "2026.10.10",
            "reference": path
        },
        "artifacts": [path],
    })
}

fn source_event(event_id: &str, node_id: &str, path: &str, commit: &str) -> Value {
    json!({
        "id": event_id,
        "schema": "kaya.event/v0",
        "type": "model.entity.created",
        "timestamp": "2026-11-16T10:00:00Z",
        "actor": { "type": "human", "name": "alice" },
        "subject": { "type": "Source", "id": node_id },
        "data": { "attributes": { "path": path, "commit": commit } },
        "provenance": { "tool": "kaya", "version": "0.1.0", "reference": event_id },
        "artifacts": [],
    })
}

fn run_event(event_id: &str, run_id: &str, passed: u64) -> Value {
    json!({
        "id": event_id,
        "schema": "kaya.event/v0",
        "type": "experiment.run.completed",
        "timestamp": "2026-11-16T10:00:00Z",
        "actor": { "type": "tool", "name": "test-runner" },
        "subject": { "type": "Run", "id": run_id },
        "data": { "passed": passed, "failed": 0, "ignored": 0, "total": passed },
        "provenance": {
            "tool": "test-runner",
            "version": "libtest-json",
            "reference": "target/run.jsonl"
        },
        "artifacts": [],
    })
}

fn run_correction(event_id: &str, run_id: &str, supersedes: &str, passed: u64) -> Value {
    let mut event = run_event(event_id, run_id, passed);
    event["data"]["supersedes"] = json!(supersedes);
    event
}

fn git(dir: &TempDir, args: &[&str]) {
    let output = StdCommand::new("git")
        .current_dir(dir.path())
        .args(args)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .env("GIT_AUTHOR_NAME", "kaya")
        .env("GIT_AUTHOR_EMAIL", "kaya@example.com")
        .env("GIT_COMMITTER_NAME", "kaya")
        .env("GIT_COMMITTER_EMAIL", "kaya@example.com")
        .output()
        .expect("git runs");
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn commit_ledger(dir: &TempDir) {
    git(dir, &["init", "--quiet"]);
    git(dir, &["add", LEDGER]);
    git(dir, &["commit", "--quiet", "--message", "record ledger"]);
}

fn record_assertion(dir: &TempDir, id: &str, text: &str, cites: &[&str]) {
    let line = event_line(&assertion_event(id, text, cites));
    write_ledger(dir, &[line]);
}

#[test]
fn import_wai_appends_each_event_once() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    write_wai_design(&dir, "x.md");

    let first = assert_success(&import_wai(&dir));
    assert_eq!(first["envelope_kind"], json!("import"));
    assert_eq!(first["data"]["source"], json!("wai"));
    assert_eq!(first["data"]["events"], json!(1));
    assert_eq!(first["data"]["appended"], json!(1));
    assert_eq!(first["data"]["duplicates"], json!(0));
    assert_eq!(first["data"]["ledger"], json!(LEDGER));

    let stored = ledger_lines(&dir);
    assert_eq!(stored.len(), 1, "one artifact yields one ledger line");
    let event: Value = serde_json::from_str(&stored[0]).unwrap();
    assert_eq!(event["schema"], json!("kaya.event/v0"));
    assert_eq!(event["type"], json!("capture.artifact.observed"));
    assert_eq!(
        event["data"]["path"],
        json!(".wai/projects/kaya/designs/x.md")
    );

    let second = assert_success(&import_wai(&dir));
    assert_eq!(second["envelope_kind"], json!("import"));
    assert_eq!(second["data"]["events"], json!(1));
    assert_eq!(second["data"]["appended"], json!(0));
    assert_eq!(second["data"]["duplicates"], json!(1));
    assert_eq!(
        ledger_lines(&dir).len(),
        1,
        "a duplicate import never appends"
    );
}

#[test]
fn import_without_a_ledger_is_a_missing_ledger_error() {
    let dir = tempfile::tempdir().unwrap();

    let envelope = assert_failure(&import_wai(&dir));
    assert_eq!(envelope["error"]["label"], json!("missing-ledger"));
    let message = envelope["error"]["message"]
        .as_str()
        .expect("error envelopes carry a message");
    assert!(
        message.contains("kaya init"),
        "the message must suggest `kaya init`: {message}"
    );
    assert!(
        !dir.path().join(LEDGER).exists(),
        "import must not create a missing ledger"
    );
}

#[test]
fn failed_import_writes_no_events_to_the_ledger() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    write_wai_design(&dir, "x.md");

    let unparsable = dir
        .path()
        .join(".wai/projects/kaya/research/undated-binary.md");
    fs::create_dir_all(unparsable.parent().expect("research dir")).unwrap();
    fs::write(&unparsable, [0xff_u8, 0xfe]).unwrap();

    let envelope = assert_failure(&import_wai(&dir));
    assert_eq!(
        envelope["error"]["label"],
        json!("kaya.adapters.failure_atomic")
    );
    let message = envelope["error"]["message"]
        .as_str()
        .expect("error envelopes carry a message");
    assert!(message.contains("wai"), "names the source: {message}");
    assert!(
        message.contains("undated-binary.md"),
        "names the record: {message}"
    );
    assert!(
        ledger_lines(&dir).is_empty(),
        "a failed adapter run appends no events"
    );
}

#[test]
fn build_writes_index_with_generated_block_and_artifact() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    write_wai_design(&dir, "x.md");
    assert_success(&import_wai(&dir));

    let built = assert_success(&run(&dir, &["build", "--allow-uncommitted"]));
    assert_eq!(built["envelope_kind"], json!("build"));
    assert_eq!(built["data"]["out"], json!("docs/research"));
    assert_eq!(built["data"]["pending_proposals"], json!(0));
    assert_eq!(built["data"]["stale"], json!([]));
    assert!(
        built["data"]["ledger_digest"].is_string(),
        "build surfaces the ledger digest so callers can pin it: {built}"
    );
    let recomputed = built["data"]["recomputed"]
        .as_array()
        .expect("recomputed is a list");
    assert!(
        recomputed
            .iter()
            .any(|name| name == "orientation.research-map"),
        "a fresh research map is recomputed: {recomputed:?}"
    );

    let index = fs::read_to_string(dir.path().join(INDEX)).expect("index exists");
    assert!(
        index.contains("<!-- kaya:begin generated:orientation.research-map -->"),
        "index carries the generated block marker: {index}"
    );
    assert!(
        index.contains(".wai/projects/kaya/designs/x.md"),
        "index lists the observed artifact: {index}"
    );

    let again = assert_success(&run(&dir, &["build", "--allow-uncommitted"]));
    let unchanged = again["data"]["unchanged"]
        .as_array()
        .expect("unchanged is a list");
    assert!(
        unchanged
            .iter()
            .any(|name| name == "orientation.research-map"),
        "a byte-identical rebuild is unchanged: {unchanged:?}"
    );
    assert_eq!(
        again["data"]["recomputed"],
        json!([]),
        "nothing is rewritten when the block matches"
    );
}

#[test]
fn build_writes_one_report_per_experiment() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    let experiment = event_line(&experiment_event(
        "entity-1",
        "kaya/Experiment/exp-one",
        "Exp One",
    ));
    let other_project = event_line(&experiment_event(
        "entity-2",
        "alpha/Experiment/exp-one",
        "Other Exp One",
    ));
    // Each experiment declares intent, so a specification block renders.
    let question = event_line(&node_event(
        "entity-3",
        "Question",
        "kaya/Question/q-one",
        "Q One",
    ));
    let other_question = event_line(&node_event(
        "entity-4",
        "Question",
        "alpha/Question/q-one",
        "Other Q One",
    ));
    let investigates = event_line(&relationship_event(
        "rel-1",
        "investigates",
        "kaya/Question/q-one",
        "kaya/Experiment/exp-one",
    ));
    let other_investigates = event_line(&relationship_event(
        "rel-2",
        "investigates",
        "alpha/Question/q-one",
        "alpha/Experiment/exp-one",
    ));
    write_ledger(
        &dir,
        &[
            experiment,
            other_project,
            question,
            other_question,
            investigates,
            other_investigates,
        ],
    );

    let built = assert_success(&run(&dir, &["build", "--allow-uncommitted"]));
    let recomputed = built["data"]["recomputed"]
        .as_array()
        .expect("recomputed is a list");
    for name in [
        "exploration.experiment-specification:kaya/Experiment/exp-one",
        "exploration.experiment-specification:alpha/Experiment/exp-one",
    ] {
        assert!(
            recomputed.iter().any(|candidate| candidate == name),
            "each specification block has a distinct summary name ({name}): {recomputed:?}"
        );
    }

    for path in [
        "docs/research/experiments/kaya-Experiment-exp-one.md",
        "docs/research/experiments/alpha-Experiment-exp-one.md",
    ] {
        let report = fs::read_to_string(dir.path().join(path))
            .unwrap_or_else(|err| panic!("the report path sanitizes every id for {path}: {err}"));
        assert!(
            report.contains("<!-- kaya:begin generated:exploration.experiment-specification -->"),
            "report carries the specification block marker: {report}"
        );
        assert!(
            !report.contains("exploration.experiment-record"),
            "an experiment with no runs writes no record block: {report}"
        );
    }

    let sources = built["data"]["sources"]
        .as_object()
        .expect("build exposes the merged source map");
    assert!(
        sources.contains_key("exploration.experiment-specification:kaya/Experiment/exp-one")
            && sources
                .contains_key("exploration.experiment-specification:alpha/Experiment/exp-one"),
        "each report has its own source-map entry: {sources:?}"
    );

    let index = fs::read_to_string(dir.path().join(INDEX)).expect("index exists");
    assert!(
        index.contains("kaya/Experiment/exp-one") && index.contains("alpha/Experiment/exp-one"),
        "index lists both experiments: {index}"
    );
    assert!(
        index.contains("_4 nodes from 6 events._"),
        "the index summary counts rendered nodes and ledger events: {index}"
    );
}

#[test]
fn omitted_kinds_reported_once_in_build_envelope() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    let lines = [
        event_line(&experiment_event(
            "entity-1",
            "kaya/Experiment/exp-one",
            "Exp One",
        )),
        event_line(&experiment_event(
            "entity-2",
            "kaya/Experiment/exp-two",
            "Exp Two",
        )),
        event_line(&node_event(
            "entity-3",
            "Question",
            "kaya/Question/q-one",
            "Q One",
        )),
        event_line(&node_event(
            "entity-4",
            "Question",
            "kaya/Question/q-two",
            "Q Two",
        )),
        event_line(&relationship_event(
            "rel-1",
            "investigates",
            "kaya/Question/q-one",
            "kaya/Experiment/exp-one",
        )),
        event_line(&relationship_event(
            "rel-2",
            "investigates",
            "kaya/Question/q-two",
            "kaya/Experiment/exp-two",
        )),
        // Only exp-one has a run; exp-two has no records to report.
        event_line(&run_event("run-1", "kaya/Run/run-1", 1)),
        event_line(&relationship_event(
            "rel-3",
            "executes",
            "kaya/Run/run-1",
            "kaya/Experiment/exp-one",
        )),
    ];
    write_ledger(&dir, &lines);

    let built = assert_success(&run(&dir, &["build", "--allow-uncommitted"]));
    assert_eq!(
        built["data"]["omitted"],
        json!([
            {
                "family": "exploration",
                "kind": "experiment-record",
                "reason": "no-records",
            },
            {
                "family": "reuse-handoff",
                "kind": "handoff",
                "reason": "no-records",
            },
            {
                "family": "reuse-handoff",
                "kind": "reproduction-package",
                "reason": "no-records",
            },
        ]),
        "each omitted kind is named exactly once, sorted and deduplicated: {built}"
    );

    let recomputed = built["data"]["recomputed"]
        .as_array()
        .expect("recomputed is a list");
    assert!(
        recomputed
            .iter()
            .any(|name| name == "exploration.experiment-record:kaya/Experiment/exp-one"),
        "the experiment with a run writes a record block: {recomputed:?}"
    );
    assert!(
        !recomputed
            .iter()
            .any(|name| name == "exploration.experiment-record:kaya/Experiment/exp-two"),
        "an omitted kind has no summary key: {recomputed:?}"
    );

    let with_run = fs::read_to_string(
        dir.path()
            .join("docs/research/experiments/kaya-Experiment-exp-one.md"),
    )
    .expect("the run-bearing report exists");
    assert!(
        with_run.contains("<!-- kaya:begin generated:exploration.experiment-record -->"),
        "a kind with records keeps its block: {with_run}"
    );

    let without_run = fs::read_to_string(
        dir.path()
            .join("docs/research/experiments/kaya-Experiment-exp-two.md"),
    )
    .expect("the runless report exists");
    assert!(
        without_run.contains("<!-- kaya:begin generated:exploration.experiment-specification -->"),
        "the runless experiment still renders its specification: {without_run}"
    );
    assert!(
        !without_run.contains("exploration.experiment-record"),
        "an empty kind leaves no block, markers, or placeholder: {without_run}"
    );
}

#[test]
fn omitted_kinds_are_sorted_and_deduped_across_experiments() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    // exp-a and exp-c declare intent but have no runs; exp-b has no edges at
    // all, so it omits both kinds and gets no report document.
    let lines = [
        event_line(&experiment_event(
            "entity-1",
            "kaya/Experiment/exp-a",
            "Exp A",
        )),
        event_line(&experiment_event(
            "entity-2",
            "kaya/Experiment/exp-b",
            "Exp B",
        )),
        event_line(&experiment_event(
            "entity-3",
            "kaya/Experiment/exp-c",
            "Exp C",
        )),
        event_line(&node_event(
            "entity-4",
            "Question",
            "kaya/Question/q-a",
            "Q A",
        )),
        event_line(&node_event(
            "entity-5",
            "Question",
            "kaya/Question/q-c",
            "Q C",
        )),
        event_line(&relationship_event(
            "rel-1",
            "investigates",
            "kaya/Question/q-a",
            "kaya/Experiment/exp-a",
        )),
        event_line(&relationship_event(
            "rel-2",
            "investigates",
            "kaya/Question/q-c",
            "kaya/Experiment/exp-c",
        )),
    ];
    write_ledger(&dir, &lines);

    let built = assert_success(&run(&dir, &["build", "--allow-uncommitted"]));
    assert_eq!(
        built["data"]["omitted"],
        json!([
            {
                "family": "exploration",
                "kind": "experiment-record",
                "reason": "no-records",
            },
            {
                "family": "exploration",
                "kind": "experiment-specification",
                "reason": "no-records",
            },
            {
                "family": "reuse-handoff",
                "kind": "handoff",
                "reason": "no-records",
            },
            {
                "family": "reuse-handoff",
                "kind": "reproduction-package",
                "reason": "no-records",
            },
        ]),
        "three experiments yield one sorted, deduplicated pair of omissions: {built}"
    );

    assert!(
        !dir.path()
            .join("docs/research/experiments/kaya-Experiment-exp-b.md")
            .exists(),
        "an experiment with nothing to render gets no report document"
    );
}

#[test]
fn build_tolerates_a_missing_report_when_corrections_exist() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    // The correction makes `flag_stale_authored` inspect every compiled
    // document, including the report the edge-less experiment never writes.
    let lines = [
        event_line(&experiment_event(
            "entity-1",
            "kaya/Experiment/exp-empty",
            "Empty Exp",
        )),
        event_line(&run_event("run-1", "kaya/Run/run-1", 1)),
        event_line(&run_correction(
            "correction-1",
            "kaya/Run/run-1",
            "run-1",
            2,
        )),
    ];
    write_ledger(&dir, &lines);

    let built = assert_success(&run(&dir, &["build", "--allow-uncommitted"]));
    assert_eq!(
        built["data"]["omitted"],
        json!([
            {
                "family": "exploration",
                "kind": "experiment-record",
                "reason": "no-records",
            },
            {
                "family": "exploration",
                "kind": "experiment-specification",
                "reason": "no-records",
            },
            {
                "family": "reuse-handoff",
                "kind": "handoff",
                "reason": "no-records",
            },
            {
                "family": "reuse-handoff",
                "kind": "reproduction-package",
                "reason": "no-records",
            },
        ]),
        "a runless, edgeless experiment omits both kinds: {built}"
    );
    assert!(
        !dir.path()
            .join("docs/research/experiments/kaya-Experiment-exp-empty.md")
            .exists(),
        "no document is written when both kinds are omitted"
    );
}

#[test]
fn ledger_assertions_do_not_create_a_record_block_for_a_runless_experiment() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    // The assertion cites evidence unrelated to this experiment: only
    // declared intent renders, and the record kind stays omitted.
    let lines = [
        event_line(&experiment_event(
            "entity-1",
            "kaya/Experiment/exp-one",
            "Exp One",
        )),
        event_line(&node_event(
            "entity-2",
            "Question",
            "kaya/Question/q-one",
            "Q One",
        )),
        event_line(&relationship_event(
            "rel-1",
            "investigates",
            "kaya/Question/q-one",
            "kaya/Experiment/exp-one",
        )),
        event_line(&assertion_event(
            "assertion-1",
            "the model holds",
            &["evt-unrelated"],
        )),
    ];
    write_ledger(&dir, &lines);

    let built = assert_success(&run(&dir, &["build", "--allow-uncommitted"]));
    let report = fs::read_to_string(
        dir.path()
            .join("docs/research/experiments/kaya-Experiment-exp-one.md"),
    )
    .expect("the specification-bearing report exists");
    assert!(
        report.contains("<!-- kaya:begin generated:exploration.experiment-specification -->"),
        "declared intent still renders: {report}"
    );
    assert!(
        !report.contains("exploration.experiment-record"),
        "a ledger-global assertion must not materialize a record block: {report}"
    );
    assert!(
        !report.contains("## Interpretations"),
        "no interpretation section survives without observed execution: {report}"
    );
    assert_eq!(
        built["data"]["omitted"],
        json!([
            {
                "family": "exploration",
                "kind": "experiment-record",
                "reason": "no-records",
            },
            {
                "family": "reuse-handoff",
                "kind": "handoff",
                "reason": "no-records",
            },
            {
                "family": "reuse-handoff",
                "kind": "reproduction-package",
                "reason": "no-records",
            },
        ]),
        "the record kind is omitted even though an assertion exists: {built}"
    );
}

#[test]
fn build_expect_digest_pins_the_ledger() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    write_wai_design(&dir, "x.md");
    assert_success(&import_wai(&dir));

    let built = assert_success(&run(&dir, &["build", "--allow-uncommitted"]));
    let digest = built["data"]["ledger_digest"]
        .as_str()
        .expect("build reports its ledger digest")
        .to_owned();

    let pinned = assert_success(&run(&dir, &["build", "--expect-digest", digest.as_str()]));
    assert_eq!(pinned["data"]["ledger_digest"], json!(digest));
    assert!(
        pinned["data"]["unchanged"]
            .as_array()
            .expect("unchanged is a list")
            .iter()
            .any(|name| name == "orientation.research-map"),
        "the pinned rebuild leaves the research map alone"
    );

    let bogus = "0".repeat(64);
    let rejected = assert_failure(&run(&dir, &["build", "--expect-digest", bogus.as_str()]));
    assert_eq!(
        rejected["error"]["label"],
        json!("kaya.ledger.digest_mismatch"),
        "an unpinned ledger is refused, not served"
    );
}

#[test]
fn build_flags_authored_citations_superseded_by_a_correction() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    let authored = "# Notes\n\n\
        <!-- kaya:authored -->\n\
        The run was fast.\n\
        <!-- kaya:cites run-1 -->\n\
        <!-- kaya:end authored -->\n";
    fs::create_dir_all(dir.path().join("docs/research")).unwrap();
    fs::write(dir.path().join(INDEX), authored).unwrap();

    let run_line = event_line(&run_event("run-1", "kaya/Run/run-1", 1));
    let correction = event_line(&run_correction(
        "correction-1",
        "kaya/Run/run-1",
        "run-1",
        2,
    ));
    write_ledger(&dir, &[run_line, correction]);

    let built = assert_success(&run(&dir, &["build", "--allow-uncommitted"]));
    assert_eq!(
        built["data"]["stale"],
        json!([{ "block": "authored-1", "evidence": ["run-1"] }]),
        "the correction moved evidence cited by authored prose"
    );

    let after = fs::read_to_string(dir.path().join(INDEX)).expect("index exists");
    assert!(
        after.starts_with(authored),
        "authored bytes are never rewritten: {after}"
    );
    assert!(
        after.contains("_1 nodes from 2 events._"),
        "the summary counts rendered nodes and all ledger events: {after}"
    );
}

#[test]
fn build_refuses_an_in_place_edit_of_committed_ledger_history() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    let first = event_line(&run_event("run-1", "kaya/Run/run-1", 1));
    write_ledger(&dir, &[first]);
    commit_ledger(&dir);

    let path = dir.path().join(LEDGER);
    let committed = fs::read_to_string(&path).expect("ledger is readable");
    fs::write(&path, committed.replacen("\"run-1\"", "\"run-edited\"", 1)).unwrap();

    let envelope = assert_failure(&run(&dir, &["build", "--allow-uncommitted"]));
    assert_eq!(
        envelope["error"]["label"],
        json!("kaya.ledger.uncommitted_history"),
        "rewritten committed bytes are never served, even with --allow-uncommitted"
    );
    let message = envelope["error"]["message"]
        .as_str()
        .expect("error envelopes carry a message");
    assert!(
        message.contains("line 1"),
        "the refusal names the first unverified line: {message}"
    );
}

#[test]
fn build_accepts_committed_ledger_growth_with_complete_lines() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    let first = event_line(&run_event("run-1", "kaya/Run/run-1", 1));
    write_ledger(&dir, std::slice::from_ref(&first));
    commit_ledger(&dir);

    let second = event_line(&run_event("run-2", "kaya/Run/run-2", 2));
    write_ledger(&dir, &[first, second]);

    let built = assert_success(&run(&dir, &["build"]));
    assert!(
        built["data"]["recomputed"]
            .as_array()
            .expect("recomputed is a list")
            .iter()
            .any(|name| name == "orientation.research-map"),
        "a complete committed-growth suffix is verified and built: {built}"
    );
}

#[test]
fn build_accepts_an_untracked_ledger_only_when_allowed_or_pinned() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    write_ledger(
        &dir,
        &[event_line(&run_event("run-1", "kaya/Run/run-1", 1))],
    );

    let refused = assert_failure(&run(&dir, &["build"]));
    assert_eq!(
        refused["error"]["label"],
        json!("kaya.ledger.uncommitted_history")
    );
    let message = refused["error"]["message"]
        .as_str()
        .expect("error envelopes carry a message");
    assert!(
        message.contains("--expect-digest") && message.contains("--allow-uncommitted"),
        "the refusal explains how to pin or accept the ledger: {message}"
    );

    let built = assert_success(&run(&dir, &["build", "--allow-uncommitted"]));
    assert_eq!(built["data"]["pending_proposals"], json!(0));
}

#[test]
fn build_writes_reference_documents_and_leaves_the_ledger_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    let handoff_path = ".wai/projects/kaya/handoffs/h.md";
    write_ledger(
        &dir,
        &[event_line(&artifact_event(
            "artifact-1",
            "kaya/Artifact/handoffs-h",
            "handoffs",
            handoff_path,
        ))],
    );

    let ledger = dir.path().join(LEDGER);
    let bytes_before = fs::read(&ledger).expect("the ledger is readable");
    let events_before = ledger_lines(&dir).len();

    let built = assert_success(&run(&dir, &["build", "--allow-uncommitted"]));

    let recomputed = built["data"]["recomputed"]
        .as_array()
        .expect("recomputed is a list");
    assert!(
        recomputed
            .iter()
            .any(|name| name == "reuse-handoff.handoff"),
        "the handoff document reports its reuse block: {recomputed:?}"
    );
    assert_eq!(
        built["data"]["omitted"],
        json!([{
            "family": "reuse-handoff",
            "kind": "reproduction-package",
            "reason": "no-records",
        }]),
        "no Source nodes means the reproduction package is omitted: {built}"
    );

    let handoff = fs::read_to_string(dir.path().join("docs/research/handoff.md"))
        .expect("the handoff document is written");
    assert!(
        handoff.contains("<!-- kaya:begin generated:reuse-handoff.handoff -->"),
        "the handoff document carries its generated block marker: {handoff}"
    );
    assert!(
        handoff.contains(handoff_path),
        "every handoff row names its wai artifact reference: {handoff}"
    );
    assert!(
        handoff.contains("kaya/Artifact/handoffs-h"),
        "the reference row keeps the source node id: {handoff}"
    );
    assert!(
        !handoff.contains("reuse-handoff.reproduction-package"),
        "an omitted kind leaves no block in the handoff document: {handoff}"
    );
    assert!(
        !dir.path()
            .join("docs/research/reproduction-package.md")
            .exists(),
        "an omitted kind gets no document of its own"
    );

    let bytes_after = fs::read(&ledger).expect("the ledger is readable");
    assert_eq!(
        bytes_after, bytes_before,
        "a reuse document copies no payload into the ledger"
    );
    assert_eq!(
        ledger_lines(&dir).len(),
        events_before,
        "a reuse document creates no ledger entity"
    );
}

#[test]
fn build_refuses_a_legacy_block_in_a_reuse_document() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    let handoff_path = ".wai/projects/kaya/handoffs/h.md";
    // A Source row means the reproduction package has something to write:
    // the refusal must happen before that document is created.
    write_ledger(
        &dir,
        &[
            event_line(&source_event(
                "source-1",
                "kaya/Source/src-main-rs@9f2c1a",
                "src/main.rs",
                "9f2c1a",
            )),
            event_line(&artifact_event(
                "artifact-1",
                "kaya/Artifact/handoffs-h",
                "handoffs",
                handoff_path,
            )),
        ],
    );

    let seeded = "\
# Handoffs

<!-- kaya:begin generated:experiment-report -->
## Runs

- kaya/Run/run-1
<!-- kaya:end generated:experiment-report -->
";
    let document = dir.path().join("docs/research/handoff.md");
    fs::create_dir_all(document.parent().expect("document dir")).unwrap();
    fs::write(&document, seeded).unwrap();

    let envelope = assert_failure(&run(&dir, &["build", "--allow-uncommitted"]));
    assert_eq!(
        envelope["error"]["label"],
        json!("kaya.documents.legacy_block_refused"),
        "a legacy block in a reuse document is refused by the vocabulary gate"
    );
    let message = envelope["error"]["message"]
        .as_str()
        .expect("error envelopes carry a message");
    assert!(
        message.contains("experiment-report"),
        "the refusal names the offending block: {message}"
    );
    assert_eq!(
        fs::read_to_string(&document).expect("the document is readable"),
        seeded,
        "a refused document keeps its bytes untouched"
    );
    assert!(
        !dir.path()
            .join("docs/research/reproduction-package.md")
            .exists(),
        "the reproduction package is not written before the refusal"
    );
}

#[test]
fn status_counts_pending_proposals() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    record_assertion(&dir, "assertion-1", "the model holds", &["evt-1"]);

    let status = assert_success(&run(&dir, &["status"]));
    assert_eq!(status["envelope_kind"], json!("status"));
    assert_eq!(status["data"]["events"], json!(1));
    assert_eq!(
        status["data"]["pending_proposals"],
        json!(1),
        "one proposed assertion awaits a human"
    );
}

#[test]
fn promote_then_challenge_records_human_events() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    record_assertion(&dir, "assertion-1", "the model holds", &["evt-1"]);

    let promoted = assert_success(&promote(&dir, "assertion-1", "alice"));
    assert_eq!(promoted["envelope_kind"], json!("promote"));
    assert_eq!(
        promoted["data"]["event"]["type"],
        json!("promotion.conclusion.established")
    );
    assert_eq!(promoted["data"]["event"]["actor"]["type"], json!("human"));
    assert_eq!(
        promoted["data"]["event"]["data"]["assertion"],
        json!("assertion-1")
    );
    assert_eq!(ledger_lines(&dir).len(), 2, "approval is one new event");

    let status = assert_success(&run(&dir, &["status"]));
    assert_eq!(status["data"]["pending_proposals"], json!(0));

    let challenged = assert_success(&challenge(&dir, "assertion-1", &["evt-2", "evt-3"], "bob"));
    assert_eq!(challenged["envelope_kind"], json!("challenge"));
    assert_eq!(
        challenged["data"]["event"]["type"],
        json!("promotion.conclusion.challenged")
    );
    assert_eq!(
        challenged["data"]["event"]["data"]["counter_event_ids"],
        json!(["evt-2", "evt-3"])
    );
    assert_eq!(ledger_lines(&dir).len(), 3, "challenge is one new event");
}

#[test]
fn promote_without_a_recorded_assertion_is_a_promotion_refusal() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);

    let envelope = assert_failure(&promote(&dir, "assertion-missing", "alice"));
    let label = envelope["error"]["label"]
        .as_str()
        .expect("error envelopes carry a label");
    assert!(
        label.starts_with("kaya.promotion."),
        "the refusal carries the promotion label: {label}"
    );
    assert_eq!(label, "kaya.promotion.unknown_assertion");
    assert!(
        ledger_lines(&dir).is_empty(),
        "a refused promotion appends nothing"
    );
}

#[test]
fn challenge_of_an_unestablished_conclusion_is_a_promotion_refusal() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);
    record_assertion(&dir, "assertion-1", "the model holds", &["evt-1"]);

    let envelope = assert_failure(&challenge(&dir, "assertion-1", &["evt-2"], "bob"));
    assert_eq!(
        envelope["error"]["label"],
        json!("kaya.promotion.challenge_target")
    );
    assert_eq!(
        ledger_lines(&dir).len(),
        1,
        "a refused challenge appends nothing"
    );
}

#[test]
fn unknown_subcommand_returns_the_usage_envelope() {
    let dir = tempfile::tempdir().unwrap();

    let envelope = assert_failure(&run(&dir, &["frobnicate"]));
    assert_eq!(envelope["error"]["label"], json!("usage"));
    let message = envelope["error"]["message"]
        .as_str()
        .expect("error envelopes carry a message");
    assert!(
        message.contains("frobnicate"),
        "usage message must name the offending subcommand: {message}"
    );
}
