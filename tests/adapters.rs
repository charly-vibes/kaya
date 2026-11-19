//! Contract tests for `kaya.adapters::git` and `kaya.adapters::testrunner`:
//! deterministic event identity, source-read-only capture, provenance, and
//! atomic failure.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime};

use kaya::adapters::{git, testrunner};
use kaya::event::{Event, SCHEMA_V0, to_line};
use tempfile::TempDir;

const PROJECT: &str = "demo";
const FAILURE_LABEL: &str = "kaya.adapters.failure_atomic";

// ---------------------------------------------------------------------------
// git fixtures
// ---------------------------------------------------------------------------

fn git_command(dir: &Path) -> Command {
    let mut command = Command::new("git");
    command
        .arg("-C")
        .arg(dir)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .env("GIT_AUTHOR_NAME", "Kaya Fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.com")
        .env("GIT_COMMITTER_NAME", "Kaya Fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.com");
    command
}

fn git_output(dir: &Path, args: &[&str]) -> std::process::Output {
    git_command(dir)
        .args(args)
        .output()
        .expect("git must be installed and runnable")
}

fn git_ok(dir: &Path, args: &[&str]) -> String {
    let output = git_output(dir, args);
    assert!(
        output.status.success(),
        "git {args:?} failed in {}: {}",
        dir.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("git stdout is UTF-8")
}

fn init_repo() -> TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    git_ok(dir.path(), &["init", "--quiet"]);
    dir
}

fn commit(dir: &Path, message: &str, date: &str) -> String {
    let output = git_command(dir)
        .args(["commit", "--quiet", "--allow-empty", "-m", message])
        .env("GIT_AUTHOR_DATE", date)
        .env("GIT_COMMITTER_DATE", date)
        .output()
        .expect("git commit must run");
    assert!(
        output.status.success(),
        "git commit failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    git_ok(dir, &["rev-parse", "HEAD"]).trim().to_owned()
}

const COMMITS: [(&str, &str, &str); 3] = [
    (
        "initial scaffold",
        "2026-01-01T00:00:00+00:00",
        "2026-01-01T00:00:00Z",
    ),
    (
        "add ledger reader",
        "2026-01-02T03:04:05+02:00",
        "2026-01-02T01:04:05Z",
    ),
    (
        "tighten projection",
        "2026-03-04T05:06:07-05:00",
        "2026-03-04T10:06:07Z",
    ),
];

fn seed_commits(dir: &Path) -> Vec<String> {
    COMMITS
        .iter()
        .map(|&(message, date, _)| commit(dir, message, date))
        .collect()
}

fn version_line() -> String {
    let output = Command::new("git")
        .arg("--version")
        .output()
        .expect("git --version must run");
    assert!(output.status.success(), "git --version failed");
    String::from_utf8(output.stdout)
        .expect("git version is UTF-8")
        .trim()
        .to_owned()
}

// ---------------------------------------------------------------------------
// git adapter
// ---------------------------------------------------------------------------

#[test]
fn git_capture_emits_one_deterministic_event_per_commit() {
    let dir = init_repo();
    let shas = seed_commits(dir.path());
    let version = version_line();

    let status_before = git_ok(
        dir.path(),
        &["status", "--porcelain", "--untracked-files=all"],
    );
    let first = git::capture(dir.path(), PROJECT, 10).expect("capture succeeds on a repository");
    let second = git::capture(dir.path(), PROJECT, 10).expect("second capture succeeds");
    let status_after = git_ok(
        dir.path(),
        &["status", "--porcelain", "--untracked-files=all"],
    );

    assert_eq!(status_before, "", "fixture repository starts clean");
    assert_eq!(
        status_after, "",
        "capture must not write to the source repository"
    );

    assert_eq!(first.len(), COMMITS.len(), "one event per commit");
    let first_lines: Vec<String> = first
        .iter()
        .map(|event| to_line(event).expect("event data is finite"))
        .collect();
    let second_lines: Vec<String> = second
        .iter()
        .map(|event| to_line(event).expect("event data is finite"))
        .collect();
    assert_eq!(
        first_lines, second_lines,
        "capture must replay byte-identically"
    );
    assert_eq!(first, second, "capture must be deterministic");

    let ids: Vec<&str> = first.iter().map(|event| event.id.as_str()).collect();
    let mut sorted_ids = ids.clone();
    sorted_ids.sort_unstable();
    assert_eq!(ids, sorted_ids, "events are sorted by id");
    assert!(
        ids.iter()
            .all(|id| id.starts_with("evt-") && id.len() == 30)
    );

    let by_sha: HashMap<&str, &Event> = first
        .iter()
        .map(|event| (event.data["sha"].as_str().expect("sha is a string"), event))
        .collect();
    assert_eq!(by_sha.len(), COMMITS.len());

    for (index, sha) in shas.iter().enumerate() {
        let (message, _, expected_timestamp) = COMMITS[index];
        let event = by_sha
            .get(sha.as_str())
            .unwrap_or_else(|| panic!("missing event for commit {sha}"));
        let short = &sha[..7];
        assert_eq!(event.schema, SCHEMA_V0);
        assert_eq!(event.r#type, "capture.commit.observed");
        assert_eq!(event.subject.r#type, "Artifact");
        assert_eq!(event.subject.id, format!("{PROJECT}/Artifact/{short}"));
        assert_eq!(event.timestamp, expected_timestamp);
        assert_eq!(event.data["sha"].as_str(), Some(sha.as_str()));
        assert_eq!(event.data["short_sha"].as_str(), Some(short));
        assert_eq!(event.data["message"].as_str(), Some(message));
        assert_eq!(event.provenance.tool, "git");
        assert_eq!(event.provenance.version, version);
        assert_eq!(event.provenance.reference, *sha);
        assert_eq!(event.artifacts, vec![sha.clone()]);
        assert_eq!(event.actor.r#type.as_str(), "tool");
    }
}

#[test]
fn git_capture_skips_excluded_subjects() {
    let dir = init_repo();
    seed_commits(dir.path());

    let events = git::capture_excluding(dir.path(), PROJECT, 10, &["add ledger".to_owned()])
        .expect("capture succeeds");

    assert_eq!(events.len(), COMMITS.len() - 1, "one commit is skipped");
    assert!(
        events.iter().all(|event| !event.data["message"]
            .as_str()
            .unwrap_or("")
            .starts_with("add ledger")),
        "no excluded subject is recorded"
    );
}

#[test]
fn git_capture_honours_the_limit() {
    let dir = init_repo();
    let shas = seed_commits(dir.path());

    let events = git::capture(dir.path(), PROJECT, 2).expect("capture succeeds");
    assert_eq!(events.len(), 2, "limit caps the number of commits read");

    let mut captured: Vec<&str> = events
        .iter()
        .map(|event| event.data["sha"].as_str().expect("sha"))
        .collect();
    captured.sort_unstable();
    let mut expected = vec![shas[1].as_str(), shas[2].as_str()];
    expected.sort_unstable();
    assert_eq!(captured, expected, "limit keeps the newest commits");
}

#[test]
fn git_failure_is_labeled_and_emits_nothing() {
    let dir = tempfile::tempdir().expect("tempdir");
    let result = git::capture(dir.path(), PROJECT, 5);
    let error = result.expect_err("a non-repository must fail atomically");
    assert_eq!(error.label, FAILURE_LABEL);
    assert!(
        error.message.contains("git"),
        "message names the source: {}",
        error.message
    );
    assert!(
        error.message.contains(&dir.path().display().to_string()),
        "message names the source path: {}",
        error.message
    );
}

#[test]
fn git_capture_on_unborn_head_yields_no_events() {
    let dir = init_repo();

    let events = git::capture(dir.path(), PROJECT, 10)
        .expect("an unborn HEAD is an empty history, not a failure");

    assert!(
        events.is_empty(),
        "a repository without commits has no history"
    );
    assert_eq!(
        git_ok(
            dir.path(),
            &["status", "--porcelain", "--untracked-files=all"]
        ),
        "",
        "capture must not write to the source repository"
    );
}

#[test]
fn git_event_id_digests_the_project_namespace() {
    let dir = init_repo();
    let shas = seed_commits(dir.path());

    let first = git::capture(dir.path(), PROJECT, 10).expect("capture succeeds");
    let second = git::capture(dir.path(), "alternate", 10).expect("second capture succeeds");

    let ids_by_sha = |events: &[Event]| -> HashMap<String, String> {
        events
            .iter()
            .map(|event| {
                (
                    event.data["sha"].as_str().expect("sha").to_owned(),
                    event.id.clone(),
                )
            })
            .collect()
    };
    let first_ids = ids_by_sha(&first);
    let second_ids = ids_by_sha(&second);

    for sha in shas {
        assert_ne!(
            first_ids[&sha], second_ids[&sha],
            "the project namespace is part of commit {sha}'s event identity"
        );
    }
}

// ---------------------------------------------------------------------------
// test runner fixtures
// ---------------------------------------------------------------------------

const MTIME_SECONDS: u64 = 1_700_000_000;
const MTIME_RFC3339: &str = "2023-11-14T22:13:20Z";

fn write_report(dir: &Path, name: &str, body: &str) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, body).expect("write report fixture");
    let file = fs::OpenOptions::new()
        .write(true)
        .open(&path)
        .expect("open report fixture");
    file.set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(MTIME_SECONDS))
        .expect("pin report mtime");
    path
}

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

#[test]
fn testrunner_capture_summarises_the_run_deterministically() {
    let dir = tempfile::tempdir().expect("tempdir");
    let report = write_report(dir.path(), "cargo-test-results.jsonl", LIBTEST_REPORT);
    let reference = report.display().to_string();

    let first = testrunner::capture(&report, PROJECT).expect("capture succeeds");
    let second = testrunner::capture(&report, PROJECT).expect("second capture succeeds");

    assert_eq!(
        to_line(&first).expect("event data is finite"),
        to_line(&second).expect("event data is finite"),
        "capture replays identically"
    );
    assert_eq!(first, second);
    assert!(first.id.starts_with("evt-"));
    assert_eq!(first.id.len(), 30);

    assert_eq!(first.schema, SCHEMA_V0);
    assert_eq!(first.r#type, "experiment.run.completed");
    assert_eq!(first.subject.r#type, "Run");
    assert_eq!(
        first.subject.id,
        format!("{PROJECT}/Run/cargo-test-results")
    );
    assert_eq!(first.timestamp, MTIME_RFC3339);
    assert_eq!(first.data["passed"].as_u64(), Some(2));
    assert_eq!(first.data["failed"].as_u64(), Some(1));
    assert_eq!(first.data["ignored"].as_u64(), Some(1));
    assert_eq!(first.data["total"].as_u64(), Some(4));
    assert_eq!(first.provenance.tool, "test-runner");
    assert_eq!(first.provenance.version, "libtest-json");
    assert_eq!(first.provenance.reference, reference);
    assert_eq!(first.artifacts, vec![reference]);
    assert_eq!(first.actor.r#type.as_str(), "tool");
}

#[test]
fn testrunner_event_id_digests_the_project_namespace() {
    let dir = tempfile::tempdir().expect("tempdir");
    let report = write_report(dir.path(), "cargo-test-results.jsonl", LIBTEST_REPORT);

    let first = testrunner::capture(&report, PROJECT).expect("capture succeeds");
    let second = testrunner::capture(&report, "alternate").expect("second capture succeeds");

    assert_eq!(
        first.provenance.reference, second.provenance.reference,
        "both captures reference the same report"
    );
    assert_eq!(
        first.data, second.data,
        "the same report produces the same payload"
    );
    assert_ne!(
        first.id, second.id,
        "the project namespace is part of the run identity"
    );
}

#[test]
fn testrunner_malformed_line_fails_atomically() {
    let dir = tempfile::tempdir().expect("tempdir");
    let body = concat!(
        "{\"type\":\"test\",\"event\":\"ok\",\"name\":\"passes\"}\n",
        "{\"type\":\"test\",\"event\":\"ok\"\n",
    );
    let report = write_report(dir.path(), "broken.jsonl", body);

    let error = testrunner::capture(&report, PROJECT).expect_err("malformed JSON must fail");
    assert_eq!(error.label, FAILURE_LABEL);
    assert!(
        error.message.contains("line 2"),
        "message names the offending line: {}",
        error.message
    );
    assert!(
        error.message.contains(&report.display().to_string()),
        "message names the report: {}",
        error.message
    );
}

#[test]
fn testrunner_unknown_test_event_fails_atomically() {
    let dir = tempfile::tempdir().expect("tempdir");
    let report = write_report(
        dir.path(),
        "unknown.jsonl",
        "{\"type\":\"test\",\"event\":\"bench\",\"name\":\"weird\"}\n",
    );

    let error = testrunner::capture(&report, PROJECT).expect_err("unknown test event must fail");
    assert_eq!(error.label, FAILURE_LABEL);
    assert!(
        error.message.contains("bench"),
        "message names the unknown event: {}",
        error.message
    );
}

#[test]
fn testrunner_missing_report_fails_atomically() {
    let dir = tempfile::tempdir().expect("tempdir");
    let missing = dir.path().join("absent.jsonl");

    let error = testrunner::capture(&missing, PROJECT).expect_err("missing report must fail");
    assert_eq!(error.label, FAILURE_LABEL);
    assert!(
        error.message.contains("absent.jsonl"),
        "message names the source: {}",
        error.message
    );
}
