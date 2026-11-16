//! Test-runner adapter: machine-readable run output as one observed event.
//!
//! Purpose: turn a libtest JSON report into a single
//! `experiment.run.completed` record carrying the run's counts.
//! Responsibilities: parse libtest JSONL, ignore non-test records, and
//! refuse malformed or unknown records atomically.
//! Rationale: a run's counts are facts the runner already owns; kaya only
//! references the report and derives a stable id from the tool, its version,
//! the project namespace, the observation timestamp, the report path, and the
//! serialized counts so replaying the same report yields the same ledger line
//! while a capture from another environment cannot collide with it.

use std::path::Path;

use chrono::{DateTime, SecondsFormat, Utc};
use serde_json::Value;

use crate::adapters::AdapterError;
use crate::event::{Actor, ActorType, Event, Provenance, SCHEMA_V0, Subject};
use crate::ledger::sha256_hex;

const LABEL_FAILURE_ATOMIC: &str = "kaya.adapters.failure_atomic";
const SOURCE: &str = "test-runner";
const VERSION: &str = "libtest-json";
const EVENT_TYPE: &str = "experiment.run.completed";
const ID_PREFIX: &str = "evt-";
const ID_DIGEST_CHARS: usize = 26;
const FALLBACK_SLUG_CHARS: usize = 7;

/// Tallies of terminal libtest outcomes in one report.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Counts {
    passed: u64,
    failed: u64,
    ignored: u64,
}

/// Summarise the libtest JSONL report at `report` as one completed-run event.
///
/// The report is only read: its mtime becomes the observation timestamp and
/// its path becomes the provenance reference.
pub fn capture(report: &Path, project: &str) -> Result<Event, AdapterError> {
    let text = std::fs::read_to_string(report).map_err(|err| {
        failure(&format!(
            "test-runner: cannot read report {}: {err}",
            report.display()
        ))
    })?;

    let mut counts = Counts::default();
    for (index, line) in text.lines().enumerate() {
        let number = index + 1;
        let value: Value = serde_json::from_str(line).map_err(|err| {
            failure(&format!(
                "test-runner: report {} line {number} is not valid JSON: {err}",
                report.display()
            ))
        })?;
        let Some(record) = value.as_object() else {
            continue;
        };
        if record.get("type").and_then(Value::as_str) != Some("test") {
            continue;
        }
        let Some(event) = record.get("event").and_then(Value::as_str) else {
            return Err(failure(&format!(
                "test-runner: report {} line {number} is a test record without a string \"event\"",
                report.display()
            )));
        };
        match event {
            "started" => {}
            "ok" => counts.passed += 1,
            "failed" => counts.failed += 1,
            "ignored" => counts.ignored += 1,
            other => {
                return Err(failure(&format!(
                    "test-runner: report {} line {number} carries unknown test event \"{other}\"; expected started, ok, failed, or ignored",
                    report.display()
                )));
            }
        }
    }

    let modified = std::fs::metadata(report)
        .and_then(|metadata| metadata.modified())
        .map_err(|err| {
            failure(&format!(
                "test-runner: cannot read the modification time of {}: {err}",
                report.display()
            ))
        })?;
    let observed: DateTime<Utc> = DateTime::<Utc>::from(modified);
    let timestamp = observed.to_rfc3339_opts(SecondsFormat::Secs, true);

    let total = counts.passed + counts.failed + counts.ignored;
    let slug = slug_for(report);
    let reference = report.display().to_string();
    let data = serde_json::json!({
        "passed": counts.passed,
        "failed": counts.failed,
        "ignored": counts.ignored,
        "total": total,
    });
    let identity =
        format!("{SOURCE}\x1f{VERSION}\x1f{project}\x1f{timestamp}\x1f{reference}\x1f{data}");
    let digest = sha256_hex(identity.as_bytes());

    Ok(Event {
        id: format!("{ID_PREFIX}{}", &digest[..ID_DIGEST_CHARS]),
        schema: SCHEMA_V0.to_owned(),
        r#type: EVENT_TYPE.to_owned(),
        timestamp,
        actor: Actor {
            r#type: ActorType::Tool,
            name: SOURCE.to_owned(),
        },
        subject: Subject {
            r#type: "Run".to_owned(),
            id: format!("{project}/Run/{slug}"),
        },
        data,
        provenance: Provenance {
            tool: SOURCE.to_owned(),
            version: VERSION.to_owned(),
            reference,
        },
        artifacts: vec![report.display().to_string()],
    })
}

/// The report stem as a lowercase slug: non-alphanumerics collapse to `-`.
///
/// A stem that slugs to nothing falls back to a digest of the report path so
/// every captured run still has a stable key.
fn slug_for(report: &Path) -> String {
    let stem = report
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or_default();
    let mut slug = String::with_capacity(stem.len());
    for character in stem.chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
        } else if !slug.is_empty() && !slug.ends_with('-') {
            slug.push('-');
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() {
        let digest = sha256_hex(report.to_string_lossy().as_bytes());
        slug = format!("run-{}", &digest[..FALLBACK_SLUG_CHARS]);
    }
    slug
}

fn failure(message: &str) -> AdapterError {
    AdapterError::new(LABEL_FAILURE_ATOMIC, message)
}
