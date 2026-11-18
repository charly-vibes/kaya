//! `kaya.event/v0` record schema and JSONL line codec.
//!
//! Purpose: one versioned, closed-field record format for what a tool, a
//! human, or an agent observed.
//! Responsibilities: validate a raw ledger line into an [`Event`],
//! canonicalize an event back to bytes, and mint fresh ULID ids.
//! Rationale: a closed field set plus a pinned schema makes records
//! generatable from any language and diffable in git, while refusing
//! interpretation keys keeps hypothesis verdicts in `kaya.promotion` where
//! they can be argued rather than silently asserted.

use std::fmt;

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// The one event schema version this ingest accepts.
pub const SCHEMA_V0: &str = "kaya.event/v0";

/// The closed field set, in canonical serialization order.
const EVENT_FIELDS: [&str; 9] = [
    "id",
    "schema",
    "type",
    "timestamp",
    "actor",
    "subject",
    "data",
    "provenance",
    "artifacts",
];

/// Data keys that record interpretation rather than observation.
const INTERPRETATION_KEYS: [&str; 5] = [
    "verdict",
    "conclusion",
    "interpretation",
    "hypothesis_verdict",
    "hypothesis_status",
];

/// Actor kinds the schema names.
const ACTOR_KINDS: [&str; 3] = ["tool", "human", "agent"];

const ACTOR_FIELDS: [&str; 2] = ["type", "name"];
const SUBJECT_FIELDS: [&str; 2] = ["type", "id"];
const PROVENANCE_FIELDS: [&str; 3] = ["tool", "version", "reference"];

const LABEL_JSON: &str = "kaya.event.json";
const LABEL_RECORD_OBJECT: &str = "kaya.event.record_object";
const LABEL_FIELD_SET: &str = "kaya.event.field_set_closed";
const LABEL_VALUE_TYPE: &str = "kaya.event.value_type";
const LABEL_SCHEMA: &str = "kaya.event.schema_version_pinned";
const LABEL_TYPE: &str = "kaya.event.type_namespaced";
const LABEL_TIMESTAMP: &str = "kaya.event.timestamp_rfc3339";
const LABEL_ACTOR: &str = "kaya.event.actor_present";
const LABEL_SUBJECT: &str = "kaya.event.subject_present";
const LABEL_PROVENANCE: &str = "kaya.event.provenance_present";
const LABEL_DATA: &str = "kaya.event.data_object";
const LABEL_ARTIFACTS: &str = "kaya.event.artifacts_strings";
const LABEL_OBSERVATION: &str = "kaya.event.observation_only";

/// A labeled refusal raised while validating a `kaya.event/v0` record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EventError {
    /// Failure class, e.g. `kaya.event.schema_version_pinned`.
    pub label: String,
    /// Explanation naming the offending value and the rule it broke.
    pub message: String,
}

impl EventError {
    /// Build a labeled refusal.
    pub fn new(label: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for EventError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.label, self.message)
    }
}

impl std::error::Error for EventError {}

/// The kind of source that emitted an event.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActorType {
    /// A program emitted the observation.
    Tool,
    /// A person emitted the observation.
    Human,
    /// A model-driven agent emitted the observation.
    Agent,
}

impl ActorType {
    /// Wire spelling of this actor kind.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Tool => "tool",
            Self::Human => "human",
            Self::Agent => "agent",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value {
            "tool" => Some(Self::Tool),
            "human" => Some(Self::Human),
            "agent" => Some(Self::Agent),
            _ => None,
        }
    }
}

/// Who emitted an event: a closed kind plus a name.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Actor {
    /// Kind of emitter (`tool`, `human`, or `agent`).
    #[serde(rename = "type")]
    pub r#type: ActorType,
    /// Name of the emitting tool, person, or agent.
    pub name: String,
}

/// What an event is about.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Subject {
    /// Entity type of the subject.
    pub r#type: String,
    /// Id of the subject, resolvable in the research graph.
    pub id: String,
}

/// Where an event came from, resolvable outside the ledger.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Provenance {
    /// Emitting tool or human origin.
    pub tool: String,
    /// Version of that emitter.
    pub version: String,
    /// Commit, artifact path, or session id locating the observation.
    pub reference: String,
}

/// One admitted observation over the closed `kaya.event/v0` field set.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
    /// ULID identifying this event within its ledger.
    pub id: String,
    /// Schema version; always [`SCHEMA_V0`] for admitted records.
    pub schema: String,
    /// Namespaced event type, e.g. `experiment.run.completed`.
    #[serde(rename = "type")]
    pub r#type: String,
    /// RFC 3339 UTC timestamp of the observation.
    pub timestamp: String,
    /// Emitter of the event.
    pub actor: Actor,
    /// Subject the event is about.
    pub subject: Subject,
    /// Observed facts (counts, paths, references), never interpretation.
    pub data: Value,
    /// Evidence locating the observation outside the ledger.
    pub provenance: Provenance,
    /// References to artifacts produced alongside the observation.
    pub artifacts: Vec<String>,
}

/// Validate one JSONL line as a `kaya.event/v0` record.
///
/// A trailing newline is tolerated so [`to_line`] output parses back.
pub fn parse_line(line: &str) -> Result<Event, EventError> {
    let text = line.trim();
    if text.is_empty() {
        return Err(EventError::new(
            LABEL_JSON,
            "empty line: a ledger line is exactly one kaya.event/v0 JSON object",
        ));
    }
    let value: Value = serde_json::from_str(text)
        .map_err(|err| EventError::new(LABEL_JSON, format!("line is not valid JSON: {err}")))?;
    let Value::Object(map) = &value else {
        return Err(EventError::new(
            LABEL_RECORD_OBJECT,
            format!("line must be a JSON object, found {}", json_kind(&value)),
        ));
    };
    check_field_set(map)?;

    let schema = require_string(map, "schema")?;
    if schema != SCHEMA_V0 {
        return Err(EventError::new(
            LABEL_SCHEMA,
            format!(
                "schema \"{schema}\" is unsupported; the supported set is {{{SCHEMA_V0}}} — the schema grows only by a new revision"
            ),
        ));
    }

    let id = require_string(map, "id")?;
    let event_type = require_string(map, "type")?;
    if !is_namespaced(event_type) {
        return Err(EventError::new(
            LABEL_TYPE,
            format!(
                "type \"{event_type}\" is not namespaced; the type grammar is at least three dotted lowercase segments <domain>.<subject>.<outcome>, for example experiment.run.completed — the single two-segment exception is verification.acknowledged, the human acknowledgment record"
            ),
        ));
    }

    let timestamp = require_string(map, "timestamp")?;
    check_timestamp(timestamp)?;

    let actor = parse_actor(map.get("actor"))?;
    let subject = parse_subject(map.get("subject"))?;
    let data = parse_data(map.get("data"), event_type)?;
    let provenance = parse_provenance(map.get("provenance"))?;
    let artifacts = parse_artifacts(map.get("artifacts"))?;

    Ok(Event {
        id: id.to_owned(),
        schema: schema.to_owned(),
        r#type: event_type.to_owned(),
        timestamp: timestamp.to_owned(),
        actor,
        subject,
        data,
        provenance,
        artifacts,
    })
}

/// Serialize an event as one compact JSON line terminated by `\n`.
///
/// Field order follows the declaration order of [`Event`], so equal events
/// always produce equal bytes.
pub fn to_line(event: &Event) -> String {
    // `Event` holds only strings, arrays, and JSON values, so serialization
    // has no failing path; if it ever did, the empty result is refused by
    // `parse_line` rather than admitting a half-written record.
    match serde_json::to_string(event) {
        Ok(mut line) => {
            line.push('\n');
            line
        }
        Err(_) => String::new(),
    }
}

/// Mint a fresh ULID for a new event.
pub fn new_id() -> String {
    ulid::Ulid::new().to_string()
}

fn check_field_set(map: &Map<String, Value>) -> Result<(), EventError> {
    for field in EVENT_FIELDS {
        if !map.contains_key(field) {
            return Err(EventError::new(
                LABEL_FIELD_SET,
                format!(
                    "record is missing required field \"{field}\"; the kaya.event/v0 field set is exactly {{{}}}, closed against additions",
                    EVENT_FIELDS.join(", ")
                ),
            ));
        }
    }
    let mut extras: Vec<&str> = map
        .keys()
        .map(String::as_str)
        .filter(|key| !EVENT_FIELDS.contains(key))
        .collect();
    extras.sort_unstable();
    if let Some(field) = extras.first() {
        return Err(EventError::new(
            LABEL_FIELD_SET,
            format!(
                "record carries unexpected field \"{field}\"; the kaya.event/v0 field set is closed to {{{}}}, which grows only by a new schema revision",
                EVENT_FIELDS.join(", ")
            ),
        ));
    }
    Ok(())
}

fn require_string<'a>(map: &'a Map<String, Value>, field: &str) -> Result<&'a str, EventError> {
    match map.get(field) {
        Some(Value::String(text)) if !text.trim().is_empty() => Ok(text.as_str()),
        Some(Value::String(_)) => Err(EventError::new(
            LABEL_VALUE_TYPE,
            format!("field \"{field}\" is empty; it must be a non-empty string"),
        )),
        Some(other) => Err(EventError::new(
            LABEL_VALUE_TYPE,
            format!(
                "field \"{field}\" must be a string, found {}",
                json_kind(other)
            ),
        )),
        None => Err(EventError::new(
            LABEL_FIELD_SET,
            format!("record is missing required field \"{field}\""),
        )),
    }
}

fn check_timestamp(value: &str) -> Result<(), EventError> {
    let parsed: DateTime<FixedOffset> = DateTime::parse_from_rfc3339(value).map_err(|err| {
        EventError::new(
            LABEL_TIMESTAMP,
            format!(
                "timestamp \"{value}\" is not RFC 3339: {err}; kaya.event/v0 timestamps are RFC 3339 UTC, for example 2026-11-16T10:00:00Z"
            ),
        )
    })?;
    if parsed.offset().local_minus_utc() != 0 {
        return Err(EventError::new(
            LABEL_TIMESTAMP,
            format!(
                "timestamp \"{value}\" carries offset {}; kaya.event/v0 timestamps are RFC 3339 UTC (Z or +00:00)",
                parsed.offset()
            ),
        ));
    }
    Ok(())
}

fn parse_actor(value: Option<&Value>) -> Result<Actor, EventError> {
    let fields = nested_object(value, "actor", LABEL_ACTOR, &ACTOR_FIELDS)?;
    let raw_type = nested_string(fields, "actor", "type", LABEL_ACTOR)?;
    let Some(actor_type) = ActorType::parse(raw_type) else {
        return Err(EventError::new(
            LABEL_ACTOR,
            format!(
                "actor.type \"{raw_type}\" is not one of the actor kinds {{{}}}; kaya.event/v0 actors are {}",
                ACTOR_KINDS.join(", "),
                ACTOR_KINDS.join(", ")
            ),
        ));
    };
    let name = nested_string(fields, "actor", "name", LABEL_ACTOR)?;
    Ok(Actor {
        r#type: actor_type,
        name: name.to_owned(),
    })
}

fn parse_subject(value: Option<&Value>) -> Result<Subject, EventError> {
    let fields = nested_object(value, "subject", LABEL_SUBJECT, &SUBJECT_FIELDS)?;
    let subject_type = nested_string(fields, "subject", "type", LABEL_SUBJECT)?;
    let id = nested_string(fields, "subject", "id", LABEL_SUBJECT)?;
    Ok(Subject {
        r#type: subject_type.to_owned(),
        id: id.to_owned(),
    })
}

fn parse_provenance(value: Option<&Value>) -> Result<Provenance, EventError> {
    let fields = nested_object(value, "provenance", LABEL_PROVENANCE, &PROVENANCE_FIELDS)?;
    let tool = nested_string(fields, "provenance", "tool", LABEL_PROVENANCE)?;
    let version = nested_string(fields, "provenance", "version", LABEL_PROVENANCE)?;
    let reference = nested_string(fields, "provenance", "reference", LABEL_PROVENANCE)?;
    Ok(Provenance {
        tool: tool.to_owned(),
        version: version.to_owned(),
        reference: reference.to_owned(),
    })
}

fn parse_data(value: Option<&Value>, event_type: &str) -> Result<Value, EventError> {
    let Some(Value::Object(map)) = value else {
        return Err(EventError::new(
            LABEL_DATA,
            format!(
                "\"data\" must be a JSON object of observations, found {}",
                value.map_or("nothing", json_kind)
            ),
        ));
    };
    // `promotion.*` records are the sanctioned interpretation channel; the
    // observation-only guard applies to capture events, which must never
    // carry a verdict.
    if !event_type.starts_with("promotion.") {
        for key in INTERPRETATION_KEYS {
            if map.contains_key(key) {
                return Err(EventError::new(
                    LABEL_OBSERVATION,
                    format!(
                        "data.{key} records an interpretation; kaya.event/v0 events carry observations only — record conclusions and hypothesis verdicts with the kaya.promotion command"
                    ),
                ));
            }
        }
    }
    Ok(Value::Object(map.clone()))
}

fn parse_artifacts(value: Option<&Value>) -> Result<Vec<String>, EventError> {
    let Some(Value::Array(items)) = value else {
        return Err(EventError::new(
            LABEL_ARTIFACTS,
            format!(
                "\"artifacts\" must be an array of strings, found {}",
                value.map_or("nothing", json_kind)
            ),
        ));
    };
    let mut artifacts = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        let Value::String(text) = item else {
            return Err(EventError::new(
                LABEL_ARTIFACTS,
                format!(
                    "artifacts[{index}] must be a string, found {}",
                    json_kind(item)
                ),
            ));
        };
        artifacts.push(text.clone());
    }
    Ok(artifacts)
}

fn nested_object<'a>(
    value: Option<&'a Value>,
    owner: &str,
    label: &str,
    fields: &[&str],
) -> Result<&'a Map<String, Value>, EventError> {
    let Some(Value::Object(map)) = value else {
        return Err(EventError::new(
            label,
            format!(
                "\"{owner}\" must be a JSON object with exactly {{{}}}, found {}",
                fields.join(", "),
                value.map_or("nothing", json_kind)
            ),
        ));
    };
    for field in fields {
        if !map.contains_key(*field) {
            return Err(EventError::new(
                label,
                format!(
                    "\"{owner}\" is missing \"{field}\"; {owner} carries exactly {{{}}}",
                    fields.join(", ")
                ),
            ));
        }
    }
    let mut extras: Vec<&str> = map
        .keys()
        .map(String::as_str)
        .filter(|key| !fields.contains(key))
        .collect();
    extras.sort_unstable();
    if let Some(field) = extras.first() {
        return Err(EventError::new(
            LABEL_FIELD_SET,
            format!(
                "\"{owner}\" carries unexpected field \"{field}\"; {owner} carries exactly {{{}}}",
                fields.join(", ")
            ),
        ));
    }
    Ok(map)
}

fn nested_string<'a>(
    map: &'a Map<String, Value>,
    owner: &str,
    field: &str,
    label: &str,
) -> Result<&'a str, EventError> {
    match map.get(field) {
        Some(Value::String(text)) if !text.trim().is_empty() => Ok(text.as_str()),
        Some(Value::String(_)) => Err(EventError::new(
            label,
            format!("\"{owner}.{field}\" is empty; every {owner} part must be a non-empty string"),
        )),
        Some(other) => Err(EventError::new(
            label,
            format!(
                "\"{owner}.{field}\" must be a string, found {}",
                json_kind(other)
            ),
        )),
        None => Err(EventError::new(
            label,
            format!("\"{owner}\" is missing \"{field}\""),
        )),
    }
}

fn is_namespaced(event_type: &str) -> bool {
    // The type grammar is at least three dotted lowercase segments,
    // <domain>.<subject>.<outcome>. The single exception is the acknowledgment
    // requirement's `verification.acknowledged`: a two-segment
    // human-verification record with no finer outcome to name.
    if event_type == "verification.acknowledged" {
        return true;
    }
    let segments: Vec<&str> = event_type.split('.').collect();
    segments.len() >= 3 && segments.iter().copied().all(is_lower_segment)
}

fn is_lower_segment(segment: &str) -> bool {
    let mut chars = segment.chars();
    match chars.next() {
        Some(first) if first.is_ascii_lowercase() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

fn json_kind(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "a boolean",
        Value::Number(_) => "a number",
        Value::String(_) => "a string",
        Value::Array(_) => "an array",
        Value::Object(_) => "an object",
    }
}
