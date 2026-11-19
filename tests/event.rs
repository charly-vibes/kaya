//! Contract tests for the `kaya.event/v0` line codec.

use kaya::event::{
    Actor, ActorType, Event, Provenance, SCHEMA_V0, Subject, new_id, parse_line, to_line,
};
use serde_json::{Map, Value, json};

fn record_with(data: &str) -> String {
    format!(
        concat!(
            "{{\"id\":\"01JAV3Z0Y6W4Z5X8B7C6D5E4F3\",",
            "\"schema\":\"kaya.event/v0\",",
            "\"type\":\"experiment.run.completed\",",
            "\"timestamp\":\"2026-11-16T10:00:00Z\",",
            "\"actor\":{{\"type\":\"tool\",\"name\":\"cargo-test\"}},",
            "\"subject\":{{\"type\":\"Run\",\"id\":\"kaya/Run/RUN-001\"}},",
            "\"data\":{data},",
            "\"provenance\":{{\"tool\":\"kaya\",\"version\":\"0.1.0\",\"reference\":\"commit:abc123\"}},",
            "\"artifacts\":[\"reports/run.json\"]}}\n",
        ),
        data = data
    )
}

fn record() -> String {
    record_with(r#"{"passed":3}"#)
}

fn edited(edit: impl FnOnce(&mut Map<String, Value>)) -> String {
    let mut value: Value = serde_json::from_str(&record()).expect("fixture is valid JSON");
    edit(value.as_object_mut().expect("fixture is a JSON object"));
    value.to_string()
}

#[test]
fn well_formed_record_round_trips_byte_identically() {
    let line = record();
    let event = parse_line(&line).expect("well-formed record parses");
    let written = to_line(&event).expect("event data is finite");
    assert_eq!(written, line);
    assert_eq!(
        parse_line(&written).expect("serialized record parses"),
        event
    );
    assert_eq!(
        to_line(&parse_line(&written).unwrap()).expect("event data is finite"),
        written
    );

    // The canonical line is newline-terminated; an unterminated line is still
    // accepted on input and canonicalized on the way out.
    let unterminated = line.trim_end_matches('\n');
    assert_eq!(
        parse_line(unterminated).expect("unterminated line parses"),
        event
    );
    assert_eq!(
        to_line(&parse_line(unterminated).unwrap()).expect("event data is finite"),
        line
    );
}

#[test]
fn well_formed_record_validates_every_field() {
    let event = parse_line(&record()).expect("well-formed record parses");
    assert_eq!(event.id, "01JAV3Z0Y6W4Z5X8B7C6D5E4F3");
    assert_eq!(event.schema, SCHEMA_V0);
    assert_eq!(event.r#type, "experiment.run.completed");
    assert_eq!(event.timestamp, "2026-11-16T10:00:00Z");
    assert_eq!(event.actor.r#type, ActorType::Tool);
    assert_eq!(event.actor.name, "cargo-test");
    assert_eq!(event.subject.r#type, "Run");
    assert_eq!(event.subject.id, "kaya/Run/RUN-001");
    assert_eq!(event.data, json!({"passed": 3}));
    assert_eq!(event.provenance.tool, "kaya");
    assert_eq!(event.provenance.version, "0.1.0");
    assert_eq!(event.provenance.reference, "commit:abc123");
    assert_eq!(event.artifacts, vec!["reports/run.json".to_owned()]);
}

#[test]
fn unknown_schema_version_is_refused() {
    let line = edited(|record| {
        record.insert("schema".to_owned(), json!("kaya.event/v9"));
    });
    let err = parse_line(&line).expect_err("unknown schema version is refused");
    assert_eq!(err.label, "kaya.event.schema_version_pinned");
    assert!(err.message.contains("kaya.event/v9"), "{}", err.message);
    assert!(err.message.contains("kaya.event/v0"), "{}", err.message);
    assert_eq!(err.to_string(), format!("{}: {}", err.label, err.message));
}

#[test]
fn missing_provenance_is_refused_naming_provenance() {
    let line = edited(|record| {
        record.remove("provenance");
    });
    let err = parse_line(&line).expect_err("record without provenance is refused");
    assert_eq!(err.label, "kaya.event.field_set_closed");
    assert!(err.message.contains("provenance"), "{}", err.message);
    assert!(err.message.contains("missing"), "{}", err.message);
}

#[test]
fn missing_any_required_field_is_refused_by_name() {
    let err = parse_line(&edited(|record| {
        record.remove("artifacts");
    }))
    .expect_err("record without artifacts is refused");
    assert_eq!(err.label, "kaya.event.field_set_closed");
    assert!(err.message.contains("artifacts"), "{}", err.message);
}

#[test]
fn extra_field_is_refused_naming_it() {
    let err = parse_line(&edited(|record| {
        record.insert("note".to_owned(), json!("added by hand"));
    }))
    .expect_err("record with an extra field is refused");
    assert_eq!(err.label, "kaya.event.field_set_closed");
    assert!(err.message.contains("note"), "{}", err.message);
    assert!(err.message.contains("closed"), "{}", err.message);
}

#[test]
fn bare_and_short_types_are_refused() {
    for bad in [
        "created",
        "run.completed",
        "experiment.run",
        "Experiment.Run.Completed",
        "experiment..completed",
        "experiment.run.Completed",
    ] {
        let line = edited(|record| {
            record.insert("type".to_owned(), json!(bad));
        });
        let err = parse_line(&line).expect_err(&format!("type {bad} should be refused"));
        assert_eq!(err.label, "kaya.event.type_namespaced", "type {bad}");
        assert!(err.message.contains(bad), "{}", err.message);
        assert!(
            err.message.contains("experiment.run.completed"),
            "{}",
            err.message
        );
        assert!(
            err.message.contains("verification.acknowledged"),
            "the refusal names the single exception: {}",
            err.message
        );
    }
}

#[test]
fn namespaced_lowercase_types_are_accepted() {
    for good in [
        "experiment.run.completed",
        "model.entity.created",
        "verification.acknowledged",
    ] {
        let line = edited(|record| {
            record.insert("type".to_owned(), json!(good));
        });
        let event = parse_line(&line).unwrap_or_else(|_| panic!("type {good} should parse"));
        assert_eq!(event.r#type, good);
    }
}

#[test]
fn non_utc_and_loose_timestamps_are_refused() {
    for bad in [
        "2026-11-16T10:00:00+02:00",
        "2026-11-16 10:00:00",
        "2026-11-16T10:00:00",
        "16/11/2026",
        "yesterday",
    ] {
        let line = edited(|record| {
            record.insert("timestamp".to_owned(), json!(bad));
        });
        let err = parse_line(&line).expect_err(&format!("timestamp {bad} should be refused"));
        assert_eq!(err.label, "kaya.event.timestamp_rfc3339", "timestamp {bad}");
        assert!(err.message.contains(bad), "{}", err.message);
        assert!(err.message.contains("RFC 3339"), "{}", err.message);
    }
}

#[test]
fn utc_timestamps_with_explicit_zero_offset_are_accepted() {
    let line = edited(|record| {
        record.insert("timestamp".to_owned(), json!("2026-11-16T10:00:00+00:00"));
    });
    let event = parse_line(&line).expect("UTC +00:00 is RFC 3339 UTC");
    assert_eq!(event.timestamp, "2026-11-16T10:00:00+00:00");
}

#[test]
fn interpretation_fields_are_refused_pointing_at_promotion() {
    for key in [
        "verdict",
        "conclusion",
        "interpretation",
        "hypothesis_verdict",
        "hypothesis_status",
    ] {
        let data = format!("{{\"{key}\":\"hypothesis-false\"}}");
        let err =
            parse_line(&record_with(&data)).expect_err(&format!("data.{key} should be refused"));
        assert_eq!(err.label, "kaya.event.observation_only", "data.{key}");
        assert!(err.message.contains(key), "{}", err.message);
        assert!(err.message.contains("kaya.promotion"), "{}", err.message);
    }
}

#[test]
fn promotion_records_may_carry_interpretation_fields() {
    let line = concat!(
        "{\"id\":\"conclusion-1\",\"schema\":\"kaya.event/v0\",",
        "\"type\":\"promotion.conclusion.established\",",
        "\"timestamp\":\"2026-11-16T10:00:00Z\",",
        "\"actor\":{\"type\":\"human\",\"name\":\"alice\"},",
        "\"subject\":{\"type\":\"conclusion\",\"id\":\"assertion-1\"},",
        "\"data\":{\"assertion\":\"assertion-1\",\"conclusion\":\"H-001 partially supported\",\"approver\":\"alice\"},",
        "\"provenance\":{\"tool\":\"kaya\",\"version\":\"0.1.0\",\"reference\":\"assertion-1\"},",
        "\"artifacts\":[]}\n"
    );
    let event = parse_line(line).expect("promotion records are the interpretation channel");
    assert_eq!(event.r#type, "promotion.conclusion.established");
    assert_eq!(
        event.data["conclusion"],
        serde_json::json!("H-001 partially supported")
    );
}

#[test]
fn non_object_data_is_refused() {
    for bad in ["[]", "\"observations\"", "3", "null", "true"] {
        let err = parse_line(&record_with(bad)).expect_err(&format!("data {bad} refused"));
        assert_eq!(err.label, "kaya.event.data_object", "data {bad}");
        assert!(err.message.contains("data"), "{}", err.message);
    }
}

#[test]
fn non_string_artifacts_are_refused() {
    let err = parse_line(&edited(|record| {
        record.insert("artifacts".to_owned(), json!(["reports/run.json", 7]));
    }))
    .expect_err("non-string artifact is refused");
    assert_eq!(err.label, "kaya.event.artifacts_strings");
    assert!(err.message.contains("artifacts[1]"), "{}", err.message);
}

#[test]
fn empty_actor_subject_and_provenance_parts_are_refused() {
    let cases: [(&str, &str, &str, &str); 3] = [
        ("actor", "name", "kaya.event.actor_present", "actor.name"),
        ("subject", "id", "kaya.event.subject_present", "subject.id"),
        (
            "provenance",
            "reference",
            "kaya.event.provenance_present",
            "provenance.reference",
        ),
    ];
    for (owner, field, label, named) in cases {
        let err = parse_line(&edited(|record| {
            let nested = record
                .get_mut(owner)
                .and_then(Value::as_object_mut)
                .expect("fixture nested object");
            nested.insert(field.to_owned(), json!(""));
        }))
        .expect_err(&format!("empty {owner}.{field} should be refused"));
        assert_eq!(err.label, label, "empty {owner}.{field}");
        assert!(err.message.contains(named), "{}", err.message);
    }
}

#[test]
fn unknown_actor_kind_is_refused_naming_the_closed_set() {
    let err = parse_line(&edited(|record| {
        let actor = record
            .get_mut("actor")
            .and_then(Value::as_object_mut)
            .expect("fixture actor");
        actor.insert("type".to_owned(), json!("robot"));
    }))
    .expect_err("unknown actor kind is refused");
    assert_eq!(err.label, "kaya.event.actor_present");
    assert!(err.message.contains("robot"), "{}", err.message);
    assert!(
        err.message.contains("tool, human, agent"),
        "{}",
        err.message
    );
}

#[test]
fn non_object_line_is_refused() {
    let err = parse_line("[]").expect_err("array line is refused");
    assert_eq!(err.label, "kaya.event.record_object");
}

#[test]
fn actor_kind_serializes_lowercase() {
    let event = parse_line(&record()).expect("well-formed record parses");
    let line = to_line(&event).expect("event data is finite");
    assert!(
        line.contains(r#""actor":{"type":"tool","name":"cargo-test"}"#),
        "{line}"
    );
    assert!(line.ends_with('\n'));
    assert!(!line.contains("Tool"));
    assert_eq!(ActorType::Human.as_str(), "human");

    let explicit = Event {
        id: "01JAV3Z0Y6W4Z5X8B7C6D5E4F9".to_owned(),
        schema: SCHEMA_V0.to_owned(),
        r#type: "model.entity.created".to_owned(),
        timestamp: "2026-11-16T10:00:00Z".to_owned(),
        actor: Actor {
            r#type: ActorType::Agent,
            name: "agent-1".to_owned(),
        },
        subject: Subject {
            r#type: "Model".to_owned(),
            id: "kaya/Model/MODEL-001".to_owned(),
        },
        data: json!({"entities": 2}),
        provenance: Provenance {
            tool: "kaya".to_owned(),
            version: "0.1.0".to_owned(),
            reference: "session:abc".to_owned(),
        },
        artifacts: Vec::new(),
    };
    let round_tripped =
        parse_line(&to_line(&explicit).expect("event data is finite")).expect("agent event parses");
    assert_eq!(round_tripped, explicit);
    assert_eq!(round_tripped.actor.r#type, ActorType::Agent);
}

#[test]
fn new_ids_are_distinct_ulids() {
    let first = new_id();
    let second = new_id();
    assert_eq!(first.len(), 26, "{first}");
    assert_eq!(second.len(), 26, "{second}");
    assert_ne!(first, second);
    // Crockford base32: uppercase alphanumerics minus I, L, O, U.
    assert!(
        first
            .chars()
            .all(|c| c.is_ascii_digit() || c.is_ascii_uppercase()),
        "{first}"
    );
    assert!(!first.chars().any(|c| "ILOU".contains(c)), "{first}");
}

#[test]
fn duplicate_top_level_key_is_refused() {
    let line = record().replacen(
        "\"type\":\"experiment.run.completed\"",
        "\"type\":\"experiment.run.completed\",\"type\":\"experiment.run.completed\"",
        1,
    );
    let err = parse_line(&line).expect_err("duplicate top-level key is refused");
    assert_eq!(err.label, "kaya.event.duplicate_key");
    assert!(
        err.message.contains("\"type\""),
        "message names the offending key: {}",
        err.message
    );
}

#[test]
fn duplicate_nested_data_key_is_refused() {
    let line = record_with(r#"{"passed":3,"passed":4}"#);
    let err = parse_line(&line).expect_err("duplicate data key is refused");
    assert_eq!(err.label, "kaya.event.duplicate_key");
    assert!(
        err.message.contains("\"passed\""),
        "message names the offending key: {}",
        err.message
    );
    assert!(
        err.message.contains("data"),
        "message names the object path: {}",
        err.message
    );
}

#[test]
fn duplicate_key_deep_in_data_is_refused() {
    let line = record_with(r#"{"nested":{"same":1,"same":2}}"#);
    let err = parse_line(&line).expect_err("duplicate key at depth is refused");
    assert_eq!(err.label, "kaya.event.duplicate_key");
    assert!(
        err.message.contains("\"same\"") && err.message.contains("data.nested"),
        "message names key and path: {}",
        err.message
    );
}

#[test]
fn escaped_key_lookalike_is_not_a_duplicate() {
    // The JSON text "a\n" is the key "a" followed by a newline, not "a".
    let line = record_with(r#"{"a":"one","a\n":"two"}"#);
    let event = parse_line(&line).expect("escaped-key lookalike parses");
    assert_eq!(
        event.data.get("a"),
        Some(&json!("one")),
        "the plain key keeps its own value"
    );
    assert_eq!(
        event.data.get("a\n"),
        Some(&json!("two")),
        "the newline-suffixed key is distinct, not a duplicate"
    );
}

#[test]
fn non_finite_floats_cannot_reach_a_write() {
    // serde_json's `Value` cannot carry NaN or an infinity: `Number::from_f64`
    // refuses them, `json!` collapses them to null before any write can see
    // them, and the nearest JSON text spelling is refused at parse. The
    // write-side `kaya.event.non_finite` guard in `to_line` covers the day a
    // non-finite number becomes representable (e.g. the arbitrary_precision
    // feature); today the parse side holds the boundary.
    assert_eq!(serde_json::Number::from_f64(f64::NAN), None);
    let line = record_with(r#"{"ratio": 1e999}"#);
    let err = parse_line(&line).expect_err("overflowing float literal is refused");
    assert_eq!(err.label, "kaya.event.json");
}

#[test]
fn finite_float_data_round_trips_stably() {
    let line = record_with(r#"{"big":1e300,"ratio":0.3333333333333333}"#);
    let event = parse_line(&line).expect("finite float data parses");
    let canonical = to_line(&event).expect("event data is finite");
    // serde_json's shortest round-trip (ryu) representation is unchanged
    // by a parse/serialize cycle.
    assert!(
        canonical.contains(r#""ratio":0.3333333333333333"#)
            && canonical.contains(r#""big":1e+300"#),
        "finite floats keep their shortest representation: {canonical}"
    );
    let again = to_line(&parse_line(&canonical).expect("canonical line parses"))
        .expect("event data is finite");
    assert_eq!(again, canonical, "serialization is idempotent");
}

#[test]
fn escaped_spelled_duplicate_key_is_refused() {
    // One occurrence of the key is spelled with an escape sequence; the
    // decoded keys are equal, so the line must still be refused.
    let line = record_with(r#"{"pa\u0073sed":3,"passed":4}"#);
    let err = parse_line(&line).expect_err("escaped duplicate key is refused");
    assert_eq!(err.label, "kaya.event.duplicate_key");
    assert!(
        err.message.contains("\"passed\""),
        "message names the offending key: {}",
        err.message
    );
    assert!(
        err.message.contains("data"),
        "message names the object path: {}",
        err.message
    );
}

#[test]
fn duplicate_key_inside_array_element_names_the_index() {
    let line = record_with(r#"{"items":[{"same":1,"same":2}]}"#);
    let err = parse_line(&line).expect_err("duplicate key in array element is refused");
    assert_eq!(err.label, "kaya.event.duplicate_key");
    assert!(
        err.message.contains("\"same\""),
        "message names the offending key: {}",
        err.message
    );
    assert!(
        err.message.contains("data.items[0]"),
        "message names the array index: {}",
        err.message
    );
}
