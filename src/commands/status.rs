//! Ledger shape commands: bootstrap the ledger and report its state.
//!
//! Responsibilities: hold `cmd_init`, which creates the ledger unless a
//! non-ledger file shadows it, and `cmd_status`, the single streaming pass
//! that reports event count, size, digest, replay integrity, and pending
//! proposals, with its line-counting and proposal-capping helpers.
//!
//! Rationale: both commands read or create the ledger without replaying it,
//! so their helpers stay together and out of the replay-grade command
//! families.

use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Read};

use kaya::envelope::Envelope;
use kaya::event::{SCHEMA_V0, parse_line};
use kaya::ledger::sha256_hex;
use kaya::promotion::{PROPOSAL_BUDGET, PromotionState, cap_proposals};
use serde_json::json;

use crate::{Failure, KAYA_DIR, LEDGER_PATH, failure, promotion_failure};

/// Size of the single buffer streamed over the ledger by `kaya status`.
const STATUS_BUFFER_BYTES: usize = 64 * 1024;

pub(crate) fn cmd_init() -> Result<Envelope, Failure> {
    fs::create_dir_all(KAYA_DIR)
        .map_err(|err| failure("ledger-io", format!("failed to create {KAYA_DIR}: {err}")))?;
    let created = match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(LEDGER_PATH)
    {
        Ok(_) => true,
        Err(err) if err.kind() == ErrorKind::AlreadyExists => {
            ensure_ledger_is_regular_file()?;
            false
        }
        Err(err) => {
            return Err(failure(
                "ledger-io",
                format!("failed to create {LEDGER_PATH}: {err}"),
            ));
        }
    };
    Ok(Envelope::ok(
        "init",
        json!({ "ledger": LEDGER_PATH, "created": created }),
    ))
}

/// Reject a pre-existing path that shadows the ledger but is not a regular file.
fn ensure_ledger_is_regular_file() -> Result<(), Failure> {
    match fs::metadata(LEDGER_PATH) {
        Ok(metadata) if metadata.is_file() => Ok(()),
        Ok(_) => Err(not_a_file()),
        // A dangling symlink exists but resolves to no regular file.
        Err(err) if err.kind() == ErrorKind::NotFound => Err(not_a_file()),
        Err(err) => Err(failure(
            "ledger-io",
            format!("failed to inspect {LEDGER_PATH}: {err}"),
        )),
    }
}

fn not_a_file() -> Failure {
    failure(
        "not-a-file",
        format!(
            "{LEDGER_PATH} exists but is not a regular ledger file; \
             move it aside and rerun `kaya init`"
        ),
    )
}

pub(crate) fn cmd_status() -> Result<Envelope, Failure> {
    let mut file = File::open(LEDGER_PATH).map_err(|err| match err.kind() {
        ErrorKind::NotFound => failure(
            "missing-ledger",
            format!("missing ledger {LEDGER_PATH}: run `kaya init` to create it"),
        ),
        _ => failure("ledger-io", format!("failed to read {LEDGER_PATH}: {err}")),
    })?;

    // One streaming pass: size, and tail byte together; the digest is the
    // shared one-shot sha256_hex over the same bytes build and check use.
    let mut buffer = [0u8; STATUS_BUFFER_BYTES];
    let mut bytes: u64 = 0;
    let mut last_byte: Option<u8> = None;
    let mut content = Vec::new();
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|err| failure("ledger-io", format!("failed to read {LEDGER_PATH}: {err}")))?;
        if read == 0 {
            break;
        }
        let chunk = &buffer[..read];
        content.extend_from_slice(chunk);
        bytes += read as u64;
        last_byte = Some(chunk[read - 1]);
    }
    let digest = sha256_hex(&content);

    let (events, blank_lines) = line_counts(&content);

    let replay_integrity = match last_byte {
        None | Some(b'\n') => "ok",
        Some(_) => "recovered-truncated-tail",
    };
    Ok(Envelope::ok(
        "status",
        json!({
            "ledger": LEDGER_PATH,
            "events": events,
            "blank_lines": blank_lines,
            "bytes": bytes,
            "digest": digest,
            "replay_integrity": replay_integrity,
            "pending_proposals": pending_proposals()?,
        }),
    ))
}

/// Count ledger lines by recognition, not by newline.
///
/// Returns `(events, blank_lines)`: the number of non-empty lines that are
/// recognizable `kaya.event/v0` records — a JSON object naming the supported
/// schema, so a complete-but-unterminated final line counts while a
/// truncated or corrupt line does not — and the number of empty lines (under
/// the trailing-newline split convention, so a terminated file's final empty
/// split element is not a line). Lines that are neither count as neither;
/// status stays a cheap, non-failing, lenient read and replay commands
/// report the exact faults.
fn line_counts(bytes: &[u8]) -> (u64, u64) {
    let mut events: u64 = 0;
    let mut blank_lines: u64 = 0;
    if bytes.is_empty() {
        return (0, 0);
    }
    // Drop the split artifact left by a trailing newline; an unterminated
    // tail keeps its final (possibly complete) line.
    let body = bytes.strip_suffix(b"\n").unwrap_or(bytes);
    for line in body.split(|byte| *byte == b'\n') {
        if line.is_empty() {
            blank_lines += 1;
        } else if is_event_record(line) {
            events += 1;
        }
    }
    (events, blank_lines)
}

/// Recognize a ledger line structurally as a `kaya.event/v0` record.
///
/// Status counts by recognition, not replay-grade validation: a line counts
/// when it is a JSON object naming the supported schema, so a minimal but
/// well-formed record counts while a truncated tail, a corrupt line, or a
/// foreign-schema object does not. The full record grammar stays with the
/// replay commands, which can point at the offending line.
fn is_event_record(line: &[u8]) -> bool {
    let Ok(text) = std::str::from_utf8(line) else {
        return false;
    };
    let Ok(record) = serde_json::from_str::<serde_json::Value>(text.trim()) else {
        return false;
    };
    let Some(map) = record.as_object() else {
        return false;
    };
    map.get("schema").and_then(serde_json::Value::as_str) == Some(SCHEMA_V0)
}

/// Count assertions awaiting human approval, capped at `PROPOSAL_BUDGET`
/// (ranked and capped per `pending_proposals_bounded`, never surfaced whole).
///
/// Status stays a cheap, non-failing read: lines that do not parse as
/// `kaya.event/v0` records are skipped here and reported by the commands
/// that actually replay the ledger. Only when a well-formed promotion record
/// contradicts the promotion rules does status surface the refusal.
fn pending_proposals() -> Result<usize, Failure> {
    let bytes = match fs::read(LEDGER_PATH) {
        Ok(bytes) => bytes,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(0),
        Err(err) => {
            return Err(failure(
                "ledger-io",
                format!("failed to read {LEDGER_PATH}: {err}"),
            ));
        }
    };
    let mut events = Vec::new();
    for line in bytes.split(|byte| *byte == b'\n') {
        let Ok(text) = std::str::from_utf8(line) else {
            continue;
        };
        if let Ok(event) = parse_line(text) {
            events.push(event);
        }
    }
    PromotionState::from_events(&events)
        .map(|state| cap_proposals(&state.assertions, PROPOSAL_BUDGET).len())
        .map_err(promotion_failure)
}
