//! Capture CLI: `kaya note/question/decision/finding` to `model.entity.created`.
//!
//! Purpose: let a human or agent record authored text as one model event per
//! invocation, with secrets redacted before anything is bounded, hashed, or
//! stored.
//! Responsibilities: run the redaction filter, enforce payload bounds, derive
//! the deterministic key and event id, and decide append / duplicate /
//! correction against the key's effective stored text.
//! Rationale: captures are authored text, never adapter output — so the
//! command refuses instead of truncating or rewriting, and an edit is a
//! `data.supersedes` correction that leaves stored bytes untouched.

use std::fs;

use chrono::{SecondsFormat, Utc};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use kaya::adapters::{correction_generation, reobservation_correction};
use kaya::envelope::{CLI_VERSION, Envelope};
use kaya::event::{Actor, ActorType, Event, Provenance, SCHEMA_V0, Subject, parse_line};
use kaya::ledger::{AppendOutcome, Ledger, sha256_hex};

use crate::LEDGER_PATH;

const TOOL: &str = "kaya";
/// The event type every capture command appends.
const EVENT_TYPE: &str = "model.entity.created";
/// Maximum redacted text length, in characters.
const TEXT_LIMIT: usize = 2000;
/// Replacement marker substituted for every matched secret.
const MARKER: &str = "[redacted]";
/// Deterministic id digest length, matching the adapters' scheme.
const ID_DIGEST_LEN: usize = 26;

/// The closed-vocabulary entity kind each capture subcommand records.
#[derive(Clone, Copy, Debug)]
pub enum Kind {
    Note,
    Question,
    Decision,
    Finding,
}

impl Kind {
    /// The entity type recorded on the event's subject.
    pub fn entity_type(self) -> &'static str {
        match self {
            Self::Note => "Observation",
            Self::Question => "Question",
            Self::Decision => "Decision",
            Self::Finding => "Finding",
        }
    }

    /// The subcommand spelling used in the provenance reference.
    pub fn subcommand(self) -> &'static str {
        match self {
            Self::Note => "note",
            Self::Question => "question",
            Self::Decision => "decision",
            Self::Finding => "finding",
        }
    }
}

/// One capture invocation: redact, bound, derive, replay, append.
///
/// The predicate: the newest stored event for the derived subject id (ledger
/// order) carrying a `data.text` decides the outcome — no such event (or none
/// with text) means a first capture; equal redacted text means a duplicate
/// naming that event's id; different text means a correction superseding it.
pub fn run(
    ledger: &mut Ledger,
    kind: Kind,
    text: &str,
    actor: &str,
    actor_type: &str,
    key: Option<&str>,
    project: &str,
) -> Result<Envelope, (String, String)> {
    // Redaction runs first: bounds apply to the authored bytes that would be
    // stored, never to the raw paste.
    let text = redact(text);
    if text.is_empty() {
        return Err((
            "kaya.capture.empty_text".to_owned(),
            "capture text is empty; capture needs non-empty authored text".to_owned(),
        ));
    }
    let chars = text.chars().count();
    if chars > TEXT_LIMIT {
        return Err((
            "kaya.capture.bound_exceeded".to_owned(),
            format!("capture text is {chars} characters; the limit is {TEXT_LIMIT}"),
        ));
    }
    let key = match key {
        Some(key) => key.to_owned(),
        None => default_key(&text),
    };
    check_key(&key)?;

    let actor_type = match actor_type {
        "human" => ActorType::Human,
        "agent" => ActorType::Agent,
        _ => {
            return Err((
                "kaya.capture.actor_required".to_owned(),
                format!(
                    "actor type \"{actor_type}\" may not author a capture; captures are \
                     authored by a human or agent actor named with --actor"
                ),
            ));
        }
    };

    let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    let subject_id = format!("{project}/{}/{key}", kind.entity_type());
    let data = json!({ "text": text, "key": key });
    let event = Event {
        id: event_id(project, &timestamp, &subject_id, &data),
        schema: SCHEMA_V0.to_owned(),
        r#type: EVENT_TYPE.to_owned(),
        timestamp,
        actor: Actor {
            r#type: actor_type,
            name: actor.to_owned(),
        },
        subject: Subject {
            r#type: kind.entity_type().to_owned(),
            id: subject_id.clone(),
        },
        data: data.clone(),
        provenance: Provenance {
            tool: TOOL.to_owned(),
            version: CLI_VERSION.to_owned(),
            reference: format!("{}/{key}", kind.subcommand()),
        },
        artifacts: Vec::new(),
    };

    // Replay against the key's effective stored text before appending.
    let stored = stored_events()?;
    let effective = stored
        .iter()
        .rev()
        .find(|stored| stored.subject.id == subject_id && stored.data.get("text").is_some());
    let (outcome, id) = match effective {
        Some(effective) if effective.data["text"] == data["text"] => {
            ("duplicate", effective.id.clone())
        }
        Some(effective) => {
            // Reuse the adapters' correction scheme over the prior effective
            // id: the generation salt keeps revisits of an earlier state
            // distinct from the correction already stored for it.
            let mut target = event.clone();
            target.id = effective.id.clone();
            let generation = correction_generation(&stored, &target);
            let correction = reobservation_correction(&target, generation);
            match ledger.append(&correction) {
                Ok(AppendOutcome::Appended | AppendOutcome::Duplicate) => {
                    ("corrected", correction.id)
                }
                Err(err) if err.label == kaya::ledger::LABEL_ID_REUSE => {
                    // A racing twin minted the same generation-salted id with
                    // its own timestamp; re-resolve against the stored state.
                    let stored = stored_events()?;
                    match stored.iter().rev().find(|stored| {
                        stored.subject.id == subject_id && stored.data.get("text").is_some()
                    }) {
                        Some(effective) if effective.data["text"] == data["text"] => {
                            ("duplicate", effective.id.clone())
                        }
                        _ => return Err(crate::ledger_failure(err)),
                    }
                }
                Err(err) => return Err(crate::ledger_failure(err)),
            }
        }
        None => match ledger.append(&event).map_err(crate::ledger_failure)? {
            AppendOutcome::Appended | AppendOutcome::Duplicate => ("appended", event.id.clone()),
        },
    };

    Ok(Envelope::ok(
        "capture",
        json!({
            "outcome": outcome,
            "event": id,
            "ledger": LEDGER_PATH,
        }),
    ))
}

/// The stored ledger events, in ledger order.
fn stored_events() -> Result<Vec<Event>, (String, String)> {
    let text = fs::read_to_string(LEDGER_PATH).map_err(|err| {
        (
            "ledger-io".to_owned(),
            format!("failed to read {LEDGER_PATH}: {err}"),
        )
    })?;
    let events = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            parse_line(line).map_err(|err| {
                (
                    err.label.to_owned(),
                    format!("failed to parse {LEDGER_PATH}: {}", err.message),
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(events)
}

/// The default key: `<UTC date of capture>-<sha256(redacted text)[:8]>`.
fn default_key(text: &str) -> String {
    let date = &Utc::now().format("%Y-%m-%d").to_string()[..];
    let digest = Sha256::digest(text.as_bytes());
    let hex = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("{date}-{}", &hex[..8])
}

/// Refuse a key outside `1..=64` characters of `[a-z0-9-]`.
fn check_key(key: &str) -> Result<(), (String, String)> {
    let valid = !key.is_empty()
        && key.len() <= 64
        && key
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
    if valid {
        Ok(())
    } else {
        Err((
            "kaya.capture.bound_exceeded".to_owned(),
            format!("capture key {key:?} must be 1 to 64 characters of [a-z0-9-]"),
        ))
    }
}

/// Deterministic capture id: `evt-` plus the first 26 hex characters of
/// sha256 over the `\x1f`-joined identity payload, mirroring the adapters'
/// derivation scheme.
fn event_id(project: &str, timestamp: &str, subject_id: &str, data: &Value) -> String {
    let data = serde_json::to_string(data).expect("event data is valid JSON");
    let identity =
        format!("{TOOL}\x1f{CLI_VERSION}\x1f{project}\x1f{timestamp}\x1f{subject_id}\x1f{data}");
    let digest = sha256_hex(identity.as_bytes());
    format!("evt-{}", &digest[..ID_DIGEST_LEN])
}

/// Replace each matched secret with `[redacted]` in place.
///
/// Deterministic and order-stable: the inherited session-capture pattern
/// list is tried at every position before the default rule, and a match
/// consumes exactly the secret so the surrounding text survives byte-exactly.
fn redact(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < bytes.len() {
        if let Some(len) = match_secret(bytes, i) {
            out.push_str(MARKER);
            i += len;
        } else {
            let ch = text[i..].chars().next().expect("input is valid UTF-8");
            out.push(ch);
            i += ch.len_utf8();
        }
    }
    out
}

/// The length of the secret starting at `i`, or `None`.
fn match_secret(bytes: &[u8], i: usize) -> Option<usize> {
    private_key_armor(bytes, i)
        .or_else(|| literal(bytes, i, b"PuTTY-User-Key-File"))
        .or_else(|| fixed_after(bytes, i, b"ghp_", 36, 36, is_alnum))
        .or_else(|| {
            fixed_after(bytes, i, b"github_pat_", 22, usize::MAX, |b| {
                is_alnum(b) || b == b'_'
            })
        })
        .or_else(|| {
            fixed_after(bytes, i, b"sk-", 16, usize::MAX, |b| {
                is_alnum(b) || b == b'-' || b == b'_'
            })
        })
        .or_else(|| aws_id(bytes, i))
        .or_else(|| slack_token(bytes, i))
        .or_else(|| jwt(bytes, i))
        .or_else(|| keyword_assignment(bytes, i))
        .or_else(|| entropy_run(bytes, i))
}

/// `-----BEGIN [A-Z ]*PRIVATE KEY-----` armour, subsuming every OpenSSL
/// header spelling (RSA, DSA, EC, ECDSA, OPENSSH, ENCRYPTED, bare) and the
/// PGP `PRIVATE KEY BLOCK` form from the inherited pattern list.
fn private_key_armor(bytes: &[u8], i: usize) -> Option<usize> {
    const OPEN: &[u8] = b"-----BEGIN ";
    const CLOSES: &[&[u8]] = &[b"PRIVATE KEY-----", b"PRIVATE KEY BLOCK-----"];
    if !bytes[i..].starts_with(OPEN) {
        return None;
    }
    let mut end = i + OPEN.len();
    loop {
        for close in CLOSES {
            if literal(bytes, end, close).is_some() {
                return Some(end + close.len() - i);
            }
        }
        let byte = *bytes.get(end)?;
        if !(byte.is_ascii_uppercase() || byte == b' ') {
            return None;
        }
        end += 1;
    }
}

/// A fixed prefix followed by `min..=max` characters of an allowed class.
fn fixed_after(
    bytes: &[u8],
    i: usize,
    prefix: &[u8],
    min: usize,
    max: usize,
    class: impl Fn(u8) -> bool,
) -> Option<usize> {
    literal(bytes, i, prefix)?;
    let start = i + prefix.len();
    let mut end = start;
    while end < bytes.len() && class(bytes[end]) && end - start < max {
        end += 1;
    }
    if end - start >= min {
        Some(end - i)
    } else {
        None
    }
}

/// `AKIA…` / `ASIA…`: 16 characters of `[0-9A-Z]`.
fn aws_id(bytes: &[u8], i: usize) -> Option<usize> {
    [b"AKIA".as_slice(), b"ASIA".as_slice()]
        .iter()
        .find_map(|prefix| {
            fixed_after(bytes, i, prefix, 16, 16, |b| {
                b.is_ascii_digit() || b.is_ascii_uppercase()
            })
        })
}

/// `xox[baprs]-` followed by 10+ characters of `[A-Za-z0-9-]`.
fn slack_token(bytes: &[u8], i: usize) -> Option<usize> {
    let kind = *bytes.get(i + 3)?;
    if !bytes[i..].starts_with(b"xox") || !matches!(kind, b'b' | b'a' | b'p' | b'r' | b's') {
        return None;
    }
    let mut end = i + 4;
    if bytes.get(end) != Some(&b'-') {
        return None;
    }
    end += 1;
    let start = end;
    while end < bytes.len() && (is_alnum(bytes[end]) || bytes[end] == b'-') {
        end += 1;
    }
    (end - start >= 10).then_some(end - i)
}

/// A three-segment `eyJ…` JWT.
fn jwt(bytes: &[u8], i: usize) -> Option<usize> {
    if !bytes[i..].starts_with(b"eyJ") {
        return None;
    }
    let mut end = i + 3;
    for expected in 1..=3 {
        let start = end;
        while end < bytes.len()
            && (is_alnum(bytes[end]) || bytes[end] == b'-' || bytes[end] == b'_')
        {
            end += 1;
        }
        if end - start < 10 {
            return None;
        }
        if expected < 3 {
            if bytes.get(end) != Some(&b'.') {
                return None;
            }
            end += 1;
        }
    }
    Some(end - i)
}

/// `(api[_-]?key|secret|password|token)\s*[:=]\s*\S{8,}`, case-insensitive.
fn keyword_assignment(bytes: &[u8], i: usize) -> Option<usize> {
    const KEYWORDS: &[&str] = &[
        "api key", "api_key", "api-key", "apikey", "secret", "password", "token",
    ];
    let mut keyword_len = 0;
    for keyword in KEYWORDS {
        let candidate = bytes.get(i..i + keyword.len())?;
        if candidate.eq_ignore_ascii_case(keyword.as_bytes()) {
            keyword_len = keyword.len();
            break;
        }
    }
    if keyword_len == 0 {
        return None;
    }
    let mut end = i + keyword_len;
    while end < bytes.len() && bytes[end].is_ascii_whitespace() {
        end += 1;
    }
    if !matches!(bytes.get(end), Some(b':') | Some(b'=')) {
        return None;
    }
    end += 1;
    while end < bytes.len() && bytes[end].is_ascii_whitespace() {
        end += 1;
    }
    let start = end;
    while end < bytes.len() && !bytes[end].is_ascii_whitespace() {
        end += 1;
    }
    (end - start >= 8).then_some(end - i)
}

/// A base64 run of ≥ 32 characters mixing digits, upper, and lower case —
/// or a hex run of ≥ 32 characters containing at least one hex letter and at
/// least one digit: secret by default, so an unlisted key family cannot leak
/// silently. Words, all-digit spans, and letter-only fillers (which the
/// over-bound refusal relies on) survive.
fn entropy_run(bytes: &[u8], i: usize) -> Option<usize> {
    let mut end = i;
    let (mut digit, mut lower, mut upper, mut hex_letter) = (false, false, false, false);
    while end < bytes.len() && is_alnum(bytes[end]) {
        let byte = bytes[end];
        digit |= byte.is_ascii_digit();
        lower |= byte.is_ascii_lowercase();
        upper |= byte.is_ascii_uppercase();
        hex_letter |= byte.is_ascii_hexdigit() && !byte.is_ascii_digit();
        end += 1;
    }
    let mixed = end - i >= 32 && digit && lower && upper;
    let hex = end - i >= 32 && hex_letter && digit;
    (mixed || hex).then_some(end - i)
}

/// A literal byte match, returning its length.
fn literal(bytes: &[u8], i: usize, pattern: &[u8]) -> Option<usize> {
    bytes[i..].starts_with(pattern).then_some(pattern.len())
}

fn is_alnum(byte: u8) -> bool {
    byte.is_ascii_alphanumeric()
}
