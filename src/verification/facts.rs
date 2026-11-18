//! Record facts reported alongside a `kaya check` verdict.
//!
//! Purpose: describe the verified record in one place, so a report can answer
//! "what did this check actually read?" without re-parsing the ledger.
//! Responsibilities: count the verified events by canonical capture source,
//! record the sha256 of the exact verified ledger bytes, name the newest
//! recorded capture commit and the newest event timestamp, and list the
//! dotted `family.kind` names the projections emitted or omitted.
//! Rationale: every fact is derived from the same verified bytes the drift
//! comparison reads, so worktree and index modes cannot disagree, and a source
//! is classified by event type and provenance — never by the actor name —
//! because the actor records who wrote an event, not what observed it.

use std::collections::BTreeMap;
use std::path::PathBuf;

use chrono::{DateTime, FixedOffset};
use serde::Serialize;

use crate::event::Event;
use crate::projection::{self, BlockKind, ProjectionError};

/// The canonical `events_by_source` keys, in report order.
///
/// A capture family or provenance names the source; no other name, and in
/// particular no actor name, ever becomes a key.
pub const SOURCES: [&str; 4] = ["wai", "git", "test-runner", "model"];

/// The event type the git adapter records one commit as.
const CAPTURE_COMMIT: &str = "capture.commit.observed";

/// The event type the test-runner adapter records one run as.
const RUN_COMPLETED: &str = "experiment.run.completed";

/// The event type the wai adapter records one artifact as.
const CAPTURE_ARTIFACT: &str = "capture.artifact.observed";

/// The provenance tool that makes an artifact observation a wai source.
const WAI_TOOL: &str = "wai";

/// What one check observed in the verified record.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Facts {
    /// Verified events the ledger records.
    pub events: usize,
    /// sha256 of the exact verified ledger bytes.
    pub ledger_digest: String,
    /// Newest commit named by a `capture.commit.observed` event, as the record
    /// states it; `None` when no event names one. The commit is not required
    /// to be reachable: the observation is a fact about the record.
    pub last_capture_commit: Option<String>,
    /// Greatest event timestamp, or `None` for an empty record.
    pub newest_event_at: Option<String>,
    /// Event counts under the canonical sources, every source present.
    pub events_by_source: BTreeMap<String, usize>,
    /// Dotted `family.kind` names the research projections emit, sorted.
    pub emitted_kinds: Vec<String>,
    /// Dotted `family.kind` names a planner omitted for having no records,
    /// sorted.
    pub omitted_kinds: Vec<String>,
}

impl Facts {
    /// Derive the facts from the verified bytes and the planned projections.
    ///
    /// `documents` are the expected research documents, so scanning them names
    /// exactly the generated blocks a clean `build` leaves. `omitted` is the
    /// planner's no-records vocabulary, one dotted name per omission.
    pub fn derive(
        events: &[Event],
        ledger_digest: String,
        documents: &BTreeMap<PathBuf, String>,
        omitted: &[String],
    ) -> Result<Self, ProjectionError> {
        Ok(Self {
            events: events.len(),
            ledger_digest,
            last_capture_commit: last_capture_commit(events),
            newest_event_at: newest_event_at(events),
            events_by_source: events_by_source(events),
            emitted_kinds: emitted_kinds(documents)?,
            omitted_kinds: omitted.to_vec(),
        })
    }
}

/// Count events under the canonical sources, every source present even at 0.
fn events_by_source(events: &[Event]) -> BTreeMap<String, usize> {
    let mut counts: BTreeMap<String, usize> = SOURCES
        .iter()
        .map(|source| ((*source).to_owned(), 0))
        .collect();
    for event in events {
        if let Some(source) = source_of(event) {
            *counts.entry(source.to_owned()).or_insert(0) += 1;
        }
    }
    counts
}

/// The canonical source an event belongs to, from its type and provenance.
fn source_of(event: &Event) -> Option<&'static str> {
    match event.r#type.as_str() {
        CAPTURE_COMMIT => Some("git"),
        RUN_COMPLETED => Some("test-runner"),
        CAPTURE_ARTIFACT if event.provenance.tool == WAI_TOOL => Some("wai"),
        other if other.starts_with("model.") => Some("model"),
        _ => None,
    }
}

/// The newest commit a capture event names, ordered by parsed timestamp then
/// event id.
///
/// Only an event that names a string `data.sha` can contribute a frontier, so
/// an unnamed observation never hides an older named one. Timestamps are
/// compared as parsed RFC 3339 instants, because a ledger admits more than one
/// textual spelling of the same second. The sha is reported as recorded,
/// without a reachability check: a rewritten history never rewrites what the
/// record observed.
fn last_capture_commit(events: &[Event]) -> Option<String> {
    events
        .iter()
        .filter(|event| event.r#type == CAPTURE_COMMIT)
        .filter(|event| event.data.get("sha").is_some_and(|sha| sha.is_string()))
        .max_by(|left, right| {
            compare_timestamps(&left.timestamp, &right.timestamp)
                .then_with(|| left.id.cmp(&right.id))
        })
        .and_then(|event| event.data.get("sha"))
        .and_then(|sha| sha.as_str())
        .map(str::to_owned)
}

/// The greatest event timestamp, or `None` for an empty record.
fn newest_event_at(events: &[Event]) -> Option<String> {
    events
        .iter()
        .map(|event| event.timestamp.as_str())
        .max_by(|left, right| compare_timestamps(left, right))
        .map(str::to_owned)
}

/// The dotted `family.kind` names the planned documents carry, sorted.
///
/// Only generated blocks count, and a `:<namespace>` suffix is dropped so a
/// namespaced block still reports its closed vocabulary name.
fn emitted_kinds(documents: &BTreeMap<PathBuf, String>) -> Result<Vec<String>, ProjectionError> {
    let mut kinds = Vec::new();
    for document in documents.values() {
        for block in projection::scan(document)? {
            if let BlockKind::Generated(name) = block.kind {
                let base = name.split_once(':').map_or(name.as_str(), |(base, _)| base);
                kinds.push(base.to_owned());
            }
        }
    }
    kinds.sort();
    kinds.dedup();
    Ok(kinds)
}

/// Compare two RFC 3339 timestamps, falling back to lexical order.
///
/// A ledger timestamp is validated as RFC 3339 before it is admitted, so the
/// fallback only keeps an already-refused record from producing an ordering
/// error of its own.
fn compare_timestamps(left: &str, right: &str) -> std::cmp::Ordering {
    match (
        DateTime::<FixedOffset>::parse_from_rfc3339(left),
        DateTime::<FixedOffset>::parse_from_rfc3339(right),
    ) {
        (Ok(left), Ok(right)) => left.cmp(&right),
        _ => left.cmp(right),
    }
}
