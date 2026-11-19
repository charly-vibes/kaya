//! Append-only JSONL ledger over `kaya.event/v0` records.
//!
//! Purpose: preserve admitted events as immutable evidence every projection
//! derives from.
//! Responsibilities: crash-tail recovery, idempotent append, id-reuse
//! refusal, digest-anchored reads, and supersede-aware resolution.
//! Recovery preserves a complete final record by appending its missing
//! newline; only bytes that cannot parse as one event are truncated.
//! Rationale: one plain file of lines under version control needs no
//! database, stays diffable, and — because written bytes are never rewritten
//! — a correction must be a new event naming the superseded id.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs::{self, File, OpenOptions, TryLockError};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::event::{self, Event};

const LABEL_MISSING: &str = "kaya.ledger.missing";
const LABEL_IO: &str = "kaya.ledger.io";
const LABEL_RECORD_INVALID: &str = "kaya.ledger.record_invalid";
const LABEL_ID_REUSE: &str = "kaya.ledger.id_reuse";
const LABEL_DIGEST: &str = "kaya.ledger.digest_mismatch";
const LABEL_SUPERSEDES_INVALID: &str = "kaya.ledger.supersedes_invalid";
const LABEL_LOCKED: &str = "kaya.ledger.locked";

/// Exclusive advisory lock on the ledger file, released when dropped.
///
/// Every ledger critical section (load, append, recovery) holds one so a
/// concurrent reader never races a repair: contention is refused with
/// [`LABEL_LOCKED`] and the bytes stay untouched.
struct LockGuard(File);

impl Drop for LockGuard {
    fn drop(&mut self) {
        let _ = self.0.unlock();
    }
}

impl LockGuard {
    /// Open `path` and take its exclusive advisory lock.
    ///
    /// With `create`, a missing file is created first (the append path);
    /// otherwise a missing file surfaces as [`LABEL_MISSING`]. Contention
    /// refuses with [`LABEL_LOCKED`] instead of blocking.
    fn acquire(path: &Path, create: bool) -> Result<Self, LedgerError> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(create)
            .open(path)
            .map_err(|err| {
                if !create && err.kind() == ErrorKind::NotFound {
                    missing_error(path)
                } else {
                    io_error(path, "open", err)
                }
            })?;
        match file.try_lock() {
            Ok(()) => Ok(LockGuard(file)),
            Err(TryLockError::WouldBlock) => Err(LedgerError::new(
                LABEL_LOCKED,
                format!(
                    "ledger {} is locked; its bytes are held by another writer",
                    path.display()
                ),
            )),
            Err(TryLockError::Error(err)) => Err(io_error(path, "lock", err)),
        }
    }
}

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

    /// Read every stored event, repairing the final line first.
    ///
    /// A crash can leave at most one partial final line. If that line parses
    /// as a complete event, recovery appends the missing newline without
    /// touching preceding bytes; otherwise it truncates the partial line.
    /// Either repair is reported in the returned [`Recovery`]. This is the
    /// explicit unverified path; projections use [`Self::read_verified`].
    pub fn read_recovered(&self) -> Result<(Vec<Event>, Recovery), LedgerError> {
        let loaded = self.load()?;
        Ok((loaded.events, loaded.recovery))
    }

    /// Read every stored event, then verify the ledger against `anchor`.
    ///
    /// Recovery runs first so a crash tail is reported rather than mistaken
    /// for tampering. Verification then requires the anchored byte prefix to
    /// be unchanged and every byte after it to be a complete, valid line
    /// (growth is valid, mutation is not). A mismatch refuses the whole read
    /// and names the first differing 1-based line; projections never serve
    /// from unverifiable history.
    pub fn read_verified(&self, anchor: &Anchor) -> Result<(Vec<Event>, Recovery), LedgerError> {
        let loaded = self.load()?;
        self.verify(anchor, &loaded.bytes)?;
        Ok((loaded.events, loaded.recovery))
    }

    /// Refuse `bytes` unless they extend the anchored prefix without changing it.
    fn verify(&self, anchor: &Anchor, bytes: &[u8]) -> Result<(), LedgerError> {
        if bytes.len() < anchor.byte_len {
            let line = first_differing_line(anchor, &Anchor::of(bytes));
            return Err(self.digest_mismatch(anchor, bytes, line));
        }
        let prefix = &bytes[..anchor.byte_len];
        if sha256_hex(prefix) != anchor.digest {
            let line = first_differing_line(anchor, &Anchor::of(prefix));
            return Err(self.digest_mismatch(anchor, bytes, line));
        }
        if !prefix.is_empty() && !prefix.ends_with(b"\n") {
            // The anchor ends in the middle of a line: the prefix itself is
            // corrupt, so there is no complete line to attribute it to. Line
            // 1 is the conservative first-line report.
            return Err(self.digest_mismatch(anchor, bytes, 1));
        }
        if prefix.iter().filter(|byte| **byte == b'\n').count() != anchor.line_digests.len() {
            let line = first_differing_line(anchor, &Anchor::of(prefix));
            return Err(self.digest_mismatch(anchor, bytes, line));
        }
        let actual = Anchor::of(bytes);
        if !actual.line_digests.starts_with(&anchor.line_digests) {
            let line = first_differing_line(anchor, &actual);
            return Err(self.digest_mismatch(anchor, bytes, line));
        }
        if !bytes.is_empty() && !bytes.ends_with(b"\n") {
            // Recovery leaves only complete lines, so this is unreachable
            // unless recovery itself was bypassed; refuse rather than serve
            // a partial trailing record.
            let line = actual.line_digests.len() + 1;
            return Err(self.digest_mismatch(anchor, bytes, line));
        }
        Ok(())
    }

    fn digest_mismatch(&self, anchor: &Anchor, bytes: &[u8], line: usize) -> LedgerError {
        LedgerError::new(
            LABEL_DIGEST,
            format!(
                "ledger {} does not match its anchor: line {line} differs (anchor digest {} over {} bytes, current digest {} over {} bytes); refusing to serve from unverifiable history",
                self.path.display(),
                anchor.digest,
                anchor.byte_len,
                sha256_hex(bytes),
                bytes.len()
            ),
        )
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

        let pre_existing = self.path.exists();
        let lock = LockGuard::acquire(&self.path, true)?;
        let loaded = self.load_held(true, &lock)?;
        // A refusal before any write must leave a not-yet-existing ledger
        // absent, as it was before locking: a failed first append creates
        // nothing.
        let refuse = |err: LedgerError| -> Result<AppendOutcome, LedgerError> {
            if !pre_existing && loaded.line_bytes.is_empty() {
                let _ = fs::remove_file(&self.path);
            }
            Err(err)
        };
        for existing in &loaded.line_bytes {
            if existing.as_slice() == line.as_bytes() {
                return Ok(AppendOutcome::Duplicate);
            }
        }
        if let Some(existing) = loaded.events.iter().find(|stored| stored.id == event.id) {
            return refuse(LedgerError::new(
                LABEL_ID_REUSE,
                format!(
                    "event id \"{}\" already exists in the ledger with different bytes; ids are never reused — append a correction event naming \"{}\" in data.supersedes",
                    existing.id, existing.id
                ),
            ));
        }

        if let Err(err) = validate_supersedes(event, &loaded.events) {
            return refuse(err);
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

    fn load_inner(&self, allow_missing: bool) -> Result<Loaded, LedgerError> {
        let _lock = LockGuard::acquire(&self.path, false)?;
        self.load_held(allow_missing, &_lock)
    }

    /// Load under a lock already held on the ledger file.
    fn load_held(&self, allow_missing: bool, _lock: &LockGuard) -> Result<Loaded, LedgerError> {
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
                    return Err(LedgerError::new(
                        LABEL_ID_REUSE,
                        format!(
                            "event id \"{}\" appears on ledger lines {} and {}; ids are never reused — record a correction naming the id in data.supersedes",
                            event.id,
                            previous + 1,
                            index + 1
                        ),
                    ));
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

    /// Repair the final line, on disk as well as in memory.
    ///
    /// A complete event lacking only its newline is preserved: the newline is
    /// appended and no preceding byte changes. Only a tail that cannot parse
    /// as one event is truncated.
    fn recover(&self, bytes: &mut Vec<u8>) -> Result<Recovery, LedgerError> {
        if bytes.is_empty() || bytes.ends_with(b"\n") {
            return Ok(Recovery::default());
        }
        let line_start = bytes
            .iter()
            .rposition(|byte| *byte == b'\n')
            .map_or(0, |index| index + 1);
        let tail = &bytes[line_start..];
        let tail_len = tail.len();
        let complete = std::str::from_utf8(tail)
            .ok()
            .is_some_and(|text| event::parse_line(text).is_ok());
        if complete {
            let mut file = OpenOptions::new()
                .append(true)
                .open(&self.path)
                .map_err(|err| io_error(&self.path, "append a newline to", err))?;
            file.write_all(b"\n")
                .map_err(|err| io_error(&self.path, "append a newline to", err))?;
            file.flush()
                .map_err(|err| io_error(&self.path, "flush", err))?;
            bytes.push(b'\n');
            return Ok(Recovery {
                truncated_bytes: 0,
                appended_newline: true,
            });
        }
        bytes.truncate(line_start);
        let file = OpenOptions::new()
            .write(true)
            .open(&self.path)
            .map_err(|err| io_error(&self.path, "truncate the crash tail of", err))?;
        file.set_len(line_start as u64)
            .map_err(|err| io_error(&self.path, "truncate the crash tail of", err))?;
        Ok(Recovery {
            truncated_bytes: tail_len,
            appended_newline: false,
        })
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

/// Report of a read-time final-line repair.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Recovery {
    /// Bytes removed from a truncated final line; zero means no truncation.
    pub truncated_bytes: usize,
    /// Whether a complete final event lacked its newline and had one appended.
    pub appended_newline: bool,
}

/// SHA-256 of `bytes` as lowercase hex.
pub fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// Content digest binding a ledger read to the bytes it verified.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Anchor {
    /// SHA-256 over the anchored ledger byte sequence.
    pub digest: String,
    /// Number of anchored bytes; the ledger may grow beyond this prefix.
    pub byte_len: usize,
    /// Ordered SHA-256 of each newline-terminated line in the prefix.
    pub line_digests: Vec<String>,
}

impl Anchor {
    /// Compute the anchor of an exact ledger byte sequence.
    ///
    /// Line digests cover newline-terminated lines only; `byte_len` still
    /// records every byte, including an unterminated tail, so verification
    /// can refuse an anchor that does not end on a line boundary.
    pub fn of(bytes: &[u8]) -> Self {
        let mut line_digests = Vec::new();
        let mut start = 0;
        for (index, byte) in bytes.iter().enumerate() {
            if *byte == b'\n' {
                line_digests.push(sha256_hex(&bytes[start..=index]));
                start = index + 1;
            }
        }
        Self {
            digest: sha256_hex(bytes),
            byte_len: bytes.len(),
            line_digests,
        }
    }
}

/// Ids whose stored data is superseded by a correction that changes it.
///
/// A correction whose effective payload is byte-for-byte equal to the stored
/// target payload supersedes the record without changing any observed fact,
/// so it does not appear here. Results follow ledger order; correction events
/// themselves are instructions, not observations, and are never returned.
pub fn changed_event_ids(events: &[Event]) -> Vec<String> {
    let effective = resolve_effective(events);
    let effective_data: HashMap<&str, &serde_json::Value> = effective
        .iter()
        .map(|event| (event.id.as_str(), &event.data))
        .collect();
    events
        .iter()
        .filter(|event| {
            effective_data
                .get(event.id.as_str())
                .is_some_and(|data| *data != &event.data)
        })
        .map(|event| event.id.clone())
        .collect()
}

/// Apply corrections to the events they supersede, preserving target identity.
///
/// Each event naming a target in `data.supersedes` patches that target's
/// `data` with its own data minus the `supersedes` key. The target keeps its
/// id, type, subject, and timestamp, so a Run-counts correction remains a
/// Run observation with corrected numbers. Later corrections win, and
/// correction events themselves are instructions rather than observations,
/// so they are not returned.
pub fn resolve_effective(events: &[Event]) -> Vec<Event> {
    let mut corrections: HashMap<&str, Vec<&str>> = HashMap::new();
    for event in events {
        let targets = supersede_targets(&event.data);
        if !targets.is_empty() {
            corrections.insert(event.id.as_str(), targets);
        }
    }
    let mut effective_data: HashMap<String, serde_json::Value> = HashMap::new();
    let mut visiting: HashSet<String> = HashSet::new();
    for event in events {
        let Some(targets) = corrections.get(event.id.as_str()) else {
            continue;
        };
        let mut data = event.data.clone();
        if let serde_json::Value::Object(fields) = &mut data {
            fields.remove("supersedes");
        }
        for &target in targets {
            apply_effective_data(
                target,
                &data,
                &corrections,
                &mut effective_data,
                &mut visiting,
            );
        }
    }
    events
        .iter()
        .filter(|event| !corrections.contains_key(event.id.as_str()))
        .map(|event| {
            let mut effective = event.clone();
            if let Some(data) = effective_data.get(&event.id) {
                effective.data = data.clone();
            }
            effective
        })
        .collect()
}

/// Patch the ultimate target of `target`, following a correction chain.
fn apply_effective_data(
    target: &str,
    data: &serde_json::Value,
    corrections: &HashMap<&str, Vec<&str>>,
    effective_data: &mut HashMap<String, serde_json::Value>,
    visiting: &mut HashSet<String>,
) {
    if !visiting.insert(target.to_owned()) {
        return;
    }
    match corrections.get(target) {
        Some(nested_targets) => {
            for &nested in nested_targets {
                apply_effective_data(nested, data, corrections, effective_data, visiting);
            }
        }
        None => {
            effective_data.insert(target.to_owned(), data.clone());
        }
    }
    visiting.remove(target);
}

/// Refuse an event whose `data.supersedes` is malformed, dangling,
/// self-referential, or closes a cycle through the stored ledger.
///
/// `supersedes` must be a non-empty string or a non-empty array of non-empty
/// strings naming ids already stored; every target's own stored chain must
/// not lead back to the new event's id, or the correction history would
/// become circular and unresolvable.
fn validate_supersedes(event: &Event, stored: &[Event]) -> Result<(), LedgerError> {
    let Some(value) = event.data.get("supersedes") else {
        return Ok(());
    };
    let invalid = |detail: String| {
        LedgerError::new(
            LABEL_SUPERSEDES_INVALID,
            format!("event \"{}\" {}: {}", event.id, detail, value),
        )
    };
    let targets: Vec<&str> = match value {
        serde_json::Value::String(target) if !target.is_empty() => vec![target.as_str()],
        serde_json::Value::Array(items) if !items.is_empty() => {
            let mut targets = Vec::with_capacity(items.len());
            for item in items {
                match item {
                    serde_json::Value::String(target) if !target.is_empty() => {
                        targets.push(target.as_str());
                    }
                    _ => {
                        return Err(invalid(
                            "has a data.supersedes that must be a non-empty string or a \
                             non-empty array of non-empty strings, but an entry is not"
                                .to_owned(),
                        ));
                    }
                }
            }
            targets
        }
        _ => {
            return Err(invalid(
                "has a data.supersedes that must be a non-empty string or a non-empty \
                 array of non-empty strings"
                    .to_owned(),
            ));
        }
    };
    let stored_ids: HashSet<&str> = stored.iter().map(|stored| stored.id.as_str()).collect();
    let mut corrections: HashMap<&str, Vec<&str>> = HashMap::new();
    for stored in stored {
        let targets = supersede_targets(&stored.data);
        if !targets.is_empty() {
            corrections.insert(stored.id.as_str(), targets);
        }
    }
    for target in &targets {
        if *target == event.id {
            return Err(invalid(format!(
                "supersedes itself; a correction may not name its own id \"{}\"",
                event.id
            )));
        }
        if !stored_ids.contains(target) {
            return Err(invalid(format!(
                "supersedes \"{}\", which is not present in the ledger",
                target
            )));
        }
        // Walk the target's stored chain; reaching the new event's id closes
        // a cycle the resolver could never untangle.
        let mut frontier = vec![*target];
        let mut visited: HashSet<&str> = HashSet::new();
        while let Some(id) = frontier.pop() {
            if !visited.insert(id) {
                continue;
            }
            if id == event.id {
                return Err(invalid(format!(
                    "closes a supersedes cycle: \"{}\" chain leads back to \"{}\"",
                    target, event.id
                )));
            }
            if let Some(nested) = corrections.get(id) {
                frontier.extend(nested.iter().copied());
            }
        }
    }
    Ok(())
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
            recovery: Recovery::default(),
        }
    }
}
