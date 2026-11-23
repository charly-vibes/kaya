//! Fixtures and readers for the facts census tests.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use crate::common::{
    AUTHORED_BEGIN, AUTHORED_END, AUTHORED_PROSE, Fixture, INDEX, LEDGER, git, run_ok,
};
use crate::snapshot::sha256_hex;
use crate::tampering::worktree_bytes;

pub(crate) const WAI_NOTE: &str = ".wai/projects/kaya/research/2026-11-18-note.md";

/// The canonical `events_by_source` keys: capture family and provenance, never
/// the actor name.
pub(crate) const SOURCES: [&str; 5] = ["wai", "git", "test-runner", "model", "turu"];

/// A turu whisper bundle with one entry, in the layout `turu bundle pack`
/// produces: a repo key, a scope, and the entries themselves.
pub(crate) const TURU_BUNDLE: &str = concat!(
    "{\"repo_key\":\"charly/whisper-notes\",\"scope\":\"daily-log\",\"entries\":[",
    "{\"ts\":\"2026-01-02T03:04:05Z\",",
    "\"id\":\"5f8a1f60bd1e3a19c2e77b0aa1c4d9e2f3b4c5d6e7f8091a2b3c4d5e6f708192\",",
    "\"topic\":\"morning-musing\",\"text\":\"a whisper the adapter never copies\"}",
    "]}",
);

/// Where a fixture writes the turu bundle it imports.
pub(crate) const TURU_BUNDLE_PATH: &str = "whisper-2026-01.turubundle";

/// Write the turu bundle under `root` and return its path.
pub(crate) fn write_turu_bundle(root: &Path) -> PathBuf {
    let path = root.join(TURU_BUNDLE_PATH);
    fs::write(&path, TURU_BUNDLE).expect("write the turu bundle");
    path
}

/// A record holding one turu whisper-bundle entry: import the bundle, then
/// rebuild, publish, and commit the grown ledger, so `check` reads a record
/// whose committed history it can verify.
pub(crate) fn record_with_turu_source(root: &Path) {
    let bundle = write_turu_bundle(root);
    let bundle = bundle.to_str().expect("the bundle path is UTF-8");
    run_ok(root, &["import", "turu", "--bundle", bundle]);
    run_ok(root, &["build"]);
    run_ok(root, &["publish"]);
    git(root, &["add", "--all"]);
    git(
        root,
        &[
            "commit",
            "--quiet",
            "--message",
            "record the imported turu observation",
        ],
    );
}

/// Write one human-authored research note into the wai tree.
pub(crate) fn add_wai_artifact(root: &Path) {
    let path = root.join(WAI_NOTE);
    fs::create_dir_all(path.parent().expect("the wai artifact has a parent"))
        .expect("create the wai research directory");
    fs::write(
        &path,
        "# facts fixture note\n\nA human note the wai adapter only observes.\n",
    )
    .expect("write the wai artifact");
}

/// A valid libtest JSONL report: suite and bare-string records are ignored, so
/// the terminal test events are what the adapter tallies.
pub(crate) const LIBTEST_REPORT: &str = concat!(
    "{\"type\":\"suite\",\"event\":\"started\",\"test_count\":4}\n",
    "{\"type\":\"test\",\"event\":\"started\",\"name\":\"passes_one\"}\n",
    "{\"type\":\"test\",\"event\":\"ok\",\"name\":\"passes_one\"}\n",
    "{\"type\":\"test\",\"event\":\"started\",\"name\":\"passes_two\"}\n",
    "{\"type\":\"test\",\"event\":\"ok\",\"name\":\"passes_two\"}\n",
    "{\"type\":\"test\",\"event\":\"started\",\"name\":\"fails\"}\n",
    "{\"type\":\"test\",\"event\":\"failed\",\"name\":\"fails\"}\n",
    "{\"type\":\"test\",\"event\":\"ignored\",\"name\":\"skipped\"}\n",
    "\"a bare JSON string is not a test event\"\n",
    "{\"type\":\"suite\",\"event\":\"ok\"}\n",
);

/// Where a fixture writes the run report it imports.
pub(crate) const LIBTEST_REPORT_PATH: &str = "cargo-test-results.jsonl";

/// Write the libtest report under `root` and return its path.
pub(crate) fn write_libtest_report(root: &Path) -> PathBuf {
    let path = root.join(LIBTEST_REPORT_PATH);
    fs::write(&path, LIBTEST_REPORT).expect("write the libtest report");
    path
}

/// A record with two capture sources: import the wai tree and `limit` recent
/// commits, then rebuild, publish, and commit the grown ledger, so `check`
/// reads a record whose committed history it can verify.
pub(crate) fn record_with_two_sources(root: &Path, limit: &str) {
    add_wai_artifact(root);
    run_ok(root, &["import", "wai"]);
    run_ok(root, &["import", "git", "--limit", limit]);
    run_ok(root, &["build"]);
    run_ok(root, &["publish"]);
    git(root, &["add", "--all"]);
    git(
        root,
        &[
            "commit",
            "--quiet",
            "--message",
            "record imported observations",
        ],
    );
}

/// Every event the worktree ledger records, in file order.
pub(crate) fn ledger_events(root: &Path) -> Vec<Value> {
    let text = fs::read_to_string(root.join(LEDGER)).expect("the ledger is readable");
    text.lines()
        .map(|line| {
            serde_json::from_str(line)
                .unwrap_or_else(|error| panic!("a ledger line is one JSON event ({error}): {line}"))
        })
        .collect()
}

/// The greatest event timestamp the ledger records.
pub(crate) fn newest_timestamp(events: &[Value]) -> String {
    events
        .iter()
        .map(|event| {
            event["timestamp"]
                .as_str()
                .unwrap_or_else(|| panic!("every event carries a timestamp: {event}"))
        })
        .max()
        .expect("the ledger records events")
        .to_owned()
}

/// Cite `id` from the seeded authored block, leaving every other byte alone.
pub(crate) fn cite_in_authored_prose(root: &Path, id: &str) {
    let path = root.join(INDEX);
    let document = fs::read_to_string(&path).expect("the fixture index is readable");
    let marker = format!("<!-- kaya:cites {id} -->");
    assert!(
        !document.contains(&marker),
        "the authored block does not cite {id} yet: {document}"
    );
    let cited = format!("{AUTHORED_PROSE}\n{marker}");
    let updated = document.replace(AUTHORED_PROSE, &cited);
    assert_ne!(updated, document, "the authored prose carries the marker");
    fs::write(&path, updated).expect("cite the event from authored prose");
}

/// The authored bytes of the research index, markers included.
pub(crate) fn authored_bytes(root: &Path) -> Vec<u8> {
    let document = fs::read_to_string(root.join(INDEX)).expect("the index is readable");
    let start = document
        .find(AUTHORED_BEGIN)
        .unwrap_or_else(|| panic!("the index carries an authored block: {document}"));
    let rest = &document[start..];
    let end = rest
        .find(AUTHORED_END)
        .unwrap_or_else(|| panic!("the authored block is closed: {document}"))
        + AUTHORED_END.len();
    rest.as_bytes()[..end].to_vec()
}

/// Append one correction that changes the stored data of the `target` event.
///
/// The correction clones the cited event's envelope — schema, type, actor,
/// subject, provenance, and artifacts — takes a fresh id, a later timestamp,
/// and data naming the superseded id plus a changed attribute. The appended
/// line extends the committed ledger on a line boundary, so the record's
/// history stays verifiable.
pub(crate) fn append_correction(root: &Path, target: &Value) {
    const CORRECTION_ID: &str = "evt-0123456789abcdef0123456789";
    const CORRECTION_AT: &str = "2026-11-20T00:00:00Z";

    let events = ledger_events(root);
    assert!(
        !events
            .iter()
            .any(|event| event["id"] == json!(CORRECTION_ID)),
        "the correction id is fresh: {CORRECTION_ID}"
    );
    assert!(
        newest_timestamp(&events).as_str() < CORRECTION_AT,
        "the correction is recorded after every seed event"
    );
    let target_id = target["id"]
        .as_str()
        .unwrap_or_else(|| panic!("the cited event carries an id: {target}"));

    let mut correction = target.clone();
    correction["id"] = json!(CORRECTION_ID);
    correction["timestamp"] = json!(CORRECTION_AT);
    correction["data"] = json!({
        "supersedes": target_id,
        "attributes": { "title": "corrected by test" },
    });
    let mut line = serde_json::to_string(&correction)
        .expect("the correction event serializes to one JSON line");
    line.push('\n');

    let mut ledger = fs::OpenOptions::new()
        .append(true)
        .open(root.join(LEDGER))
        .expect("the ledger is appended to, never rewritten");
    ledger
        .write_all(line.as_bytes())
        .expect("append the correction line");
}

/// A committed record whose authored index block cites a seed event, then a
/// correction appended to the ledger after the commit changes that event.
///
/// Returns the fixture and the changed evidence id. `build` must report the
/// stale block and still exit 0 without rewriting authored bytes; `publish`
/// then composes the publication tree from the corrected research bytes, so
/// staleness is the only defect a `check` can report.
pub(crate) fn record_with_a_stale_authored_block() -> (Fixture, String) {
    let fixture = Fixture::new();
    let root = fixture.root();
    let cited = ledger_events(root)[0].clone();
    let target = cited["id"]
        .as_str()
        .expect("the first seed event carries an id")
        .to_owned();

    cite_in_authored_prose(root, &target);
    run_ok(root, &["build"]);
    run_ok(root, &["publish"]);
    git(root, &["add", "--all"]);
    git(
        root,
        &[
            "commit",
            "--quiet",
            "--message",
            "cite the seed event from authored prose",
        ],
    );

    let authored = authored_bytes(root);
    append_correction(root, &cited);
    let built = run_ok(root, &["build"]);
    assert_eq!(
        built["data"]["stale"],
        json!([{ "block": "authored-1", "evidence": [target.clone()] }]),
        "build reports the stale authored block without failing: {built}"
    );
    assert_eq!(
        authored_bytes(root),
        authored,
        "build reports staleness without rewriting authored bytes"
    );
    run_ok(root, &["publish"]);
    assert_eq!(
        authored_bytes(root),
        authored,
        "publish leaves the authored bytes alone"
    );

    (fixture, target)
}

pub(crate) fn record_facts(envelope: &Value) -> &Value {
    const FIELDS: [&str; 7] = [
        "events",
        "ledger_digest",
        "last_capture_commit",
        "newest_event_at",
        "events_by_source",
        "emitted_kinds",
        "omitted_kinds",
    ];
    let data = envelope["data"]
        .as_object()
        .unwrap_or_else(|| panic!("check reports data as an object: {envelope}"));
    for field in FIELDS {
        assert!(
            data.contains_key(field),
            "check reports data.{field}: {envelope}"
        );
    }
    &envelope["data"]
}

/// The canonical source counts, requiring exactly the four source keys.
pub(crate) fn source_counts(facts: &Value) -> BTreeMap<String, u64> {
    let counts = facts["events_by_source"]
        .as_object()
        .unwrap_or_else(|| panic!("check reports data.events_by_source as an object: {facts}"));
    let keys = counts.keys().cloned().collect::<BTreeSet<_>>();
    let expected = SOURCES
        .iter()
        .map(|source| (*source).to_owned())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        keys, expected,
        "events_by_source holds exactly the canonical sources {SOURCES:?}: {facts}"
    );
    counts
        .iter()
        .map(|(source, count)| {
            let count = count
                .as_u64()
                .unwrap_or_else(|| panic!("the {source} source count is an integer: {facts}"));
            (source.clone(), count)
        })
        .collect()
}

/// A dotted `family.kind` list: strings, strictly sorted (hence unique), with a
/// non-empty family and kind on either side of a dot.
pub(crate) fn dotted_kinds(facts: &Value, field: &str) -> Vec<String> {
    let list = facts[field]
        .as_array()
        .unwrap_or_else(|| panic!("check reports data.{field} as an array: {facts}"));
    let names = list
        .iter()
        .map(|name| {
            name.as_str()
                .unwrap_or_else(|| panic!("data.{field} holds strings: {facts}"))
                .to_owned()
        })
        .collect::<Vec<_>>();
    for pair in names.windows(2) {
        assert!(
            pair[0] < pair[1],
            "data.{field} is sorted and deduplicated: {names:?}"
        );
    }
    for name in &names {
        let (family, kind) = name.split_once('.').unwrap_or_else(|| {
            panic!("data.{field} names dotted family.kind values, got \"{name}\": {facts}")
        });
        assert!(
            !family.is_empty() && !kind.is_empty() && !name.split('.').any(str::is_empty),
            "data.{field} names dotted family.kind values, got \"{name}\": {facts}"
        );
    }
    names
}

/// Assert the facts describe the verified ledger bytes: the event count, the
/// sha256 ledger digest, and the newest event timestamp.
pub(crate) fn assert_facts_match_ledger(facts: &Value, root: &Path, envelope: &Value) {
    let events = ledger_events(root);
    assert_eq!(
        facts["events"].as_u64(),
        Some(events.len() as u64),
        "the event count matches the ledger: {envelope}"
    );
    assert_eq!(
        facts["ledger_digest"],
        json!(sha256_hex(&worktree_bytes(root, LEDGER))),
        "the digest is sha256 over the verified ledger bytes: {envelope}"
    );
    assert_eq!(
        facts["newest_event_at"],
        json!(newest_timestamp(&events)),
        "the newest event timestamp is the greatest recorded: {envelope}"
    );
}
