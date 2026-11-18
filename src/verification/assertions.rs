//! Assertions and staleness reported by `kaya check`.
//!
//! Purpose: let a repository declare what the verified record must contain and
//! refuse authored prose that no longer rests on the evidence it cites, without
//! the verifier evaluating any policy of its own.
//! Responsibilities: validate the request vocabulary, decide which declared
//! kinds and sources the already-derived facts satisfy, and turn the
//! projection's stale report into the entries a check envelope carries.
//! Rationale: a requirement is a claim about the facts the check derived from
//! the verified bytes — never a second ledger read — and staleness reuses the
//! same authored-block analysis `build` reports, so an advisory run and an
//! enforced run name exactly the same blocks and evidence ids.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::Serialize;

use crate::event::Event;
use crate::ledger::changed_event_ids;
use crate::projection::{StaleBlock, report};

use super::facts::{Facts, SOURCES};
use super::{LABEL_USAGE, Source, VerificationError};

/// One requested assertion the verified record does not satisfy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Required {
    /// `kind` or `source`.
    pub area: String,
    /// The requested dotted kind, or the canonical source name.
    pub name: String,
    /// Why the requirement is unmet: `no-records`, `absent`, or `no-events`.
    pub reason: String,
}

/// One authored block whose cited evidence changed after it was written.
///
/// The path is part of the finding because a block name is only unique within
/// its document: `authored-1` in one document says nothing about `authored-1`
/// in another, and a reader needs the document to act on the report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Stale {
    /// Document path relative to the invocation root, `/`-separated.
    pub path: String,
    /// Authored block name, as the projection reports it.
    pub block: String,
    /// Changed evidence ids the block cites.
    pub evidence: Vec<String>,
}

impl Stale {
    /// The entry for one projection stale block in `path`.
    fn from_block(path: String, block: &StaleBlock) -> Self {
        Self {
            path,
            block: block.block.clone(),
            evidence: block.evidence.clone(),
        }
    }
}

/// Refuse a requirement the vocabulary cannot express.
///
/// A kind is a dotted `family.kind` name — the single spelling `emitted_kinds`
/// and `omitted_kinds` share — so a bare kind is a usage failure rather than a
/// requirement that can never be met. A source is one of the canonical capture
/// sources, never an actor or adapter name.
pub fn validate_request(kinds: &[String], sources: &[String]) -> Result<(), VerificationError> {
    for kind in kinds {
        if !is_dotted_kind(kind) {
            return Err(VerificationError::new(
                LABEL_USAGE,
                format!("--require-kind \"{kind}\" is not a dotted `family.kind` name"),
            ));
        }
    }
    for source in sources {
        if !SOURCES.contains(&source.as_str()) {
            return Err(VerificationError::new(
                LABEL_USAGE,
                format!(
                    "--require-source \"{source}\" is not one of {}",
                    SOURCES.join(", ")
                ),
            ));
        }
    }
    Ok(())
}

/// Whether `kind` is a dotted `family.kind` name with both sides non-empty.
///
/// Exactly one dot separates the two parts and neither part carries
/// whitespace: a kind outside that vocabulary could never be emitted or
/// omitted, so accepting it would turn a typo into a permanent silent failure.
fn is_dotted_kind(kind: &str) -> bool {
    match kind.split_once('.') {
        Some((family, kind)) => {
            !family.is_empty()
                && !kind.is_empty()
                && !kind.contains('.')
                && !kind.contains(char::is_whitespace)
                && !family.contains(char::is_whitespace)
        }
        None => false,
    }
}

/// The requested assertions the verified facts do not satisfy, request order.
///
/// A kind is satisfied when the projections emit it; an omitted kind is
/// reported as `no-records`, because the planner saw no records of that family
/// at all, and anything else as `absent`. A source is satisfied by at least one
/// event under its canonical name. Empty lists satisfy everything, so the
/// always-on check reports no requirement.
pub fn required(facts: &Facts, kinds: &[String], sources: &[String]) -> Vec<Required> {
    let mut required: Vec<Required> = Vec::new();

    for kind in kinds {
        if facts.emitted_kinds.iter().any(|emitted| emitted == kind) {
            continue;
        }
        let reason = if facts.omitted_kinds.iter().any(|omitted| omitted == kind) {
            "no-records"
        } else {
            "absent"
        };
        push_once(
            &mut required,
            Required {
                area: "kind".to_owned(),
                name: kind.clone(),
                reason: reason.to_owned(),
            },
        );
    }

    for source in sources {
        let count = facts.events_by_source.get(source).copied().unwrap_or(0);
        if count > 0 {
            continue;
        }
        push_once(
            &mut required,
            Required {
                area: "source".to_owned(),
                name: source.clone(),
                reason: "no-events".to_owned(),
            },
        );
    }

    required
}

/// Record `entry` once: a repeated flag names the same defect, not a second
/// one.
fn push_once(entries: &mut Vec<Required>, entry: Required) {
    if entries
        .iter()
        .any(|seen| seen.area == entry.area && seen.name == entry.name)
    {
        return;
    }
    entries.push(entry);
}

/// Every authored block in the expected documents that cites changed evidence.
///
/// The documents are the fixed point a clean `build` would leave, so each one
/// carries the human's authored bytes exactly as committed. Documents are
/// visited in path order, which is the order the projection plans them in, and
/// each entry names its document as a root-relative `/`-separated path — the
/// form drift findings and their fixes use. The report is sorted and deduped
/// by `(path, block, evidence)`: the same citation reached twice is one
/// finding, exactly as `build` counts it.
pub fn stale(
    source: &Source,
    expected: &BTreeMap<PathBuf, String>,
    events: &[Event],
) -> Vec<Stale> {
    let changed = changed_event_ids(events);
    if changed.is_empty() {
        return Vec::new();
    }
    // No acknowledgments are recorded yet, so every changed citation is stale.
    let acknowledged = BTreeMap::new();
    let mut stale = Vec::new();
    for (path, document) in expected {
        let reported = source.relative(path);
        stale.extend(
            report::stale_authored(document, &changed, &acknowledged)
                .iter()
                .map(|block| Stale::from_block(reported.clone(), block)),
        );
    }
    stale.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then_with(|| left.block.cmp(&right.block))
            .then_with(|| left.evidence.cmp(&right.evidence))
    });
    stale.dedup();
    stale
}
