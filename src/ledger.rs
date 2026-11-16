//! Append-only JSONL ledger over `kaya.event/v0` records.
//!
//! Purpose: preserve admitted events as immutable evidence every projection
//! derives from.
//! Responsibilities: crash-tail recovery, idempotent append, id-reuse
//! refusal, digest-anchored reads, and supersede-aware resolution.
//! Rationale: one plain file of lines under version control needs no
//! database, stays diffable, and — because written bytes are never rewritten
//! — a correction must be a new event naming the superseded id.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::event::{self, Event};

const LABEL_MISSING: &str = "kaya.ledger.missing";
const LABEL_IO: &str = "kaya.ledger.io";
const LABEL_RECORD_INVALID: &str = "kaya.ledger.record_invalid";
const LABEL_ID_REUSE: &str = "kaya.ledger.id_reuse";
const LABEL_DIGEST: &str = "kaya.ledger.digest_mismatch";

/// A labeled refusal raised while reading from or appending to the ledger.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LedgerError {
    /// Failure class, e.g. `kaya.ledger.digest_mismatch`.
    pub label: String,
    /// Explanation naming the offending value and the rule it broke.
    pub message: String,
}

impl LedgerError {
    /// Build a labeled refusal.
    pub fn new(label: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for LedgerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.label, self.message)
    }
}

impl std::error::Error for LedgerError {}

/// The append-only research ledger, addressed by its file path.
#[derive(Clone, Debug)]
pub struct Ledger {
    path: PathBuf,
}

impl Ledger {
    /// Address a ledger at `path` without requiring the file to exist yet.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Open an existing ledger, refusing a missing file.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, LedgerError> {
        let path = path.into();
        match File::open(&path) {
            Ok(_) => Ok(Self { path }),
            Err(err) if err.kind() == ErrorKind::NotFound => Err(missing_error(&path)),
            Err(err) => Err(io_error(&path, "open", err)),
        }
    }

    /// Path of the ledger file.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Read every stored event, repairing a truncated final line first.
    ///
    /// A crash can leave at most one partial final line; because the ledger
    /// is append-only, dropping that unwritten tail is the only legal
    /// recovery. The number of bytes removed is reported.
    pub fn read_recovered(&self) -> Result<(Vec<Event>, Recovery), LedgerError> {
        let loaded = self.load()?;
        Ok((loaded.events, loaded.recovery))
    }

    /// Read every stored event, then verify the ledger against `anchor`.
    ///
    /// A mismatch refuses the whole read and names the first differing
    /// 1-based line; projections never serve from unverifiable history.
    pub fn read_verified(&self, anchor: Option<&Anchor>) -> Result<Vec<Event>, LedgerError> {
        let loaded = self.load()?;
        if let Some(expected) = anchor {
            let actual = Anchor::of(&loaded.bytes);
            if &actual != expected {
                let line = first_differing_line(expected, &actual);
                return Err(LedgerError::new(
                    LABEL_DIGEST,
                    format!(
                        "ledger {} does not match its anchor: line {line} differs (anchor digest {}, current digest {}); refusing to serve from unverifiable history",
                        self.path.display(),
                        expected.digest,
                        actual.digest
                    ),
                ));
            }
        }
        Ok(loaded.events)
    }

    /// Append one event unless it is already stored byte-identically.
    ///
    /// A byte-identical replay is a no-op acknowledged as a duplicate; the
    /// same id with different bytes is refused, because ledger ids are never
    /// reused. The write is one `write_all` of the whole line.
    pub fn append(&mut self, event: &Event) -> Result<AppendOutcome, LedgerError> {
        let line = event::to_line(event);
        if line.is_empty() {
            return Err(LedgerError::new(
                LABEL_RECORD_INVALID,
                format!(
                    "event \"{}\" could not be serialized; refusing to append a partial record",
                    event.id
                ),
            ));
        }
        event::parse_line(&line).map_err(|err| {
            LedgerError::new(
                err.label,
                format!("refusing to append event \"{}\": {}", event.id, err.message),
            )
        })?;

        let loaded = self.load_optional()?;
        for existing in &loaded.line_bytes {
            if existing.as_slice() == line.as_bytes() {
                return Ok(AppendOutcome::Duplicate);
            }
        }
        if let Some(existing) = loaded.events.iter().find(|stored| stored.id == event.id) {
            return Err(LedgerError::new(
                LABEL_ID_REUSE,
                format!(
                    "event id \"{}\" already exists in the ledger with different bytes; ids are never reused — append a correction event naming \"{}\" in data.supersedes",
                    existing.id, existing.id
                ),
            ));
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|err| io_error(&self.path, "append to", err))?;
        file.write_all(line.as_bytes())
            .map_err(|err| io_error(&self.path, "append to", err))?;
        file.flush()
            .map_err(|err| io_error(&self.path, "flush", err))?;
        Ok(AppendOutcome::Appended)
    }

    fn load(&self) -> Result<Loaded, LedgerError> {
        self.load_inner(false)
    }

    fn load_optional(&self) -> Result<Loaded, LedgerError> {
        self.load_inner(true)
    }

    fn load_inner(&self, allow_missing: bool) -> Result<Loaded, LedgerError> {
        let mut bytes = match fs::read(&self.path) {
            Ok(bytes) => bytes,
            Err(err) if allow_missing && err.kind() == ErrorKind::NotFound => {
                return Ok(Loaded::empty());
            }
            Err(err) if err.kind() == ErrorKind::NotFound => return Err(missing_error(&self.path)),
            Err(err) => return Err(io_error(&self.path, "read", err)),
        };
        let recovery = self.recover(&mut bytes)?;
        let line_bytes = split_lines(&bytes);
        let mut events = Vec::with_capacity(line_bytes.len());
        let mut first_line: HashMap<String, usize> = HashMap::with_capacity(line_bytes.len());
        for (index, raw) in line_bytes.iter().enumerate() {
            let text = std::str::from_utf8(raw).map_err(|err| {
                LedgerError::new(
                    LABEL_RECORD_INVALID,
                    format!("ledger line {} is not UTF-8: {err}", index + 1),
                )
            })?;
            let event = event::parse_line(text).map_err(|err| {
                LedgerError::new(
                    err.label,
                    format!("ledger line {}: {}", index + 1, err.message),
                )
            })?;
            match first_line.get(&event.id) {
                Some(previous) => {
                    if line_bytes[*previous].as_slice() != raw.as_slice() {
                        return Err(LedgerError::new(
                            LABEL_ID_REUSE,
                            format!(
                                "event id \"{}\" appears on ledger lines {} and {} with different bytes; ids are never reused — record a correction naming the id in data.supersedes",
                                event.id,
                                previous + 1,
                                index + 1
                            ),
                        ));
                    }
                }
                None => {
                    first_line.insert(event.id.clone(), index);
                }
            }
            events.push(event);
        }
        Ok(Loaded {
            events,
            line_bytes,
            bytes,
            recovery,
        })
    }

    /// Drop a crash-truncated final line, on disk as well as in memory.
    fn recover(&self, bytes: &mut Vec<u8>) -> Result<Recovery, LedgerError> {
        if bytes.is_empty() || bytes.ends_with(b"\n") {
            return Ok(Recovery { truncated_bytes: 0 });
        }
        let keep = bytes
            .iter()
            .rposition(|byte| *byte == b'\n')
            .map_or(0, |index| index + 1);
        let truncated_bytes = bytes.len() - keep;
        bytes.truncate(keep);
        let file = OpenOptions::new()
            .write(true)
            .open(&self.path)
            .map_err(|err| io_error(&self.path, "truncate the crash tail of", err))?;
        file.set_len(keep as u64)
            .map_err(|err| io_error(&self.path, "truncate the crash tail of", err))?;
        Ok(Recovery { truncated_bytes })
    }
}

/// Result of attempting to append one event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppendOutcome {
    /// The line was written to the ledger.
    Appended,
    /// The identical line was already stored; nothing was written.
    Duplicate,
}

/// Report of a read-time crash-tail repair.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Recovery {
    /// Bytes removed from a truncated final line; zero means no repair.
    pub truncated_bytes: usize,
}

/// SHA-256 of `bytes` as lowercase hex.
pub fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// Content digest binding a ledger read to the bytes it verified.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Anchor {
    /// SHA-256 over the whole ledger byte sequence.
    pub digest: String,
    /// Ordered SHA-256 of each line, newline included.
    pub line_digests: Vec<String>,
}

impl Anchor {
    /// Compute the anchor of an exact ledger byte sequence.
    pub fn of(bytes: &[u8]) -> Self {
        let mut line_digests = Vec::new();
        let mut start = 0;
        for (index, byte) in bytes.iter().enumerate() {
            if *byte == b'\n' {
                line_digests.push(sha256_hex(&bytes[start..=index]));
                start = index + 1;
            }
        }
        if start < bytes.len() {
            line_digests.push(sha256_hex(&bytes[start..]));
        }
        Self {
            digest: sha256_hex(bytes),
            line_digests,
        }
    }
}

/// Drop events superseded by a later event's `data.supersedes`.
///
/// Corrections are appended, never edited, so readers resolve the record to
/// its correction while the superseded line remains in the ledger as
/// evidence.
pub fn resolve(events: &[Event]) -> Vec<&Event> {
    let mut superseded: HashSet<String> = HashSet::new();
    let mut dropped = vec![false; events.len()];
    for (index, event) in events.iter().enumerate().rev() {
        if superseded.contains(&event.id) {
            dropped[index] = true;
        }
        for target in supersede_targets(&event.data) {
            superseded.insert(target.to_owned());
        }
    }
    events
        .iter()
        .enumerate()
        .filter(|(index, _)| !dropped[*index])
        .map(|(_, event)| event)
        .collect()
}

fn supersede_targets(data: &serde_json::Value) -> Vec<&str> {
    match data.get("supersedes") {
        Some(serde_json::Value::String(target)) => vec![target.as_str()],
        Some(serde_json::Value::Array(targets)) => targets
            .iter()
            .filter_map(|target| target.as_str())
            .collect(),
        _ => Vec::new(),
    }
}

/// Split a recovered ledger into its stored lines, newline terminator included.
///
/// Recovery guarantees every remaining line ends with `\n`, so a split chunk
/// is the exact byte sequence that was appended — the unit dedupe compares.
fn split_lines(bytes: &[u8]) -> Vec<Vec<u8>> {
    let mut lines = Vec::new();
    let mut start = 0;
    for (index, byte) in bytes.iter().enumerate() {
        if *byte == b'\n' {
            lines.push(bytes[start..=index].to_vec());
            start = index + 1;
        }
    }
    lines
}

fn first_differing_line(expected: &Anchor, actual: &Anchor) -> usize {
    let shared = expected.line_digests.len().min(actual.line_digests.len());
    for index in 0..shared {
        if expected.line_digests[index] != actual.line_digests[index] {
            return index + 1;
        }
    }
    if expected.line_digests.len() == actual.line_digests.len() {
        1
    } else {
        shared + 1
    }
}

fn missing_error(path: &Path) -> LedgerError {
    LedgerError::new(
        LABEL_MISSING,
        format!(
            "ledger {} does not exist: run `kaya init` to create it",
            path.display()
        ),
    )
}

fn io_error(path: &Path, action: &str, err: std::io::Error) -> LedgerError {
    LedgerError::new(
        LABEL_IO,
        format!("failed to {action} ledger {}: {err}", path.display()),
    )
}

/// A loaded ledger: parsed events plus the exact recovered bytes.
struct Loaded {
    events: Vec<Event>,
    line_bytes: Vec<Vec<u8>>,
    bytes: Vec<u8>,
    recovery: Recovery,
}

impl Loaded {
    fn empty() -> Self {
        Self {
            events: Vec::new(),
            line_bytes: Vec::new(),
            bytes: Vec::new(),
            recovery: Recovery { truncated_bytes: 0 },
        }
    }
}
