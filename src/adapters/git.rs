//! Git adapter: commit history as observed events.
//!
//! Purpose: translate the commits a repository already owns into
//! `kaya.event/v0` records without authoring anything new.
//! Responsibilities: run a read-only `git log`, parse its records, and emit
//! one deterministic `capture.commit.observed` event per commit.
//! Rationale: the commit sha is the source's own identity and the only
//! anchor a later projection can resolve, so every event carries it in both
//! provenance and artifacts, while the event id digests that sha plus every
//! other field the event writes (tool, version, project, timestamp, payload)
//! so an environment change cannot masquerade as the same observation.

use std::path::Path;
use std::process::Command;

use chrono::{DateTime, SecondsFormat, Utc};

use crate::adapters::AdapterError;
use crate::event::{Actor, ActorType, Event, Provenance, SCHEMA_V0, Subject};
use crate::ledger::sha256_hex;

const LABEL_FAILURE_ATOMIC: &str = "kaya.adapters.failure_atomic";
const SOURCE: &str = "git";
const EVENT_TYPE: &str = "capture.commit.observed";
const ID_PREFIX: &str = "evt-";
const ID_DIGEST_CHARS: usize = 26;
const SHORT_SHA_CHARS: usize = 7;
const FIELD_SEPARATOR: char = '\x1f';
const LOG_FORMAT: &str = "--format=%H%x1f%aI%x1f%s";

/// Capture up to `limit` commits from the repository at `root`.
///
/// Events are sorted by id and carry the commit sha as their source
/// reference. A fresh repository whose HEAD is unborn has no history to
/// observe and yields an empty vector rather than a failure. Nothing is
/// written under `root`.
pub fn capture(root: &Path, project: &str, limit: usize) -> Result<Vec<Event>, AdapterError> {
    capture_excluding(root, project, limit, &[])
}

/// Capture up to `limit` commits, skipping subjects that start with any prefix.
///
/// A prefix is repository-owned policy: the scheduled capture job skips its own
/// snapshot commits so the record reaches a fixed point instead of observing
/// yesterday's snapshot forever. An empty prefix list captures everything.
pub fn capture_excluding(
    root: &Path,
    project: &str,
    limit: usize,
    excluded_prefixes: &[String],
) -> Result<Vec<Event>, AdapterError> {
    // Verify the directory is a repository before treating a HEAD probe miss
    // as merely an unborn branch; a non-repository still fails atomically.
    checked_git(root, &["rev-parse", "--git-dir"], "git rev-parse --git-dir")?;
    if !has_head(root)? {
        return Ok(Vec::new());
    }
    let version = git_version(root)?;
    let log_limit = limit.to_string();
    let log = checked_git(root, &["log", LOG_FORMAT, "-n", &log_limit], "git log")?;

    let mut events = Vec::new();
    for (index, line) in log.lines().enumerate() {
        if line.is_empty() {
            continue;
        }
        let number = index + 1;
        let mut fields = line.splitn(3, FIELD_SEPARATOR);
        let (Some(sha), Some(author_date), Some(message)) =
            (fields.next(), fields.next(), fields.next())
        else {
            return Err(failure(&format!(
                "git: commit record {number} in {} is malformed; expected sha{sep}author-date{sep}subject",
                root.display(),
                sep = FIELD_SEPARATOR
            )));
        };
        if sha.len() < SHORT_SHA_CHARS || !sha.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(failure(&format!(
                "git: commit record {number} in {} carries malformed sha \"{sha}\"",
                root.display()
            )));
        }
        if excluded_prefixes
            .iter()
            .any(|prefix| !prefix.is_empty() && message.starts_with(prefix))
        {
            continue;
        }
        let Some(timestamp) = timestamp_of(author_date) else {
            return Err(failure(&format!(
                "git: commit record {number} in {} carries author date \"{author_date}\" that is not RFC 3339",
                root.display()
            )));
        };

        let short_sha = &sha[..SHORT_SHA_CHARS];
        let data = serde_json::json!({
            "sha": sha,
            "message": message,
            "short_sha": short_sha,
        });
        let digest = event_id(&version, project, &timestamp, sha, &data);
        events.push(Event {
            id: format!("{ID_PREFIX}{}", &digest[..ID_DIGEST_CHARS]),
            schema: SCHEMA_V0.to_owned(),
            r#type: EVENT_TYPE.to_owned(),
            timestamp,
            actor: Actor {
                r#type: ActorType::Tool,
                name: SOURCE.to_owned(),
            },
            subject: Subject {
                r#type: "Artifact".to_owned(),
                id: format!("{project}/Artifact/{short_sha}"),
            },
            data,
            provenance: Provenance {
                tool: SOURCE.to_owned(),
                version: version.clone(),
                reference: sha.to_owned(),
            },
            artifacts: vec![sha.to_owned()],
        });
    }

    events.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(events)
}

/// The source's self-reported version, one trimmed line.
fn git_version(root: &Path) -> Result<String, AdapterError> {
    let raw = checked_git(root, &["--version"], "git --version")?;
    let version = raw.trim().to_owned();
    if version.is_empty() {
        return Err(failure("git: `git --version` reported no version"));
    }
    Ok(version)
}

/// Deterministic event id over every field the event writes.
///
/// Tool, tool version, project namespace, timestamp, commit reference, and the
/// serialized payload all contribute, so captures from different environments
/// never share an id while a replay of the same environment does.
fn event_id(
    version: &str,
    project: &str,
    timestamp: &str,
    reference: &str,
    data: &serde_json::Value,
) -> String {
    let identity =
        format!("{SOURCE}\x1f{version}\x1f{project}\x1f{timestamp}\x1f{reference}\x1f{data}");
    sha256_hex(identity.as_bytes())
}

/// Whether `HEAD` resolves to a commit; an unborn branch reports `false`.
fn has_head(root: &Path) -> Result<bool, AdapterError> {
    let output = Command::new(SOURCE)
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "--verify", "HEAD"])
        .output()
        .map_err(|err| {
            failure(&format!(
                "git: cannot run `git rev-parse --verify HEAD` in {}: {err}",
                root.display()
            ))
        })?;
    Ok(output.status.success())
}

/// Convert a git strict-ISO author date into an RFC 3339 UTC `Z` stamp.
fn timestamp_of(value: &str) -> Option<String> {
    let parsed: DateTime<chrono::FixedOffset> = DateTime::parse_from_rfc3339(value).ok()?;
    Some(
        parsed
            .with_timezone(&Utc)
            .to_rfc3339_opts(SecondsFormat::Secs, true),
    )
}

/// Run one read-only git command against `root`, refusing failure and
/// non-UTF-8 output.
fn checked_git(root: &Path, args: &[&str], what: &str) -> Result<String, AdapterError> {
    let output = Command::new(SOURCE)
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .map_err(|err| {
            failure(&format!(
                "git: cannot run `{what}` in {}: {err}",
                root.display()
            ))
        })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let status = output
            .status
            .code()
            .map_or_else(|| "signal".to_owned(), |code| code.to_string());
        return Err(failure(&format!(
            "git: `{what}` in {} exited with status {status}: {}",
            root.display(),
            stderr.trim()
        )));
    }
    String::from_utf8(output.stdout).map_err(|err| {
        failure(&format!(
            "git: `{what}` in {} produced non-UTF-8 output: {err}",
            root.display()
        ))
    })
}

fn failure(message: &str) -> AdapterError {
    AdapterError::new(LABEL_FAILURE_ATOMIC, message)
}
