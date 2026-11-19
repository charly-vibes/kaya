//! Hegel property tests for the ledger (scratchbook dossiers
//! `correction-totality` and `append-byte-monotonicity`).
//!
//! Both properties scope themselves to their well-formed domains so a
//! passing suite stays passing: correction-totality only generates ledgers
//! where every `supersedes` target exists at append time (no cycles, no
//! self-reference, corrections appended after their targets — the malformed
//! arms are tickets kaya-cl6 and kaya-8k5), and append-byte-monotonicity
//! excludes the cosmetic-replay edge (ticket kaya-sh6). The alphabets are
//! small and exhaustive; the generator draws bounded integers and decodes
//! them into structural arms, so chains are built by construction, not
//! random search.

use std::collections::HashMap;
use std::fs;

use hegel::TestCase;
use hegel::generators as gs;
use kaya::event::{Actor, ActorType, Event, Provenance, SCHEMA_V0, Subject, to_line};
use kaya::ledger::{AppendOutcome, Ledger, Recovery, resolve_effective};
use serde_json::{Value, json};
use tempfile::TempDir;

const TIMESTAMP: &str = "2026-11-19T10:00:00Z";

/// Build an event like `tests/ledger.rs` does, over the closed field set.
fn event_with(id: &str, event_type: &str, data: Value) -> Event {
    Event {
        id: id.to_owned(),
        schema: SCHEMA_V0.to_owned(),
        r#type: event_type.to_owned(),
        timestamp: TIMESTAMP.to_owned(),
        actor: Actor {
            r#type: ActorType::Tool,
            name: "cargo-test".to_owned(),
        },
        subject: Subject {
            r#type: "Run".to_owned(),
            id: "kaya/Run/RUN-001".to_owned(),
        },
        data,
        provenance: Provenance {
            tool: "kaya".to_owned(),
            version: "0.1.0".to_owned(),
            reference: "commit:abc123".to_owned(),
        },
        artifacts: vec!["reports/run.json".to_owned()],
    }
}

fn observation(id: &str, passed: u32) -> Event {
    event_with(id, "experiment.run.completed", json!({ "passed": passed }))
}

/// How a correction names its targets: one id rendered as a bare string, one
/// id rendered as a one-element array, or several ids as an array — all
/// forms `supersede_targets` accepts.
enum Supersedes {
    String(String),
    ArrayOne(String),
    ArrayMany(Vec<String>),
}

fn correction(id: &str, supersedes: Supersedes, passed: u32) -> Event {
    let supersedes_value = match supersedes {
        Supersedes::String(target) => json!(target),
        Supersedes::ArrayOne(target) => json!([target]),
        Supersedes::ArrayMany(targets) => json!(targets),
    };
    event_with(
        id,
        "experiment.run.corrected",
        json!({ "passed": passed, "supersedes": supersedes_value }),
    )
}

fn supersedes_targets_of(event: &Event) -> Vec<String> {
    match event.data.get("supersedes") {
        Some(Value::String(target)) => vec![target.clone()],
        Some(Value::Array(targets)) => targets
            .iter()
            .filter_map(|target| target.as_str().map(str::to_owned))
            .collect(),
        _ => Vec::new(),
    }
}

/// Independent oracle: replay corrections in ledger order, walk each target
/// chain to its terminal, last writer wins, `supersedes` stripped from the
/// patched payload. The domain is acyclic, so plain recursion terminates.
fn oracle_effective(events: &[Event]) -> Vec<Event> {
    let corrections: HashMap<&str, Vec<String>> = events
        .iter()
        .map(|event| (event.id.as_str(), supersedes_targets_of(event)))
        .filter(|(_, targets)| !targets.is_empty())
        .collect();

    fn walk(
        target: &str,
        corrections: &HashMap<&str, Vec<String>>,
        payload: &Value,
        effective: &mut HashMap<String, Value>,
    ) {
        match corrections.get(target) {
            Some(nexts) => {
                for next in nexts {
                    walk(next, corrections, payload, effective);
                }
            }
            None => {
                effective.insert(target.to_owned(), payload.clone());
            }
        }
    }

    let mut effective: HashMap<String, Value> = HashMap::new();
    for event in events {
        let targets = supersedes_targets_of(event);
        if targets.is_empty() {
            continue;
        }
        let mut payload = event.data.clone();
        if let Value::Object(fields) = &mut payload {
            fields.remove("supersedes");
        }
        for target in &targets {
            walk(target, &corrections, &payload, &mut effective);
        }
    }

    events
        .iter()
        .filter(|event| !corrections.contains_key(event.id.as_str()))
        .map(|event| {
            let mut resolved = event.clone();
            if let Some(data) = effective.get(&event.id) {
                resolved.data = data.clone();
            }
            resolved
        })
        .collect()
}

/// One draw byte packs three decisions over an exhaustive small alphabet:
/// the low 3 bits pick the structural arm (chain depth 0-3 or the diamond),
/// bit 3 picks the single-target `supersedes` rendering (string vs
/// one-element array), bits 4-5 pick the payload.
fn decode(draw: u8) -> (u8, bool, u32) {
    let shape = draw & 0b111;
    let as_array_form = (draw >> 3) & 1 == 1;
    let passed = ((draw >> 4) & 0b11) as u32 + 1;
    (shape, as_array_form, passed)
}

/// `correction-totality`: for well-formed ledgers (every `supersedes`
/// target exists at append time, no cycles, no self-reference, corrections
/// appended after their targets), `resolve_effective` returns every
/// non-correction event exactly once, each superseded event carrying the
/// payload of the last correction in ledger order that transitively reaches
/// it, with the `supersedes` key stripped and id/type/subject/timestamp/
/// actor/provenance/artifacts preserved — and never returns a correction
/// event.
#[hegel::test]
fn correction_totality(tc: TestCase) {
    let draws = tc.draw(
        gs::vecs(gs::integers::<u8>().min_value(0).max_value(63))
            .min_size(1)
            .max_size(4),
    );

    let mut events: Vec<Event> = Vec::new();
    for (slot, draw) in draws.iter().enumerate() {
        let (shape, as_array_form, passed) = decode(*draw);
        let obs_id = format!("OBS{slot}");
        events.push(observation(&obs_id, passed));

        match shape {
            // Depth 0: a plain observation, nothing supersedes it.
            0 => {}
            // Depth 1-3: a correction chain of that length, each correction
            // naming its predecessor, which always exists at append time.
            depth @ 1..=3 => {
                let mut prev = obs_id.clone();
                for k in 1..=u32::from(depth) {
                    let id = format!("COR{slot}_{k}");
                    let target = if as_array_form {
                        Supersedes::ArrayOne(prev.clone())
                    } else {
                        Supersedes::String(prev.clone())
                    };
                    events.push(correction(&id, target, passed + k));
                    prev = id;
                }
            }
            // Diamond: corrections B and D both supersede the observation,
            // then a multi-target correction supersedes [B, D] as an array;
            // both branches reach one terminal.
            _ => {
                let branch_b = format!("COR{slot}_B");
                let branch_d = format!("COR{slot}_D");
                let apex = format!("COR{slot}_A");
                events.push(correction(
                    &branch_b,
                    Supersedes::String(obs_id.clone()),
                    passed + 1,
                ));
                events.push(correction(
                    &branch_d,
                    Supersedes::ArrayOne(obs_id.clone()),
                    passed + 2,
                ));
                events.push(correction(
                    &apex,
                    Supersedes::ArrayMany(vec![branch_b.clone(), branch_d.clone()]),
                    passed + 3,
                ));
            }
        }
    }

    let resolved = resolve_effective(&events);
    let expected = oracle_effective(&events);
    assert_eq!(
        resolved, expected,
        "resolve_effective diverged from the independent oracle"
    );

    // Every non-correction event survives exactly once, in ledger order.
    let surviving_ids: Vec<&str> = events
        .iter()
        .filter(|event| supersedes_targets_of(event).is_empty())
        .map(|event| event.id.as_str())
        .collect();
    let resolved_ids: Vec<&str> = resolved.iter().map(|event| event.id.as_str()).collect();
    assert_eq!(
        resolved_ids, surviving_ids,
        "an observation silently disappeared from (or was reordered in) the output"
    );

    // No correction-typed event appears in the output, and no patched data
    // retains the supersedes key.
    assert!(
        resolved
            .iter()
            .all(|event| event.data.get("supersedes").is_none()),
        "a correction event or an unstripped supersedes key leaked into the output"
    );
}

/// `append-byte-monotonicity`: for any sequence of appends of generated
/// events plus deliberate near-duplicates, the ledger file only ever changes
/// by whole canonical lines appended at the end; every outcome is exactly
/// one of `Appended` (bytes grew by `to_line(e)`), `Duplicate` (bytes
/// unchanged), or a labeled `id_reuse` refusal (bytes bit-identical to
/// before). The file is always a concatenation of canonical lines and is
/// prefix-monotone across steps. The cosmetic-replay edge is excluded
/// (ticket kaya-sh6): near-duplicate arms use the same bytes with different
/// ids, or the same id with different bytes — never hand-edited lines.
#[hegel::test]
fn append_byte_monotonicity(tc: TestCase) {
    let draws = tc.draw(
        gs::vecs(gs::integers::<u8>().min_value(0).max_value(63))
            .min_size(1)
            .max_size(20),
    );

    let dir = TempDir::new().unwrap();
    let path = dir.path().join("ledger.jsonl");
    let mut ledger = Ledger::new(&path);

    let mut stored: Vec<Event> = Vec::new();
    let mut fresh = 0usize;

    for draw in draws {
        let before = fs::read(&path).unwrap_or_default();
        let op = draw & 0b11;
        let passed = ((draw >> 4) & 0b11) as u32;

        match op {
            // Fresh event: appended; bytes grow by exactly its line.
            0 => {
                let event = observation(&format!("EVT{fresh:03}"), passed);
                fresh += 1;
                assert_eq!(
                    ledger.append(&event).unwrap(),
                    AppendOutcome::Appended,
                    "a fresh event must append"
                );
                stored.push(event);
            }
            // Byte-identical replay of an earlier event: duplicate, no write.
            1 if !stored.is_empty() => {
                let index = ((draw >> 2) as usize) % stored.len();
                let replay = stored[index].clone();
                assert_eq!(
                    ledger.append(&replay).unwrap(),
                    AppendOutcome::Duplicate,
                    "a byte-identical replay must be acknowledged as a duplicate"
                );
            }
            // Same id, different bytes: labeled refusal, no write.
            2 if !stored.is_empty() => {
                let index = ((draw >> 2) as usize) % stored.len();
                let mut reused = stored[index].clone();
                reused.data = json!({ "passed": passed + 10 });
                let err = ledger
                    .append(&reused)
                    .expect_err("id reuse with different bytes must be refused");
                assert_eq!(err.label, "kaya.ledger.id_reuse");
                assert!(
                    err.message.contains(&stored[index].id),
                    "refusal must name the reused id: {}",
                    err.message
                );
            }
            // The same bytes modulo the id: a distinct canonical line,
            // appended — the scoped near-duplicate arm.
            3 if !stored.is_empty() => {
                let index = ((draw >> 2) as usize) % stored.len();
                let mut sibling = stored[index].clone();
                sibling.id = format!("EVT{fresh:03}");
                fresh += 1;
                assert_eq!(
                    ledger.append(&sibling).unwrap(),
                    AppendOutcome::Appended,
                    "a near-duplicate with a fresh id must append"
                );
                stored.push(sibling);
            }
            // Replay/refusal/sibling arms before anything is stored fall
            // back to a fresh append, keeping every generated step total.
            _ => {
                let event = observation(&format!("EVT{fresh:03}"), passed);
                fresh += 1;
                assert_eq!(ledger.append(&event).unwrap(), AppendOutcome::Appended);
                stored.push(event);
            }
        }

        let after = fs::read(&path).unwrap();
        // Prefix-monotone: bytes never change, only grow at the end.
        assert!(
            after.starts_with(&before),
            "ledger bytes were rewritten, not appended"
        );
        // The file is exactly the concatenation of the canonical lines of
        // the events stored so far — whole canonical lines only.
        let canonical: Vec<u8> = stored
            .iter()
            .flat_map(|event| to_line(event).expect("event data is finite").into_bytes())
            .collect();
        assert_eq!(
            after, canonical,
            "file must be a concatenation of canonical lines"
        );
        assert!(
            after.ends_with(b"\n"),
            "every stored line is newline-terminated"
        );
    }

    // Round-trip: a recovered read serves exactly the stored events with no
    // repair, confirming the appended lines are the whole truth.
    let (events, recovery) = ledger.read_recovered().unwrap();
    assert_eq!(
        recovery,
        Recovery::default(),
        "no repair on well-formed appends"
    );
    assert_eq!(events, stored);
}
