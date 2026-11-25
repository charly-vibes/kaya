//! Source adapters translating owned records into `kaya.event/v0` events.
//!
//! Purpose: one registration and one labeled refusal type shared by every
//! capture adapter, so each source names the contract it consumes.
//! Responsibilities: keep the adapter registry, expose the shared
//! [`AdapterError`] shape, and re-export the content digest helper.
//! Rationale: a closed registry makes "which contracts does kaya consume?"
//! answerable in one place, and a labeled error keeps a failed run
//! attributable to a source and record instead of a bare I/O message.

pub mod git;
pub mod testrunner;
pub mod turu;
pub mod wai;

use crate::event::Event;
pub use crate::ledger::sha256_hex;

/// Digest characters retained in a correction id, mirroring the adapters' id length.
const CORRECTION_ID_LEN: usize = 26;

/// Derive a deterministic correction for an id-collision refusal.
///
/// The correction is deterministic per (artifact identity, generation,
/// content-derived data state): replays of a corrected tree produce
/// byte-identical duplicates, so the ledger's id-reuse refusal collapses
/// into an idempotent re-append. The generation salt is revisit-aware:
/// returning to an earlier content state mints a fresh correction id
/// instead of colliding with the correction already stored for it. This
/// carries the ledger's correction guidance into the import path without
/// touching stored lines.
pub fn reobservation_correction(event: &Event, generation: usize) -> Event {
    let mut correction = event.clone();
    correction.data["supersedes"] = serde_json::Value::String(event.id.clone());
    let data = serde_json::to_string(&correction.data)
        .expect("event data is already valid JSON and serializes losslessly");
    let digest =
        sha256_hex(format!("correction\x1f{}\x1f{generation}\x1f{data}", event.id).as_bytes());
    correction.id = format!("evt-{}", &digest[..CORRECTION_ID_LEN]);
    correction
}

/// Whether a stored correction already supersedes `event`'s id.
///
/// True when any stored event names `event.id` in its `supersedes` field;
/// used to decide whether a `Duplicate` replay must re-correct.
pub fn stored_correction_for(stored: &[Event], event: &Event) -> bool {
    stored.iter().any(|stored_event| {
        stored_event.data.get("supersedes") == Some(&serde_json::Value::String(event.id.clone()))
    })
}

/// Count stored corrections of `event` whose payload differs from its
/// current state.
///
/// Each differing correction raises the generation salt, so a revisit of an
/// earlier content state mints exactly one fresh correction id; convergent
/// replays of the current state count zero and hit the already-stored id
/// byte-identically.
pub fn correction_generation(stored: &[Event], event: &Event) -> usize {
    stored
        .iter()
        .filter(|stored_event| {
            if stored_event.data.get("supersedes")
                != Some(&serde_json::Value::String(event.id.clone()))
            {
                return false;
            }
            let mut payload = stored_event.data.clone();
            payload
                .as_object_mut()
                .expect("event data is a JSON object")
                .remove("supersedes");
            payload != event.data
        })
        .count()
}

/// The content-stable identity of a capture observation, if it has one.
///
/// Some only when the event's data carries a string `path` (the shape wai
/// emits); the identity joins the actor name, subject id, data `kind`, and
/// data `path`, so re-observations of the same artifact match regardless
/// of the id their capture timestamp minted. Other event shapes have no
/// stable identity and stay untouched by the import dedup path.
pub fn stable_identity(event: &Event) -> Option<String> {
    let path = event.data.get("path")?;
    let path = path.as_str()?;
    let kind = event.data["kind"].to_string();
    Some(format!(
        "{}\x1f{}\x1f{}\x1f{}",
        event.actor.name, event.subject.id, kind, path
    ))
}

/// Whether two observations report the same facts, ignoring correction links.
///
/// Compares each event's data with the `supersedes` key removed, so a
/// correction and its target compare equal when nothing else changed.
pub fn same_observation(stored: &Event, event: &Event) -> bool {
    let strip = |event: &Event| {
        let mut data = event.data.clone();
        data.as_object_mut()
            .expect("event data is a JSON object")
            .remove("supersedes");
        data
    };
    strip(stored) == strip(event)
}

/// The most recent stored event with the same stable identity as `event`.
///
/// Scans from the end, so the returned observation is the ledger's newest
/// view of the artifact.
pub fn latest_stored_same_identity<'a>(stored: &'a [Event], event: &Event) -> Option<&'a Event> {
    let identity = stable_identity(event)?;
    stored
        .iter()
        .rev()
        .find(|stored_event| stable_identity(stored_event).as_deref() == Some(identity.as_str()))
}

/// The root (uncorrected) stored observation for `event`'s identity.
///
/// The first same-identity stored event whose data carries no `supersedes`
/// key; corrections name this root id, keeping the supersede chain rooted
/// at the original observation.
pub fn root_observation<'a>(stored: &'a [Event], event: &Event) -> Option<&'a Event> {
    let identity = stable_identity(event)?;
    stored.iter().find(|stored_event| {
        stable_identity(stored_event).as_deref() == Some(identity.as_str())
            && stored_event.data.get("supersedes").is_none()
    })
}

/// Failure class for an adapter run that must contribute no events.
pub(crate) const LABEL_FAILURE_ATOMIC: &str = "kaya.adapters.failure_atomic";

/// One source-owned contract an adapter consumes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AdapterRegistration {
    /// Adapter name, matching the module that implements it.
    pub name: &'static str,
    /// Source-owned store or format the adapter reads.
    pub source_contract: &'static str,
}

/// Adapters wired into this build, in stable order.
pub const REGISTRY: &[AdapterRegistration] = &[
    AdapterRegistration {
        name: "wai",
        source_contract: "`.wai/` artifact tree",
    },
    AdapterRegistration {
        name: "git",
        source_contract: "repository history (commits and tags)",
    },
    AdapterRegistration {
        name: "test-runner",
        source_contract: "machine-readable run output (libtest JSONL)",
    },
    AdapterRegistration {
        name: "turu",
        source_contract: "bundle per whisper's shared-store contract (turu bundle pack, keyed by repo key)",
    },
];

/// A labeled refusal raised while translating a source record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdapterError {
    /// Failure class, e.g. `kaya.adapters.failure_atomic`.
    pub label: String,
    /// Explanation naming the source and the offending record.
    pub message: String,
}

impl AdapterError {
    /// Build a labeled refusal.
    pub fn new(label: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            message: message.into(),
        }
    }
}

impl std::fmt::Display for AdapterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.label, self.message)
    }
}

impl std::error::Error for AdapterError {}
