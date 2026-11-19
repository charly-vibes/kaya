//! wai adapter: `.wai/` markdown artifacts to `capture.artifact.observed`.
//!
//! Purpose: record that a research, design, plan, or handoff artifact exists
//! in the wai artifact tree, without copying or interpreting its contents.
//! Responsibilities: walk the artifact tree read-only, derive a deterministic
//! event id per artifact, and abort atomically when a record cannot be read.
//! Rationale: wai owns the artifact text; kaya only needs a stable, replayable
//! observation naming where the artifact lives.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use chrono::{DateTime, NaiveDate, SecondsFormat, Utc};
use serde_json::json;

use crate::event::{self, Actor, ActorType, Event, Provenance, SCHEMA_V0, Subject};

use super::{AdapterError, LABEL_FAILURE_ATOMIC, sha256_hex};

/// Source tool named in every event's provenance.
const TOOL: &str = "wai";

/// Version of the wai artifact-tree layout this adapter reads.
const SOURCE_VERSION: &str = "2026.10.10";

/// Event type emitted once per markdown artifact.
const EVENT_TYPE: &str = "capture.artifact.observed";

/// Artifact kinds, matching the source directory names.
const KINDS: [&str; 4] = ["research", "designs", "plans", "handoffs"];

/// Length of a leading `YYYY-MM-DD` date in a file name.
const DATE_LEN: usize = 10;

/// Hex characters kept from the digest in a deterministic event id.
const ID_DIGEST_LEN: usize = 26;

/// Hex characters kept from the digest when a file stem has no slug.
const SLUG_FALLBACK_LEN: usize = 8;

/// Capture every markdown artifact under `<root>/.wai/projects/*/{kind}/`.
///
/// `project` is the kaya project namespace recorded in each event's identity;
/// the wai tree may hold more than one source project and all are traversed.
/// Subject ids are prefixed with the wai source project directory so the same
/// kind and file stem in two source projects never collapses into one node.
/// A missing `.wai` tree or missing kind directory yields no events. Every
/// emitted event is sorted by id so replaying the same tree produces the
/// same sequence. Nothing under `root` is ever written.
pub fn capture(root: &Path, project: &str) -> Result<Vec<Event>, AdapterError> {
    let mut events = Vec::new();
    let projects_dir = root.join(".wai").join("projects");
    let Some(project_dirs) = read_dir_sorted(&projects_dir)? else {
        return Ok(events);
    };

    for project_dir in project_dirs {
        let metadata = fs::metadata(&project_dir)
            .map_err(|err| io_failure(&project_dir, "inspect project directory", err))?;
        if !metadata.is_dir() {
            continue;
        }
        let Some(source_project) = project_dir.file_name().and_then(|name| name.to_str()) else {
            return Err(io_failure(
                &project_dir,
                "read project directory name",
                "is not UTF-8",
            ));
        };
        for kind in KINDS {
            let Some(artifacts) = read_dir_sorted(&project_dir.join(kind))? else {
                continue;
            };
            for artifact in artifacts {
                if artifact.extension().and_then(|ext| ext.to_str()) != Some("md") {
                    continue;
                }
                let metadata = fs::metadata(&artifact)
                    .map_err(|err| io_failure(&artifact, "inspect artifact", err))?;
                if !metadata.is_file() {
                    continue;
                }
                events.push(capture_file(
                    root,
                    source_project,
                    project,
                    kind,
                    &artifact,
                )?);
            }
        }
    }

    events.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(events)
}

/// Translate one markdown artifact into an observation event.
fn capture_file(
    root: &Path,
    source_project: &str,
    project: &str,
    kind: &str,
    path: &Path,
) -> Result<Event, AdapterError> {
    let relative = relative_path(root, path)?;

    // Reading proves the record is both accessible and valid UTF-8; the
    // contents stay wai's truth and are never copied into the event.
    fs::read_to_string(path)
        .map_err(|err| record_failure(&relative, format!("read record: {err}")))?;

    let timestamp = timestamp_for(path, &relative)?;
    let data = json!({ "kind": kind, "path": relative.clone() });
    let event = Event {
        id: event_id(project, &timestamp, &relative, &data),
        schema: SCHEMA_V0.to_owned(),
        r#type: EVENT_TYPE.to_owned(),
        timestamp,
        actor: Actor {
            r#type: ActorType::Tool,
            name: TOOL.to_owned(),
        },
        subject: Subject {
            r#type: "Artifact".to_owned(),
            id: format!(
                "{source_project}/Artifact/{kind}-{}",
                slug_for(path, &relative)
            ),
        },
        data,
        provenance: Provenance {
            tool: TOOL.to_owned(),
            version: SOURCE_VERSION.to_owned(),
            reference: relative.clone(),
        },
        artifacts: vec![relative],
    };

    // Refuse to emit a record the ledger would later reject.
    let line = event::to_line(&event).map_err(|err| {
        record_failure(
            &event.provenance.reference,
            format!("generated event could not be written: {err}"),
        )
    })?;
    event::parse_line(&line).map_err(|err| {
        record_failure(
            &event.provenance.reference,
            format!("generated event rejected: {err}"),
        )
    })
}

/// List a directory's entries as sorted paths; missing directories are `None`.
fn read_dir_sorted(dir: &Path) -> Result<Option<Vec<PathBuf>>, AdapterError> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(io_failure(dir, "read directory", err)),
    };

    let mut paths = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|err| io_failure(dir, "read directory entry", err))?;
        paths.push(entry.path());
    }
    paths.sort();
    Ok(Some(paths))
}

/// Render `path` relative to `root` with forward slashes.
fn relative_path(root: &Path, path: &Path) -> Result<String, AdapterError> {
    let stripped = path.strip_prefix(root).map_err(|_| {
        AdapterError::new(
            LABEL_FAILURE_ATOMIC,
            format!("wai: record {} is outside the adapter root", path.display()),
        )
    })?;

    let mut relative = String::new();
    for component in stripped.components() {
        let part = component.as_os_str().to_str().ok_or_else(|| {
            AdapterError::new(
                LABEL_FAILURE_ATOMIC,
                format!("wai: record {} has a non-UTF-8 path", path.display()),
            )
        })?;
        if !relative.is_empty() {
            relative.push('/');
        }
        relative.push_str(part);
    }
    Ok(relative)
}

/// Timestamp from a leading file-name date, else the file's modification time.
fn timestamp_for(path: &Path, relative: &str) -> Result<String, AdapterError> {
    if let Some(date) = leading_date(path) {
        return Ok(format!("{date}T00:00:00Z"));
    }

    let modified = fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .map_err(|err| record_failure(relative, format!("modified time unavailable: {err}")))?;
    Ok(DateTime::<Utc>::from(modified).to_rfc3339_opts(SecondsFormat::Secs, true))
}

/// The leading `YYYY-MM-DD` of a file name, when it is a real calendar date.
fn leading_date(path: &Path) -> Option<&str> {
    let name = path.file_name()?.to_str()?;
    let bytes = name.as_bytes();
    if bytes.len() < DATE_LEN {
        return None;
    }
    for (index, byte) in bytes[..DATE_LEN].iter().enumerate() {
        let separator = index == 4 || index == 7;
        let well_formed = if separator {
            *byte == b'-'
        } else {
            byte.is_ascii_digit()
        };
        if !well_formed {
            return None;
        }
    }
    let candidate = &name[..DATE_LEN];
    NaiveDate::parse_from_str(candidate, "%Y-%m-%d")
        .ok()
        .map(|_| candidate)
}

/// Lowercase alphanumeric runs joined by single dashes; digest prefix if empty.
fn slug_for(path: &Path, relative: &str) -> String {
    let stem = path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    let mut slug = String::new();
    let mut separator = false;
    for character in stem.to_lowercase().chars() {
        if character.is_alphanumeric() {
            if separator && !slug.is_empty() {
                slug.push('-');
            }
            separator = false;
            slug.push(character);
        } else {
            separator = true;
        }
    }

    if slug.is_empty() {
        return sha256_hex(relative.as_bytes())[..SLUG_FALLBACK_LEN].to_owned();
    }
    slug
}

/// Deterministic event id over every field the event writes.
///
/// Tool, tool version, project namespace, timestamp, source reference, and the
/// serialized payload all contribute, so two captures from different
/// environments never share an id while a replay of the same environment does.
fn event_id(project: &str, timestamp: &str, reference: &str, data: &serde_json::Value) -> String {
    let identity =
        format!("{TOOL}\x1f{SOURCE_VERSION}\x1f{project}\x1f{timestamp}\x1f{reference}\x1f{data}");
    let digest = sha256_hex(identity.as_bytes());
    format!("evt-{}", &digest[..ID_DIGEST_LEN])
}

fn io_failure(path: &Path, action: &str, err: impl std::fmt::Display) -> AdapterError {
    record_failure(&path.display().to_string(), format!("{action}: {err}"))
}

fn record_failure(record: &str, detail: impl std::fmt::Display) -> AdapterError {
    AdapterError::new(
        LABEL_FAILURE_ATOMIC,
        format!("wai: record {record}: {detail}"),
    )
}
