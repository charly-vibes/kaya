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

pub use crate::ledger::sha256_hex;

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
