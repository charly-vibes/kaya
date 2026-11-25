//! Verification commands: human promotion, challenge, and acknowledgment.
//!
//! Responsibilities: hold `cmd_promote` and `cmd_challenge` (replay-checked
//! human events over promotion state) and `cmd_acknowledge` (the writer the
//! staleness report reads), together with their private equivalence and
//! failure-mapping helpers.
//!
//! Rationale: the clap argument tree stays in `main.rs`; this module carries
//! only the command bodies and their family-private helpers, sharing the
//! ledger-opening and failure helpers kept in `main.rs`.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use kaya::envelope::Envelope;
use kaya::event::{Event, Provenance, SCHEMA_V0, Subject, new_id};
use kaya::ledger::{AppendOutcome, Ledger, LedgerError};
use kaya::projection::report;
use kaya::promotion::{self, PromotionState};
use serde_json::json;

use crate::{
    Failure, LEDGER_PATH, failure, human_actor, ledger_failure, open_ledger, projection_failure,
    promotion_failure,
};

pub(crate) fn cmd_promote(assertion_id: &str, actor_name: &str) -> Result<Envelope, Failure> {
    let mut ledger = open_ledger()?;
    let (events, _recovery) = ledger.read_recovered().map_err(ledger_failure)?;
    let approval =
        promotion::approve(&human_actor(actor_name), assertion_id).map_err(promotion_failure)?;

    // Replaying the record with the candidate event is what refuses an
    // approval of an assertion that was never recorded: the projection, not
    // the CLI, owns the promotion rules.
    let mut replayed = events;
    replayed.push(approval.clone());
    PromotionState::from_events(&replayed).map_err(promotion_failure)?;

    ledger.append(&approval).map_err(ledger_failure)?;
    Ok(Envelope::ok("promote", json!({ "event": approval })))
}

pub(crate) fn cmd_challenge(
    conclusion_id: &str,
    counter_event_ids: &[String],
    actor_name: &str,
) -> Result<Envelope, Failure> {
    let mut ledger = open_ledger()?;
    let (events, _recovery) = ledger.read_recovered().map_err(ledger_failure)?;
    let challenge =
        promotion::challenge(&human_actor(actor_name), conclusion_id, counter_event_ids)
            .map_err(promotion_failure)?;

    // Replay proves the target is an established conclusion before the
    // challenge is written; the projection carries the refusal label.
    let mut replayed = events;
    replayed.push(challenge.clone());
    PromotionState::from_events(&replayed).map_err(promotion_failure)?;

    ledger.append(&challenge).map_err(ledger_failure)?;
    Ok(Envelope::ok("challenge", json!({ "event": challenge })))
}

/// Append a human acknowledgment of one authored block's changed evidence.
///
/// This is the writer the staleness report reads: exactly one
/// `verification.acknowledged` event naming the document, the block, and the
/// ids reviewed. Only a human may acknowledge — the record is a claim that a
/// person read the evidence. Every refusal happens before the append, so a
/// rejected invocation leaves the ledger byte-identical. An equivalent
/// acknowledgment already recorded is reported with `appended: false` and the
/// stored event, so repeating the command appends nothing. Authored bytes are
/// never touched and nothing is rebuilt: the next `build` or `check` reflects
/// the acknowledgment.
pub(crate) fn cmd_acknowledge(
    document: &Path,
    block: &str,
    event_ids: &[String],
    actor_name: &str,
    actor_type: &str,
) -> Result<Envelope, Failure> {
    if actor_type != "human" {
        return Err(failure(
            "kaya.acknowledge.tool_actor",
            format!(
                "actor type \"{actor_type}\" may not acknowledge authored evidence; \
                 only a human can acknowledge what a human wrote"
            ),
        ));
    }

    let canonical = fs::canonicalize(document).ok();
    let canonical_cwd = std::env::current_dir()
        .ok()
        .and_then(|dir| fs::canonicalize(dir).ok());
    let label = report::document_label_for(document);
    if let (Some(canonical), Some(cwd)) = (&canonical, &canonical_cwd)
        && canonical.strip_prefix(cwd).is_err()
    {
        return Err(failure(
            "kaya.acknowledge.unknown_document",
            format!(
                "unknown document \"{label}\": it lies outside the working directory {}",
                cwd.display()
            ),
        ));
    }
    match fs::metadata(document) {
        Ok(metadata) if metadata.is_file() => {}
        Ok(_) => {
            return Err(failure(
                "kaya.acknowledge.unknown_document",
                format!("unknown document \"{label}\": the path is not a file"),
            ));
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return Err(failure(
                "kaya.acknowledge.unknown_document",
                format!("unknown document \"{label}\": {error}"),
            ));
        }
        Err(error) => {
            return Err(failure(
                "kaya.acknowledge.document_io",
                format!("failed to inspect \"{label}\": {error}"),
            ));
        }
    }

    let text = fs::read_to_string(document).map_err(|error| {
        failure(
            "kaya.acknowledge.document_io",
            format!("failed to read \"{label}\": {error}"),
        )
    })?;
    let blocks = report::authored_block_names(&text).map_err(projection_failure)?;
    if !blocks.iter().any(|name| name == block) {
        let available = if blocks.is_empty() {
            "it carries none".to_owned()
        } else {
            format!("the document carries {}", blocks.join(", "))
        };
        return Err(failure(
            "kaya.acknowledge.unknown_block",
            format!("\"{label}\" has no authored block \"{block}\": {available}"),
        ));
    }

    let mut ledger = Ledger::open(LEDGER_PATH).map_err(acknowledge_ledger_failure)?;
    let (events, _recovery) = ledger
        .read_recovered()
        .map_err(acknowledge_ledger_failure)?;

    let known: BTreeSet<&str> = events.iter().map(|event| event.id.as_str()).collect();
    for id in event_ids {
        if !known.contains(id.as_str()) {
            return Err(failure(
                "kaya.acknowledge.unknown_event",
                format!(
                    "unknown event id \"{id}\": an acknowledgment covers evidence the ledger records"
                ),
            ));
        }
    }
    let covered: Vec<String> = event_ids
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let latest = report::latest_corrections(&events);
    let reviewed: BTreeMap<String, String> = covered
        .iter()
        .filter_map(|id| {
            latest
                .get(id)
                .map(|correction| (id.clone(), correction.clone()))
        })
        .collect();

    if let Some(existing) = equivalent_acknowledgment(&events, &label, block, &covered, &reviewed) {
        return Ok(Envelope::ok(
            "acknowledge",
            json!({ "event": existing, "appended": false }),
        ));
    }

    let acknowledgment = Event {
        id: new_id(),
        schema: SCHEMA_V0.to_owned(),
        r#type: report::ACKNOWLEDGED_EVENT.to_owned(),
        timestamp: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        actor: human_actor(actor_name),
        subject: Subject {
            r#type: report::DOCUMENT_SUBJECT.to_owned(),
            id: label.clone(),
        },
        data: json!({ "block": block, "events": covered, "reviewed": reviewed }),
        provenance: Provenance {
            tool: "kaya".to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            reference: label,
        },
        artifacts: Vec::new(),
    };
    let outcome = ledger
        .append(&acknowledgment)
        .map_err(acknowledge_ledger_failure)?;

    Ok(Envelope::ok(
        "acknowledge",
        json!({
            "event": acknowledgment,
            "appended": outcome == AppendOutcome::Appended,
        }),
    ))
}

/// The recorded acknowledgment covering exactly `(document, block, ids,
/// reviewed)`, if any.
///
/// Replay is a no-op only when the id sets and the reviewed correction map
/// match: acknowledging a subset again is a second, narrower review, and the
/// same ids after a newer correction reviewed a version the human has not
/// seen.
fn equivalent_acknowledgment<'a>(
    events: &'a [Event],
    document: &str,
    block: &str,
    ids: &[String],
    reviewed: &BTreeMap<String, String>,
) -> Option<&'a Event> {
    let wanted: BTreeSet<&str> = ids.iter().map(String::as_str).collect();
    events.iter().find(|event| {
        if event.r#type != report::ACKNOWLEDGED_EVENT
            || event.subject.r#type != report::DOCUMENT_SUBJECT
            || report::document_label(&event.subject.id) != document
        {
            return false;
        }
        let Some(data) = event.data.as_object() else {
            return false;
        };
        if data.get("block").and_then(|value| value.as_str()) != Some(block) {
            return false;
        }
        let stored: BTreeSet<&str> = data
            .get("events")
            .and_then(|value| value.as_array())
            .into_iter()
            .flatten()
            .filter_map(|id| id.as_str())
            .filter(|id| !id.is_empty())
            .collect();
        if stored != wanted {
            return false;
        }
        let stored_reviewed: BTreeMap<&str, &str> = data
            .get("reviewed")
            .and_then(|value| value.as_object())
            .into_iter()
            .flatten()
            .filter_map(|(target, correction)| {
                correction
                    .as_str()
                    .map(|correction| (target.as_str(), correction))
            })
            .collect();
        stored_reviewed.len() == reviewed.len()
            && reviewed.iter().all(|(target, correction)| {
                stored_reviewed.get(target.as_str()).copied() == Some(correction.as_str())
            })
    })
}

/// A ledger refusal inside `kaya acknowledge`, kept in the command's namespace.
fn acknowledge_ledger_failure(error: LedgerError) -> Failure {
    failure(
        "kaya.acknowledge.ledger",
        format!("{}: {}", error.label, error.message),
    )
}
