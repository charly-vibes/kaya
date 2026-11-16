//! Epistemic promotion: events, assertions, and approved conclusions.
//!
//! Purpose: keep observation separate from interpretation, mechanically.
//! Responsibilities: record assertions as `proposed`, require a human actor
//! for every promotion to `established`, allow counter-assertions to demote
//! a conclusion's displayed status to `challenged`, and cap pending
//! proposals by rank instead of pushing them onto a human.
//! Rationale: a tool reports counts; only an approved human interpretation
//! may say what those counts mean. Agents may propose visibly, never
//! establish silently.
//!
//! Two event families carry promotion state:
//!
//! - `promotion.assertion.recorded` with data
//!   `{text, status: "proposed", event_ids, machine_proposed}`
//! - `promotion.conclusion.established` with data
//!   `{assertion, approver}` — human actors only
//! - `promotion.conclusion.challenged` with data
//!   `{conclusion, counter_event_ids}`
//!
//! [`PromotionState::from_events`] replays those events in ledger order, so
//! the state is a projection of the record rather than a mutable store.

use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::event::{Actor, ActorType, Event, Provenance, SCHEMA_V0, new_id};

/// Event type that records a proposed assertion.
pub const ASSERTION_RECORDED_EVENT: &str = "promotion.assertion.recorded";
/// Event type that establishes a conclusion with human approval.
pub const CONCLUSION_ESTABLISHED_EVENT: &str = "promotion.conclusion.established";
/// Event type that challenges an established conclusion.
pub const CONCLUSION_CHALLENGED_EVENT: &str = "promotion.conclusion.challenged";

/// The closed set of epistemic categories.
pub const CATEGORY_SET: &str = "{event, assertion, conclusion}";

const LABEL_TOOLS: &str = "kaya.promotion.tools_emit_events_only";
const LABEL_CATEGORY: &str = "kaya.promotion.category_closed";
const LABEL_CHANNEL: &str = "kaya.promotion.proposal_channel";
const LABEL_APPROVAL: &str = "kaya.promotion.approval_required";
const LABEL_PROVENANCE: &str = "kaya.promotion.approval_provenance";
const LABEL_UNKNOWN: &str = "kaya.promotion.unknown_assertion";
const LABEL_CHALLENGE: &str = "kaya.promotion.challenge_target";
const LABEL_CHALLENGE_EVIDENCE: &str = "kaya.promotion.challenge_evidence";
const LABEL_ACTOR: &str = "kaya.promotion.actor_required";
const LABEL_DUPLICATE: &str = "kaya.promotion.duplicate_assertion";

/// A labeled refusal raised by the promotion capability.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PromotionError {
    /// Failure class, e.g. `kaya.promotion.tools_emit_events_only`.
    pub label: String,
    /// Explanation naming the offending value and the rule it broke.
    pub message: String,
}

impl PromotionError {
    /// Build a labeled refusal.
    pub fn new(label: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for PromotionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.label, self.message)
    }
}

impl std::error::Error for PromotionError {}

/// Where an assertion sits in the promotion lifecycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    /// Recorded but not yet approved by a human.
    Proposed,
    /// Approved by a human and displayed as a conclusion.
    Established,
    /// A counter-assertion is open against the conclusion.
    Challenged,
}

impl Status {
    /// Wire and display spelling, one of `proposed`, `established`,
    /// `challenged`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Established => "established",
            Self::Challenged => "challenged",
        }
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One interpretation and its promotion state.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Assertion {
    /// Assertion id; the id of the recording event.
    pub id: String,
    /// The interpretation text.
    pub text: String,
    /// Current lifecycle status.
    pub status: Status,
    /// Whether an agent proposed it (visibly flagged, never established).
    pub machine_proposed: bool,
    /// Events this interpretation cites.
    pub event_ids: Vec<String>,
    /// Name of the authoring human or agent.
    pub author: String,
}

/// The promotion projection over a ledger.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromotionState {
    /// Assertions in recording order.
    pub assertions: Vec<Assertion>,
}

impl PromotionState {
    /// Replay promotion events into state.
    ///
    /// Events are applied in the order given (ledger order). Unknown event
    /// types are ignored; a record whose epistemic category falls outside
    /// `{event, assertion, conclusion}` is refused.
    pub fn from_events(events: &[Event]) -> Result<PromotionState, PromotionError> {
        let mut state = PromotionState::default();
        for event in events {
            check_category(event)?;
            match event.r#type.as_str() {
                ASSERTION_RECORDED_EVENT => state.record_assertion(event)?,
                CONCLUSION_ESTABLISHED_EVENT => state.establish(event)?,
                CONCLUSION_CHALLENGED_EVENT => state.challenge(event)?,
                _ => {}
            }
        }
        Ok(state)
    }

    /// Number of assertions still awaiting human approval.
    pub fn pending_count(&self) -> usize {
        self.assertions
            .iter()
            .filter(|assertion| assertion.status == Status::Proposed)
            .count()
    }

    /// The displayed status of an assertion, when one exists.
    pub fn displayed_status(&self, assertion_id: &str) -> Option<Status> {
        self.assertions
            .iter()
            .find(|assertion| assertion.id == assertion_id)
            .map(|assertion| assertion.status)
    }

    fn record_assertion(&mut self, event: &Event) -> Result<(), PromotionError> {
        if event.actor.r#type == ActorType::Tool {
            return Err(tool_refusal(&event.actor, "record an assertion"));
        }
        let mut missing: Vec<&str> = Vec::new();
        let text = string_field(event, "text").unwrap_or_else(|| {
            missing.push("text");
            ""
        });
        if text.trim().is_empty() && !missing.contains(&"text") {
            missing.push("text");
        }
        let status = string_field(event, "status");
        if status != Some("proposed") {
            missing.push("status");
        }
        let event_ids = string_array_field(event, "event_ids").unwrap_or_default();
        if event_ids.is_empty() {
            missing.push("event_ids");
        }
        if !missing.is_empty() {
            return Err(PromotionError::new(
                LABEL_CHANNEL,
                format!(
                    "assertion event \"{}\" is missing {}; an assertion carries status \"proposed\", its author, and the event ids it interprets",
                    event.id,
                    missing.join(", ")
                ),
            ));
        }
        if event.actor.name.trim().is_empty() {
            return Err(PromotionError::new(
                LABEL_CHANNEL,
                format!(
                    "assertion event \"{}\" has no author name; an assertion carries its author",
                    event.id
                ),
            ));
        }
        let id = assertion_id(event);
        if self.assertions.iter().any(|assertion| assertion.id == id) {
            return Err(PromotionError::new(
                LABEL_DUPLICATE,
                format!("assertion \"{id}\" is already recorded"),
            ));
        }
        self.assertions.push(Assertion {
            id,
            text: text.to_string(),
            status: Status::Proposed,
            machine_proposed: event.actor.r#type == ActorType::Agent,
            event_ids,
            author: event.actor.name.clone(),
        });
        Ok(())
    }

    fn establish(&mut self, event: &Event) -> Result<(), PromotionError> {
        if event.actor.r#type != ActorType::Human {
            return Err(approval_refusal(event));
        }
        let assertion_ref = string_field(event, "assertion");
        let approver = string_field(event, "approver");
        let (assertion_ref, approver) = match (assertion_ref, approver) {
            (Some(assertion_ref), Some(approver))
                if !assertion_ref.is_empty() && !approver.is_empty() =>
            {
                (assertion_ref, approver)
            }
            _ => {
                return Err(PromotionError::new(
                    LABEL_PROVENANCE,
                    format!(
                        "conclusion event \"{}\" must name both the promoted assertion id and its approver",
                        event.id
                    ),
                ));
            }
        };
        let assertion = self
            .assertions
            .iter_mut()
            .find(|assertion| assertion.id == assertion_ref)
            .ok_or_else(|| {
                PromotionError::new(
                    LABEL_UNKNOWN,
                    format!(
                        "conclusion event \"{}\" names assertion \"{assertion_ref}\", which has not been recorded",
                        event.id
                    ),
                )
            })?;
        if approver != event.actor.name {
            return Err(PromotionError::new(
                LABEL_PROVENANCE,
                format!(
                    "conclusion event \"{}\" records approver \"{approver}\" but was emitted by \"{}\"",
                    event.id, event.actor.name
                ),
            ));
        }
        assertion.status = Status::Established;
        Ok(())
    }

    fn challenge(&mut self, event: &Event) -> Result<(), PromotionError> {
        if event.actor.r#type == ActorType::Tool {
            return Err(tool_refusal(&event.actor, "challenge a conclusion"));
        }
        let conclusion_ref = string_field(event, "conclusion").unwrap_or_default();
        let counter_event_ids = string_array_field(event, "counter_event_ids").unwrap_or_default();
        if conclusion_ref.is_empty() || counter_event_ids.is_empty() {
            return Err(PromotionError::new(
                LABEL_CHANNEL,
                format!(
                    "challenge event \"{}\" must name a conclusion and at least one counter-event id",
                    event.id
                ),
            ));
        }
        let assertion = self
            .assertions
            .iter_mut()
            .find(|assertion| assertion.id == conclusion_ref)
            .ok_or_else(|| {
                PromotionError::new(
                    LABEL_CHALLENGE,
                    format!(
                        "challenge event \"{}\" names conclusion \"{conclusion_ref}\", which has not been recorded",
                        event.id
                    ),
                )
            })?;
        if assertion.status != Status::Established {
            return Err(PromotionError::new(
                LABEL_CHALLENGE,
                format!(
                    "conclusion \"{conclusion_ref}\" is {}, not established; only an established conclusion can be challenged",
                    assertion.status
                ),
            ));
        }
        assertion.status = Status::Challenged;
        Ok(())
    }
}

/// Record an interpretation as a `proposed` assertion event.
///
/// Tools may emit events only, so a tool actor is refused. An agent actor
/// yields `machine_proposed: true`; a human actor yields `false`.
pub fn propose(actor: &Actor, text: &str, event_ids: &[String]) -> Result<Event, PromotionError> {
    if actor.r#type == ActorType::Tool {
        return Err(tool_refusal(actor, "propose an assertion"));
    }
    if actor.name.trim().is_empty() {
        return Err(PromotionError::new(
            LABEL_ACTOR,
            "cannot record an assertion without its author: the actor name is blank",
        ));
    }
    let mut missing: Vec<&str> = Vec::new();
    if text.trim().is_empty() {
        missing.push("text");
    }
    if event_ids.is_empty() || event_ids.iter().any(|id| id.trim().is_empty()) {
        missing.push("event_ids");
    }
    if !missing.is_empty() {
        return Err(PromotionError::new(
            LABEL_CHANNEL,
            format!(
                "cannot record an assertion without {}; an assertion carries status \"proposed\", its author, and the events it interprets",
                missing.join(", ")
            ),
        ));
    }
    let id = new_id();
    let machine_proposed = actor.r#type == ActorType::Agent;
    Ok(Event {
        id: id.clone(),
        schema: SCHEMA_V0.to_string(),
        r#type: ASSERTION_RECORDED_EVENT.to_string(),
        timestamp: now_timestamp(),
        actor: actor.clone(),
        subject: crate::event::Subject {
            r#type: "assertion".to_string(),
            id: id.clone(),
        },
        data: json!({
            "text": text,
            "status": Status::Proposed.as_str(),
            "event_ids": event_ids,
            "machine_proposed": machine_proposed,
        }),
        provenance: provenance(&id),
        artifacts: Vec::new(),
    })
}

/// Establish an assertion as a conclusion with recorded human approval.
///
/// Only a human actor can approve; every other actor is refused with
/// `kaya.promotion.approval_required` naming the missing human approval.
pub fn approve(approver: &Actor, assertion_id: &str) -> Result<Event, PromotionError> {
    if approver.r#type != ActorType::Human {
        return Err(approval_refusal_actor(approver, assertion_id));
    }
    if approver.name.trim().is_empty() {
        return Err(PromotionError::new(
            LABEL_ACTOR,
            "cannot record a conclusion without its approver: the actor name is blank",
        ));
    }
    if assertion_id.trim().is_empty() {
        return Err(PromotionError::new(
            LABEL_PROVENANCE,
            "a promotion event must name the promoted assertion id",
        ));
    }
    let id = new_id();
    Ok(Event {
        id: id.clone(),
        schema: SCHEMA_V0.to_string(),
        r#type: CONCLUSION_ESTABLISHED_EVENT.to_string(),
        timestamp: now_timestamp(),
        actor: approver.clone(),
        subject: crate::event::Subject {
            r#type: "conclusion".to_string(),
            id: assertion_id.to_string(),
        },
        data: json!({
            "assertion": assertion_id,
            "approver": approver.name,
        }),
        provenance: provenance(assertion_id),
        artifacts: Vec::new(),
    })
}

/// Challenge an established conclusion with counter-events.
///
/// The challenge is a new record; it changes displayed status only. Tools
/// are refused because no promotion command accepts a tool actor.
pub fn challenge(
    actor: &Actor,
    conclusion_id: &str,
    counter_event_ids: &[String],
) -> Result<Event, PromotionError> {
    if actor.r#type == ActorType::Tool {
        return Err(tool_refusal(actor, "challenge a conclusion"));
    }
    if actor.name.trim().is_empty() {
        return Err(PromotionError::new(
            LABEL_ACTOR,
            "cannot record a challenge without its author: the actor name is blank",
        ));
    }
    if conclusion_id.trim().is_empty() {
        return Err(PromotionError::new(
            LABEL_CHANNEL,
            "a challenge must name the conclusion it counters",
        ));
    }
    if counter_event_ids.is_empty() {
        return Err(PromotionError::new(
            LABEL_CHALLENGE_EVIDENCE,
            "a challenge requires counter evidence; at least one counter-event id must be cited",
        ));
    }
    if counter_event_ids.iter().any(|id| id.trim().is_empty()) {
        return Err(PromotionError::new(
            LABEL_CHALLENGE_EVIDENCE,
            "a challenge cites a blank counter-event id; counter evidence must name the events it counters",
        ));
    }
    let id = new_id();
    Ok(Event {
        id: id.clone(),
        schema: SCHEMA_V0.to_string(),
        r#type: CONCLUSION_CHALLENGED_EVENT.to_string(),
        timestamp: now_timestamp(),
        actor: actor.clone(),
        subject: crate::event::Subject {
            r#type: "conclusion".to_string(),
            id: conclusion_id.to_string(),
        },
        data: json!({
            "conclusion": conclusion_id,
            "counter_event_ids": counter_event_ids,
        }),
        provenance: provenance(conclusion_id),
        artifacts: Vec::new(),
    })
}

/// Rank pending proposals and keep at most `budget` of them.
///
/// Ranking is deterministic: more cited events first, then assertion id
/// ascending. Only `proposed` assertions are candidates; established and
/// challenged conclusions are never returned. An unapproved proposal stays
/// `proposed` indefinitely — this function caps generation, it does not
/// settle or expire anything.
pub fn cap_proposals(assertions: &[Assertion], budget: usize) -> Vec<Assertion> {
    let mut pending: Vec<Assertion> = assertions
        .iter()
        .filter(|assertion| assertion.status == Status::Proposed)
        .cloned()
        .collect();
    pending.sort_by(|left, right| {
        right
            .event_ids
            .len()
            .cmp(&left.event_ids.len())
            .then_with(|| left.id.cmp(&right.id))
    });
    pending.truncate(budget);
    pending
}

fn check_category(event: &Event) -> Result<(), PromotionError> {
    let Some(category) = event.data.get("category").and_then(Value::as_str) else {
        return Ok(());
    };
    if matches!(category, "event" | "assertion" | "conclusion") {
        return Ok(());
    }
    Err(PromotionError::new(
        LABEL_CATEGORY,
        format!(
            "event \"{}\" carries epistemic category \"{category}\"; the closed set is {CATEGORY_SET}",
            event.id
        ),
    ))
}

fn tool_refusal(actor: &Actor, attempted: &str) -> PromotionError {
    PromotionError::new(
        LABEL_TOOLS,
        format!(
            "actor \"{}\" is a tool and cannot {attempted}; tools may write events only — assertion and conclusion records are human/agent authored",
            actor.name
        ),
    )
}

fn approval_refusal(event: &Event) -> PromotionError {
    PromotionError::new(
        LABEL_APPROVAL,
        format!(
            "conclusion event \"{}\" was emitted by {} actor \"{}\"; approval requires a promotion event whose actor type is human",
            event.id,
            event.actor.r#type.as_str(),
            event.actor.name
        ),
    )
}

fn approval_refusal_actor(actor: &Actor, assertion_id: &str) -> PromotionError {
    PromotionError::new(
        LABEL_APPROVAL,
        format!(
            "approving assertion \"{assertion_id}\" requires a human actor; actor \"{}\" is {} and no human approval is recorded",
            actor.name,
            actor.r#type.as_str()
        ),
    )
}

fn assertion_id(event: &Event) -> String {
    if event.subject.id.is_empty() {
        event.id.clone()
    } else {
        event.subject.id.clone()
    }
}

fn string_field<'a>(event: &'a Event, key: &str) -> Option<&'a str> {
    event.data.get(key).and_then(Value::as_str)
}

fn string_array_field(event: &Event, key: &str) -> Option<Vec<String>> {
    let values = event.data.get(key)?.as_array()?;
    let mut strings = Vec::with_capacity(values.len());
    for value in values {
        strings.push(value.as_str()?.to_string());
    }
    Some(strings)
}

fn provenance(reference: &str) -> Provenance {
    Provenance {
        tool: "kaya".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        reference: reference.to_string(),
    }
}

fn now_timestamp() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}
