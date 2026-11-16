//! Genesis-style CLI envelope.
//!
//! Every kaya invocation prints exactly one JSON object carrying `ok`,
//! version metadata, the command's `data` payload, and — on failure — a
//! labeled `error`. Ledger records use a separate schema (`kaya.event/v0`)
//! and are never CLI envelopes.

use std::io::Write as _;

use serde::Serialize;
use serde_json::{Map, Value};

/// Envelope schema version spoken by this CLI.
pub const ENVELOPE_VERSION: &str = "0.1";

/// Version of the kaya binary emitting the envelope.
pub const CLI_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Labeled failure carried by an error envelope.
#[derive(Debug, Serialize)]
pub struct EnvelopeError {
    pub label: String,
    pub message: String,
}

/// One CLI result, success or failure.
#[derive(Debug, Serialize)]
pub struct Envelope {
    pub ok: bool,
    pub envelope_version: &'static str,
    pub cli_version: &'static str,
    pub envelope_kind: &'static str,
    pub data: Value,
    pub warnings: Vec<String>,
    pub hints: Vec<String>,
    pub meta: Map<String, Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<EnvelopeError>,
}

impl Envelope {
    fn new(ok: bool, kind: &'static str, data: Value) -> Self {
        Self {
            ok,
            envelope_version: ENVELOPE_VERSION,
            cli_version: CLI_VERSION,
            envelope_kind: kind,
            data,
            warnings: Vec::new(),
            hints: Vec::new(),
            meta: Map::new(),
            error: None,
        }
    }

    /// Success envelope carrying `data` for a command of `kind`.
    pub fn ok(kind: &'static str, data: Value) -> Self {
        Self::new(true, kind, data)
    }

    /// Failure envelope: `kind` is the envelope kind (`"error"` on the CLI),
    /// `label` names the failure class, and `message` explains it.
    pub fn error(kind: &'static str, label: &str, message: impl Into<String>) -> Self {
        Self {
            error: Some(EnvelopeError {
                label: label.to_owned(),
                message: message.into(),
            }),
            ..Self::new(false, kind, Value::Null)
        }
    }

    /// Write the envelope as pretty JSON followed by one newline.
    pub fn print(&self) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|err| format!("failed to serialize envelope: {err}"))?;
        let mut stdout = std::io::stdout().lock();
        writeln!(stdout, "{json}").map_err(|err| format!("failed to write envelope: {err}"))
    }
}
