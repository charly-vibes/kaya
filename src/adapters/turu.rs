//! turu adapter: turu whisper bundle files to `capture.entry.observed`.
//!
//! Purpose: record that a turu whisper bundle entry exists, without copying
//! or interpreting its text. Responsibilities: read one bundle file exactly
//! once, translate each `entries[i]` into a deterministic event, and abort
//! atomically on any read or validation failure so a bad bundle contributes
//! no events. Rationale: turu owns the whisper payload; kaya only needs a
//! stable, replayable observation naming the bundle digest and entry. The
//! bundle's `freeform` section is turu's own extension data, out of this
//! contract: it is never read and never emitted.

use std::path::Path;

use chrono::{DateTime, SecondsFormat, Utc};
use serde_json::json;

use crate::event::{self, Actor, ActorType, Event, Provenance, SCHEMA_V0, Subject};

use super::{AdapterError, LABEL_FAILURE_ATOMIC, sha256_hex};

/// Source tool named in every event's provenance.
pub const TOOL: &str = "turu";

/// Version of the turu bundle layout this adapter reads.
pub const SOURCE_VERSION: &str = "0.11.2";

/// Event type emitted once per bundle entry.
const EVENT_TYPE: &str = "capture.entry.observed";

/// Hex characters kept from the digest in a deterministic event id.
const ID_DIGEST_LEN: usize = 26;

/// Translate a turu whisper bundle into observation events.
///
/// `project` is the kaya project namespace recorded in each event's identity.
/// The bundle is read exactly once and parsed fully before any event is
/// emitted, so every failure is atomic: a bundle that cannot be read, is not
/// UTF-8, is not valid JSON, or holds an entry missing a string `id`, `ts`,
/// or `topic` — or a non-RFC-3339 `ts` — contributes no events and is
/// reported as a `kaya.adapters.failure_atomic` refusal naming the bundle
/// and the offending entry. Entry text is never copied into an event. Each
/// entry becomes one event whose subject is
/// `<project>/Source/entry-<entry id>`; events are sorted by id so
/// replaying the same bundle produces the same sequence. Nothing beside the
/// bundle file is read and nothing is ever written.
pub fn capture(bundle_path: &Path, project: &str) -> Result<Vec<Event>, AdapterError> {
    let bundle_name = bundle_path.display().to_string();

    // One read serves both the digest and the parse; the bytes stay turu's.
    let bytes = std::fs::read(bundle_path)
        .map_err(|err| bundle_failure(&bundle_name, format!("read bundle: {err}")))?;
    let digest = sha256_hex(&bytes);
    let text = String::from_utf8(bytes)
        .map_err(|err| bundle_failure(&bundle_name, format!("bundle is not UTF-8: {err}")))?;
    let bundle: serde_json::Value = serde_json::from_str(&text)
        .map_err(|err| bundle_failure(&bundle_name, format!("bundle is not valid JSON: {err}")))?;
    let repo_key = bundle
        .get("repo_key")
        .and_then(|value| value.as_str())
        .ok_or_else(|| bundle_failure(&bundle_name, "missing string repo_key"))?;
    let scope = bundle
        .get("scope")
        .and_then(|value| value.as_str())
        .ok_or_else(|| bundle_failure(&bundle_name, "missing string scope"))?;
    let entries = bundle
        .get("entries")
        .and_then(|value| value.as_array())
        .ok_or_else(|| bundle_failure(&bundle_name, "missing entries array"))?;

    // The ids the bundle itself declares, for naming an entry that cannot
    // name itself (a missing or non-string id) in an atomic refusal.
    let named: Vec<&str> = entries
        .iter()
        .filter_map(|entry| entry.get("id").and_then(|value| value.as_str()))
        .collect();
    let named = named.join(", ");

    // Parse fully before emitting: a single malformed entry refuses the run.
    let mut events = Vec::with_capacity(entries.len());
    for entry in entries {
        events.push(capture_entry(
            &bundle_name,
            entry,
            project,
            repo_key,
            scope,
            &digest,
            &named,
        )?);
    }

    events.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(events)
}

/// Translate one bundle entry into an observation event.
fn capture_entry(
    bundle_name: &str,
    entry: &serde_json::Value,
    project: &str,
    repo_key: &str,
    scope: &str,
    digest: &str,
    named: &str,
) -> Result<Event, AdapterError> {
    let entry_id = string_field(bundle_name, entry, "id", named)?;
    let topic = string_field(bundle_name, entry, "topic", named)?;
    let ts = string_field(bundle_name, entry, "ts", named)?;

    // Normalize to a UTC second-resolution RFC 3339 timestamp so the same
    // instant spelled differently still replays to the same event id.
    let timestamp = DateTime::parse_from_rfc3339(&ts)
        .map_err(|err| {
            entry_failure(
                bundle_name,
                &entry_id,
                format!("timestamp {ts:?} is not RFC 3339: {err}"),
            )
        })?
        .with_timezone(&Utc)
        .to_rfc3339_opts(SecondsFormat::Secs, true);

    let data = json!({
        "entry": entry_id,
        "topic": topic,
        "scope": scope,
        "repo_key": repo_key,
        "bundle_digest": digest,
    });
    let event = Event {
        id: event_id(project, &timestamp, digest, &data),
        schema: SCHEMA_V0.to_owned(),
        r#type: EVENT_TYPE.to_owned(),
        timestamp,
        actor: Actor {
            r#type: ActorType::Tool,
            name: TOOL.to_owned(),
        },
        subject: Subject {
            r#type: "Source".to_owned(),
            id: format!("{project}/Source/entry-{entry_id}"),
        },
        data,
        provenance: Provenance {
            tool: TOOL.to_owned(),
            version: SOURCE_VERSION.to_owned(),
            reference: digest.to_owned(),
        },
        artifacts: vec![format!("sha256:{digest}")],
    };

    // Refuse to emit a record the ledger would later reject.
    let line = event::to_line(&event).map_err(|err| {
        entry_failure(
            bundle_name,
            &entry_id,
            format!("generated event could not be written: {err}"),
        )
    })?;
    event::parse_line(&line).map_err(|err| {
        entry_failure(
            bundle_name,
            &entry_id,
            format!("generated event rejected: {err}"),
        )
    })
}

/// A bundle entry's named string field, refusing absence or a non-string.
fn string_field(
    bundle_name: &str,
    entry: &serde_json::Value,
    field: &str,
    named: &str,
) -> Result<String, AdapterError> {
    let value = entry
        .get(field)
        .and_then(|value| value.as_str())
        .ok_or_else(|| {
            let self_named = entry
                .get("id")
                .and_then(|value| value.as_str())
                .unwrap_or("unnamed");
            entry_failure(
                bundle_name,
                self_named,
                format!("missing string {field} (entries named by the bundle: {named})"),
            )
        })?;
    Ok(value.to_owned())
}

/// A labeled atomic refusal naming the source and the bundle.
fn bundle_failure(bundle_name: &str, detail: impl std::fmt::Display) -> AdapterError {
    AdapterError::new(
        LABEL_FAILURE_ATOMIC,
        format!("turu: bundle {bundle_name}: {detail}"),
    )
}

/// A labeled atomic refusal naming the source, bundle, and offending entry.
fn entry_failure(
    bundle_name: &str,
    entry_id: &str,
    detail: impl std::fmt::Display,
) -> AdapterError {
    AdapterError::new(
        LABEL_FAILURE_ATOMIC,
        format!("turu: bundle {bundle_name}: entry {entry_id}: {detail}"),
    )
}

/// Deterministic event id over every field the event writes.
///
/// Tool, tool version, project namespace, timestamp, bundle digest, and the
/// serialized payload all contribute, so two captures from different
/// environments never share an id while a replay of the same bundle does.
fn event_id(project: &str, timestamp: &str, reference: &str, data: &serde_json::Value) -> String {
    let identity =
        format!("{TOOL}\x1f{SOURCE_VERSION}\x1f{project}\x1f{timestamp}\x1f{reference}\x1f{data}");
    let digest = sha256_hex(identity.as_bytes());
    format!("evt-{}", &digest[..ID_DIGEST_LEN])
}
