//! Hegel property tests for the kaya.event/v0 line codec.
//!
//! Two properties, scoped to the well-formed domain pinned by
//! antithesis/scratchbook/properties/:
//!
//! * one_event_one_line: to_line emits exactly one newline (trailing),
//!   the line parses back to the same event, and serialization is
//!   idempotent byte-for-byte.
//! * canonical_line_convergence: structure-preserving respellings of a
//!   canonical line (top-level and data key permutations, injected
//!   whitespace, stripped trailing newline) parse to the same event and
//!   re-serialize to byte-identical canonical bytes.
//!
//! Boundary arms (depth beyond the codec recursion limit, non-finite
//! floats, duplicate JSON keys) are ticket kaya-8tn and are outside the
//! generated domain.

use hegel::TestCase;
use hegel::generators as gs;
use hegel::generators::{BoxedPrintableGenerator, Generator, PrintableGenerator, SubtreeGenerator};
use kaya::event::{Actor, ActorType, Event, Provenance, SCHEMA_V0, Subject, parse_line, to_line};
use serde_json::{Map, Value};

const ID_CHARS: &str = "0123456789ABCDEFGHJKMNPQRSTVWXYZ";
const FIELD_CHARS: &str = "aX0-_\u{00e9}\u{0301}\u{1f980}\"\\ \t\n\u{2028}\u{2029}";
const DATA_KEY_CHARS: &str = "kx\u{03bc}";
const EVENT_TYPES: [&str; 3] = [
    "experiment.run.completed",
    "observation.note.added",
    "verification.acknowledged",
];
const TIMESTAMPS: [&str; 3] = [
    "2026-11-16T10:00:00Z",
    "2026-11-16T10:00:00.123456789Z",
    "2026-11-16T10:00:00+00:00",
];
const SUBJECT_TYPES: [&str; 3] = ["Project", "Run", "Observation"];
const TOP_KEYS: [&str; 9] = [
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

/// Non-empty strings from FIELD_CHARS: require_string refuses
/// whitespace-only values, so that alphabet tail is filtered out.
fn field_strings() -> BoxedPrintableGenerator<'static, String> {
    gs::text()
        .alphabet(FIELD_CHARS)
        .min_size(1)
        .max_size(8)
        .filter(|s: &String| !s.trim().is_empty())
        .boxed_printable()
}

fn data_keys() -> BoxedPrintableGenerator<'static, String> {
    gs::text()
        .alphabet(DATA_KEY_CHARS)
        .min_size(0)
        .max_size(3)
        .boxed_printable()
}

/// Finite scalars only; non-finite floats are ticket kaya-8tn.
fn data_leaves() -> BoxedPrintableGenerator<'static, Value> {
    hegel::one_of!(
        gs::integers::<i64>()
            .min_value(-3)
            .max_value(3)
            .map(Value::from)
            .boxed_printable(),
        gs::booleans().map(Value::from).boxed_printable(),
        gs::text()
            .alphabet(FIELD_CHARS)
            .min_size(0)
            .max_size(4)
            .map(Value::from)
            .boxed_printable(),
    )
    .boxed_printable()
}

fn object_of(pairs: Vec<(String, Value)>) -> Value {
    Value::Object(pairs.into_iter().collect::<Map<String, Value>>())
}

fn data_sub_values() -> BoxedPrintableGenerator<'static, Value> {
    gs::recursive(data_leaves(), |sub: SubtreeGenerator<Value>| {
        hegel::one_of!(
            gs::vecs(sub.clone())
                .max_size(3)
                .map(Value::Array)
                .boxed_printable(),
            gs::vecs(gs::tuples!(data_keys(), sub.clone()))
                .max_size(3)
                .map(object_of)
                .boxed_printable(),
        )
    })
    .max_depth(5)
    .boxed_printable()
}

/// data is always a JSON object, possibly empty.
fn data_values() -> BoxedPrintableGenerator<'static, Value> {
    gs::vecs(gs::tuples!(data_keys(), data_sub_values()))
        .max_size(4)
        .map(object_of)
        .boxed_printable()
}

/// A well-formed Event from small deterministic alphabets; under the ci
/// profile the sweep is exhaustive over the alphabet, not random.
fn events() -> BoxedPrintableGenerator<'static, Event> {
    let ids = gs::text().alphabet(ID_CHARS).min_size(26).max_size(26);
    let types = gs::sampled_from(EVENT_TYPES.to_vec()).map(str::to_owned);
    let timestamps = gs::sampled_from(TIMESTAMPS.to_vec()).map(str::to_owned);
    let actor_types = gs::sampled_from(vec![ActorType::Tool, ActorType::Human, ActorType::Agent]);
    let subject_types = gs::sampled_from(SUBJECT_TYPES.to_vec()).map(str::to_owned);

    gs::tuples!(
        ids,
        types,
        timestamps,
        gs::tuples!(actor_types, field_strings()),
        gs::tuples!(subject_types, field_strings()),
        data_values(),
        gs::tuples!(field_strings(), field_strings(), field_strings()),
        gs::vecs(field_strings()).max_size(3),
    )
    .map(
        |(
            id,
            event_type,
            timestamp,
            (actor_type, actor_name),
            (subject_type, subject_id),
            data,
            (tool, version, reference),
            artifacts,
        )| {
            Event {
                id,
                schema: SCHEMA_V0.to_string(),
                r#type: event_type,
                timestamp,
                actor: Actor {
                    r#type: actor_type,
                    name: actor_name,
                },
                subject: Subject {
                    r#type: subject_type,
                    id: subject_id,
                },
                data,
                provenance: Provenance {
                    tool,
                    version,
                    reference,
                },
                artifacts,
            }
        },
    )
    .boxed_printable()
}

/// Serialize a JSON object with its first level in the given key order
/// (when the length matches) and whitespace injected between tokens;
/// deeper levels recurse in map order, which parse treats the same.
fn write_json(value: &Value, order: &[String], sep: &str) -> String {
    let Value::Object(map) = value else {
        return value.to_string();
    };
    let ordered: Vec<(String, Value)> = if order.len() == map.len() {
        order
            .iter()
            .map(|k| {
                let v = map.get(k).expect("permuted key exists").clone();
                (k.clone(), v)
            })
            .collect()
    } else {
        map.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    };
    let items: Vec<String> = ordered
        .into_iter()
        .map(|(k, v)| format!("\"{k}\":{sep}{}", write_json(&v, &[], sep)))
        .collect();
    let joined = items.join(&format!(",{sep}"));
    format!("{{{joined}}}")
}

/// Respell the canonical line: permuted top-level and data keys,
/// whitespace between tokens, no trailing newline.
fn respell(event: &Event, top_order: &[String], data_order: &[String], sep: &str) -> String {
    let canonical = to_line(event).expect("event data is finite");
    let value: Value =
        serde_json::from_str(canonical.trim_end_matches('\n')).expect("canonical line is JSON");
    let map = value.as_object().expect("canonical line is an object");
    let mut parts = Vec::with_capacity(top_order.len());
    for key in top_order {
        let field = map.get(key.as_str()).expect("permuted key is a field");
        let text = if key == "data" {
            write_json(field, data_order, sep)
        } else {
            field.to_string()
        };
        parts.push(format!("\"{key}\":{sep}{text}"));
    }
    let joined = parts.join(&format!(",{sep}"));
    format!("{{{joined}}}")
}

#[hegel::test]
fn one_event_one_line(tc: TestCase) {
    let event = tc.draw(events());
    let line = to_line(&event).expect("event data is finite");

    // Exactly one newline, and it terminates the line, even when field
    // values themselves contain newlines (they are escaped by serde_json).
    assert_eq!(
        line.matches('\n').count(),
        1,
        "to_line must be line-atomic: {event:?}"
    );
    assert!(line.ends_with('\n'), "to_line must end with a newline");

    // The emitted line parses back to the same event.
    let parsed = parse_line(&line).expect("canonical line parses");
    assert_eq!(parsed, event, "round trip must preserve the event");

    // Serialization is idempotent: the second pass is byte-identical.
    let again = to_line(&parse_line(&line).expect("canonical line re-parses"))
        .expect("event data is finite");
    assert_eq!(again, line, "to_line must be idempotent");
}

#[hegel::test]
fn canonical_line_convergence(tc: TestCase) {
    let event = tc.draw(events());
    let canonical = to_line(&event).expect("event data is finite");

    let top_order = tc.draw(gs::permutations(
        TOP_KEYS.iter().map(|k| k.to_string()).collect::<Vec<_>>(),
    ));
    let data_order = tc.draw(gs::permutations(
        event
            .data
            .as_object()
            .map(|m| m.keys().cloned().collect::<Vec<_>>())
            .unwrap_or_default(),
    ));
    let sep = tc.draw(gs::sampled_from(vec![
        String::new(),
        " ".to_string(),
        "\t".to_string(),
        "  ".to_string(),
    ]));

    let variant = respell(&event, &top_order, &data_order, &sep);

    // The respelling parses to the same event and re-serializes to
    // byte-identical canonical bytes.
    let from_variant = parse_line(&variant).expect("respelled line parses");
    assert_eq!(
        from_variant, event,
        "key order and whitespace must not change the event"
    );
    assert_eq!(
        to_line(&from_variant).expect("event data is finite"),
        canonical,
        "every spelling must converge to the same canonical bytes"
    );

    // Trailing-whitespace stripping (the unterminated-line case) also
    // converges.
    let stripped = variant.trim_end_matches(['\n', '\t', ' ']);
    let from_stripped = parse_line(stripped).expect("unterminated respelled line parses");
    assert_eq!(from_stripped, event);
    assert_eq!(
        to_line(&from_stripped).expect("event data is finite"),
        canonical
    );

    // So does the canonical line itself, stripped of its trailing newline.
    let bare = canonical.trim_end_matches('\n');
    let from_bare = parse_line(bare).expect("unterminated canonical line parses");
    assert_eq!(from_bare, event);
    assert_eq!(
        to_line(&from_bare).expect("event data is finite"),
        canonical
    );
}
