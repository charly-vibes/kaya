//! Contract tests for the test-runner input path on a stable toolchain.
//!
//! `kaya import test --report` consumes a libtest JSONL report. The stable
//! Rust toolchain cannot produce one — libtest's `--format json` is nightly
//! only, and stable libtest rejects `-Z unstable-options` outright — so a
//! repository on stable needs a converter from the JUnit XML that nextest
//! does emit.
//!
//! Every case drives the real converter script with `python3` and the real
//! `kaya` binary in a throwaway repository, then reads the one JSON envelope
//! each invocation prints. The conversion is checked line by line, the round
//! trip through the adapter is checked through the ledger, and a malformed
//! report is checked to produce no output at all.

use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;
use tempfile::TempDir;

/// The JUnit XML nextest writes: one `<testsuite>` per test binary, one
/// `<testcase>` per test, and the outcome element naming the result.
const JUNIT_REPORT: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="nextest-run" tests="4" failures="1" errors="1" uuid="45c50042-482e-477e-88a2-60cfcc3eaf95" timestamp="2024-01-09T07:50:12.664+00:00" time="0.023">
    <testsuite name="fixture-project::basic" tests="4" disabled="0" errors="1" failures="1">
        <testcase name="test_passes" classname="fixture-project::basic" timestamp="2024-01-09T07:50:12.665+00:00" time="0.004">
        </testcase>
        <testcase name="test_fails" classname="fixture-project::basic" timestamp="2024-01-09T07:50:12.665+00:00" time="0.004">
            <failure type="test failure">thread 'test_fails' panicked at tests/basic.rs:19:5:
assertion `left == right` failed: this is an assertion
  left: 4
 right: 5</failure>
        </testcase>
        <testcase name="test_errors" classname="fixture-project::basic" timestamp="2024-01-09T07:50:12.665+00:00" time="0.004">
            <error type="test error">thread 'test_errors' panicked at tests/basic.rs:23:5:
this is an error</error>
        </testcase>
        <testcase name="test_skipped" classname="fixture-project::basic" timestamp="2024-01-09T07:50:12.665+00:00" time="0.004">
            <skipped/>
        </testcase>
    </testsuite>
</testsuites>
"#;

/// A JUnit document that is not well-formed XML.
const MALFORMED_REPORT: &str = "<testsuites><testsuite><testcase name=\"broken\"";

/// Where the converter writes the report the adapter reads.
const REPORT_PATH: &str = "cargo-test-results.jsonl";

/// The converter script, resolved against the crate root.
fn converter() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("scripts")
        .join("junit-to-libtest.py")
}

/// Run the converter over `report` and write the result to `out`.
fn convert(report: &Path, out: &Path) -> std::process::Output {
    StdCommand::new("python3")
        .arg(converter())
        .arg(report)
        .arg(out)
        .output()
        .expect("python3 must be available to run the converter")
}

/// The `kaya` binary, run in `dir`.
fn kaya(dir: &TempDir) -> Command {
    let mut command = Command::cargo_bin("kaya").expect("kaya binary is built");
    command.current_dir(dir.path());
    command
}

/// Parse an envelope the `kaya` binary printed.
fn parse_envelope(output: &std::process::Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|err| {
        panic!(
            "expected one JSON envelope on stdout ({err}); stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

/// Write `body` to `path` under `dir` and return the path.
fn write(dir: &TempDir, path: &str, body: &str) -> PathBuf {
    let full = dir.path().join(path);
    fs::write(&full, body).expect("write the fixture");
    full
}

/// The converted report's lines, parsed as JSON objects.
fn report_lines(path: &Path) -> Vec<Value> {
    let text = fs::read_to_string(path).expect("the converted report is readable");
    text.lines()
        .map(|line| {
            serde_json::from_str(line)
                .unwrap_or_else(|err| panic!("each line is one JSON object ({err}): {line}"))
        })
        .collect()
}

/// The `event` of every test record in the converted report, in order.
fn events(path: &Path) -> Vec<String> {
    report_lines(path)
        .iter()
        .map(|record| {
            record["event"]
                .as_str()
                .unwrap_or_else(|| panic!("every record names its event: {record}"))
                .to_owned()
        })
        .collect()
}

/// The name of every test record in the converted report, in order.
fn names(path: &Path) -> Vec<String> {
    report_lines(path)
        .iter()
        .map(|record| {
            record["name"]
                .as_str()
                .unwrap_or_else(|| panic!("every record names its test: {record}"))
                .to_owned()
        })
        .collect()
}

#[test]
fn conversion_names_every_test_and_its_outcome() {
    let dir = tempfile::tempdir().expect("temp workspace");
    let report = write(&dir, "junit.xml", JUNIT_REPORT);
    let out = dir.path().join(REPORT_PATH);

    let output = convert(&report, &out);

    assert!(
        output.status.success(),
        "the converter must succeed on a well-formed report; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let lines = report_lines(&out);
    assert_eq!(lines.len(), 4, "one record per test case: {lines:?}");
    for record in &lines {
        assert_eq!(record["type"], "test");
    }
    assert_eq!(
        names(&out),
        vec![
            "fixture-project::basic::test_passes",
            "fixture-project::basic::test_fails",
            "fixture-project::basic::test_errors",
            "fixture-project::basic::test_skipped",
        ],
        "every test is named in report order, qualified by the binary it ran in \
         the way libtest qualifies its own names"
    );
    assert_eq!(
        events(&out),
        vec!["ok", "failed", "failed", "ignored"],
        "the outcome element names the result: a failure and an error both fail, \
         a skipped test is ignored"
    );
}

#[test]
fn converted_report_imports_once_and_dedupes_the_second_time() {
    let dir = tempfile::tempdir().expect("temp workspace");
    let report = write(&dir, "junit.xml", JUNIT_REPORT);
    let out = dir.path().join(REPORT_PATH);
    let converted = convert(&report, &out);
    assert!(
        converted.status.success(),
        "the converter must succeed; stderr: {}",
        String::from_utf8_lossy(&converted.stderr)
    );

    let init = kaya(&dir).arg("init").output().expect("kaya runs");
    assert!(init.status.success(), "init must succeed");

    let first = kaya(&dir)
        .args(["import", "test", "--report", REPORT_PATH])
        .output()
        .expect("kaya runs");
    assert!(
        first.status.success(),
        "the first import must succeed; stderr: {}",
        String::from_utf8_lossy(&first.stderr)
    );
    let envelope = parse_envelope(&first);
    assert_eq!(envelope["ok"], Value::Bool(true));
    assert_eq!(envelope["data"]["source"], "test-runner");
    assert_eq!(envelope["data"]["events"], 1, "one run is one event");
    assert_eq!(envelope["data"]["appended"], 1);
    assert_eq!(envelope["data"]["duplicates"], 0);

    let second = kaya(&dir)
        .args(["import", "test", "--report", REPORT_PATH])
        .output()
        .expect("kaya runs");
    assert!(
        second.status.success(),
        "the second import must succeed; stderr: {}",
        String::from_utf8_lossy(&second.stderr)
    );
    let envelope = parse_envelope(&second);
    assert_eq!(
        envelope["data"]["appended"], 0,
        "the same report adds no second event: {envelope}"
    );
    assert_eq!(
        envelope["data"]["duplicates"], 1,
        "the same report is a duplicate: {envelope}"
    );

    // The counted outcomes reach the record: one passed, two failed, one
    // ignored.
    let ledger =
        fs::read_to_string(dir.path().join(".kaya/ledger.jsonl")).expect("the ledger is readable");
    assert_eq!(
        ledger.lines().count(),
        1,
        "the ledger holds exactly one run event: {ledger}"
    );
    let event: Value = serde_json::from_str(ledger.lines().next().expect("one line"))
        .expect("the ledger line is one JSON event");
    assert_eq!(event["data"]["passed"], 1);
    assert_eq!(event["data"]["failed"], 2);
    assert_eq!(event["data"]["ignored"], 1);
    assert_eq!(event["data"]["total"], 4);
}

#[test]
fn malformed_report_is_refused_and_writes_nothing() {
    let dir = tempfile::tempdir().expect("temp workspace");
    let report = write(&dir, "junit.xml", MALFORMED_REPORT);
    let out = dir.path().join(REPORT_PATH);

    let output = convert(&report, &out);

    assert!(
        !output.status.success(),
        "a malformed report must not succeed; stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        !out.exists(),
        "a refused conversion writes no report: {}",
        out.display()
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("junit.xml"),
        "the refusal names the report it could not read: {stderr}"
    );
}
