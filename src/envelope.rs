//! Thin kaya façade over `genesis::envelope` 0.15.
//!
//! Genesis owns the ok/kind consistency invariant, the envelope and CLI
//! version stamping, and the `data` payload. Kaya owns the 0.1 wire deltas
//! its consumers depend on: free-form command-name kinds (`init`, `impact`,
//! …) that genesis's closed `EnvelopeKind` cannot express, the separate
//! `error` object carrying `label`/`message`, and the always-present
//! `warnings`/`hints`/`meta` fields. The serialized shape is unchanged from
//! the pre-adoption envelope: `ok`, `envelope_version` (`"0.1"`),
//! `cli_version`, `envelope_kind`, `data`, `warnings`, `hints`, `meta`, and
//! `error` (omitted on success). Ledger records use a separate schema
//! (`kaya.event/v0`) and are never CLI envelopes.

use std::io::Write as _;

use genesis::envelope::{Envelope as GenesisEnvelope, EnvelopeKind};
use serde::ser::{Serialize, SerializeMap, Serializer};
use serde_json::{Map, Value};

/// Envelope schema version spoken by this CLI.
pub const ENVELOPE_VERSION: &str = "0.1";

/// Version of the kaya binary emitting the envelope.
pub const CLI_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Labeled failure carried by an error envelope.
#[derive(Debug, serde::Serialize)]
pub struct EnvelopeError {
    pub label: String,
    pub message: String,
}

/// One CLI result, success or failure.
#[derive(Debug)]
pub struct Envelope {
    /// Command-name kind stamped on the wire (genesis's closed enum cannot
    /// express kaya's per-command kinds, so it is carried alongside).
    kind: &'static str,
    /// Genesis envelope carrying `ok`, the version stamps, and `data`.
    inner: GenesisEnvelope<Value>,
    /// Kaya's 0.1 wire keeps the labeled failure as its own field; genesis
    /// routes errors through `data`, so it lives beside `inner`.
    pub error: Option<EnvelopeError>,
    pub warnings: Vec<String>,
    pub hints: Vec<String>,
    pub meta: Map<String, Value>,
}

impl Envelope {
    fn shell(kind: &'static str, envelope_kind: EnvelopeKind, data: Value) -> Self {
        Self {
            kind,
            inner: GenesisEnvelope::success(
                ENVELOPE_VERSION,
                CLI_VERSION,
                envelope_kind,
                data,
                vec![],
                vec![],
            ),
            error: None,
            warnings: Vec::new(),
            hints: Vec::new(),
            meta: Map::new(),
        }
    }

    /// Success envelope carrying `data` for a command of `kind`.
    pub fn ok(kind: &'static str, data: Value) -> Self {
        Self::shell(kind, EnvelopeKind::Ok, data)
    }

    /// Failure envelope: `kind` is the envelope kind (`"error"` on the CLI),
    /// `label` names the failure class, and `message` explains it.
    pub fn error(kind: &'static str, label: &str, message: impl Into<String>) -> Self {
        Self {
            error: Some(EnvelopeError {
                label: label.to_owned(),
                message: message.into(),
            }),
            ..Self::shell(kind, EnvelopeKind::Error, Value::Null)
        }
    }

    /// Wire `ok` value owned by the genesis inner envelope.
    pub fn is_ok(&self) -> bool {
        self.inner.ok
    }

    /// Write the envelope as pretty JSON followed by one newline.
    pub fn print(&self) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|err| format!("failed to serialize envelope: {err}"))?;
        let mut stdout = std::io::stdout().lock();
        writeln!(stdout, "{json}").map_err(|err| format!("failed to write envelope: {err}"))
    }
}

impl Serialize for Envelope {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry(&"ok", &self.inner.ok)?;
        map.serialize_entry(&"envelope_version", &self.inner.envelope_version)?;
        map.serialize_entry(&"cli_version", &self.inner.cli_version)?;
        map.serialize_entry(&"envelope_kind", self.kind)?;
        map.serialize_entry(&"data", &self.inner.data)?;
        map.serialize_entry(&"warnings", &self.warnings)?;
        map.serialize_entry(&"hints", &self.hints)?;
        map.serialize_entry(&"meta", &self.meta)?;
        if let Some(error) = &self.error {
            map.serialize_entry(&"error", error)?;
        }
        map.end()
    }
}
