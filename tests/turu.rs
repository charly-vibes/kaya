//! Contract tests for the turu adapter: importing turu whisper bundles as
//! `capture.entry.observed` events with deterministic ids, read-only
//! capture, payload secrecy, and atomic failure.

use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use kaya::adapters::{REGISTRY, turu};
use kaya::event::{ActorType, SCHEMA_V0, to_line};
use kaya::ledger::sha256_hex;
use kaya::model;
use serde_json::{Value, json};
use tempfile::TempDir;

const PROJECT: &str = "demo";
const REPO_KEY: &str = "charly/whisper-notes";
const SCOPE: &str = "daily-log";
const FAILURE_LABEL: &str = "kaya.adapters.failure_atomic";
const MARKER: &str = "TOPSECRET-turu-payload-marker";

/// Two entries whose fixture order is NOT the event-id sort order.
const ENTRIES: [(&str, &str, &str, &str); 2] = [
    (
        "2026-01-02T03:04:05Z",
        "5f8a1f60bd1e3a19c2e77b0aa1c4d9e2f3b4c5d6e7f8091a2b3c4d5e6f708192",
        "morning-musing",
        "First whisper of the day containing TOPSECRET-turu-payload-marker",
    ),
    (
        "2026-01-01T09:00:00Z",
        "1a2b3c4d5e6f708192a1b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e5f607",
        "evening-note",
        "Second whisper also carrying TOPSECRET-turu-payload-marker",
    ),
];

/// Serialize the bundle exactly as `turu bundle pack` 0.11.2 lays it out.
fn bundle_json(entries: &[Value]) -> String {
    json!({
        "repo_key": REPO_KEY,
        "scope": SCOPE,
        "entries": entries,
        "freeform": [{"note": "out of contract"}],
    })
    .to_string()
}

fn entry_value((ts, id, topic, text): &(&str, &str, &str, &str)) -> Value {
    json!({ "ts": ts, "id": id, "topic": topic, "text": text })
}

fn default_bundle() -> String {
    bundle_json(&ENTRIES.iter().map(entry_value).collect::<Vec<_>>())
}

fn write_file(dir: &Path, name: &str, contents: &str) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, contents).expect("write fixture file");
    path
}

/// A temp directory holding only the bundle file, no `.whisper/` anywhere.
fn bundled_dir(body: &str) -> (TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let bundle = write_file(dir.path(), "whisper-2026-01.turubundle", body);
    (dir, bundle)
}

/// Every file under `root`, relative path -> bytes, sorted.
fn snapshot(root: &Path) -> Vec<(String, Vec<u8>)> {
    let mut files = Vec::new();
    let mut stack: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).expect("readable directory") {
            let path = entry.expect("directory entry").path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let relative = path
                .strip_prefix(root)
                .expect("relative path")
                .to_string_lossy()
                .to_string();
            files.push((relative, fs::read(&path).expect("readable file")));
        }
    }
    files.sort();
    files
}

#[test]
fn capture_emits_one_event_per_bundle_entry() {
    let (_dir, bundle) = bundled_dir(&default_bundle());

    let events = turu::capture(&bundle, PROJECT).expect("capture succeeds");

    assert_eq!(events.len(), ENTRIES.len(), "one event per bundle entry");

    for event in &events {
        assert_eq!(event.schema, SCHEMA_V0);
        assert_eq!(event.r#type, "capture.entry.observed");
        assert_eq!(event.actor.r#type, ActorType::Tool);
        assert_eq!(event.actor.name, "turu");
        assert_eq!(event.subject.r#type, "Source");

        let entry_id = event.data["entry"].as_str().expect("entry id");
        assert_eq!(
            event.subject.id,
            format!("{PROJECT}/Source/entry-{entry_id}")
        );
        assert!(
            event.data["topic"].is_string(),
            "topic stays the bundle's own string"
        );
        assert_eq!(event.data["scope"], json!(SCOPE));
        assert_eq!(event.data["repo_key"], json!(REPO_KEY));
        assert_eq!(event.provenance.tool, "turu");
        assert_eq!(event.provenance.version, "0.11.2");
        assert_eq!(event.timestamp.as_str().len(), "2026-01-02T03:04:05Z".len());
        assert!(event.id.starts_with("evt-"));
        assert_eq!(event.id.len(), 4 + 26);
    }

    let timestamps: Vec<&str> = events.iter().map(|e| e.timestamp.as_str()).collect();
    assert!(timestamps.contains(&"2026-01-02T03:04:05Z"));
    assert!(timestamps.contains(&"2026-01-01T09:00:00Z"));

    let topics: Vec<&str> = events
        .iter()
        .map(|e| e.data["topic"].as_str().expect("topic"))
        .collect();
    assert!(topics.contains(&"morning-musing"));
    assert!(topics.contains(&"evening-note"));
}

#[test]
fn capture_sorts_by_id_and_replays_identically() {
    let (_dir, bundle) = bundled_dir(&default_bundle());

    let first = turu::capture(&bundle, PROJECT).expect("first capture");
    let second = turu::capture(&bundle, PROJECT).expect("second capture");

    assert_eq!(first.len(), ENTRIES.len(), "one event per bundle entry");
    assert!(
        first.windows(2).all(|pair| pair[0].id < pair[1].id),
        "events are sorted by id"
    );
    // The fixture's entry order is not the id sort order, so sorting is real.
    assert_ne!(
        first
            .iter()
            .map(|e| e.data["entry"].as_str().expect("entry"))
            .collect::<Vec<_>>(),
        ENTRIES.iter().map(|(_, id, ..)| *id).collect::<Vec<_>>(),
        "fixture must not already be in id-sorted order"
    );

    assert_eq!(first, second, "capture replays identically");
    let first_lines: Vec<String> = first.iter().map(to_line).collect();
    let second_lines: Vec<String> = second.iter().map(to_line).collect();
    assert_eq!(first_lines, second_lines, "replay is byte-identical");
}

#[test]
fn reimport_yields_identical_event_ids() {
    let body = default_bundle();

    let first_run = bundled_dir(&body);
    let second_run = bundled_dir(&body);

    let first = turu::capture(&first_run.1, PROJECT).expect("first capture");
    let second = turu::capture(&second_run.1, PROJECT).expect("second capture");

    assert_eq!(first.len(), ENTRIES.len(), "one event per bundle entry");
    let first_ids: Vec<&str> = first.iter().map(|e| e.id.as_str()).collect();
    let second_ids: Vec<&str> = second.iter().map(|e| e.id.as_str()).collect();
    assert_eq!(first_ids, second_ids);
}

#[test]
fn bundle_captured_without_whisper_workspace_internals() {
    // Only the bundle file exists; there is no `.whisper/` tree beside it, so
    // capture must consume the shared store bundle alone.
    let (dir, bundle) = bundled_dir(&default_bundle());
    assert!(
        !dir.path().join(".whisper").exists(),
        "fixture must contain no .whisper workspace"
    );

    let events = turu::capture(&bundle, PROJECT).expect("capture succeeds without a workspace");

    assert_eq!(events.len(), ENTRIES.len());
}

#[test]
fn capture_writes_nothing_to_the_source_tree() {
    let (dir, bundle) = bundled_dir(&default_bundle());

    let before = snapshot(dir.path());
    let events = turu::capture(&bundle, PROJECT).expect("capture succeeds");
    let after = snapshot(dir.path());

    assert_eq!(events.len(), ENTRIES.len(), "one event per bundle entry");
    assert_eq!(before, after, "capture must not write to the source tree");
}

#[test]
fn bundle_entry_text_is_never_copied_into_events() {
    let (_dir, bundle) = bundled_dir(&default_bundle());

    let events = turu::capture(&bundle, PROJECT).expect("capture succeeds");

    assert!(
        !events.is_empty(),
        "the fixture entries carry the marker in their text"
    );
    for event in &events {
        let line = to_line(event);
        assert!(
            !line.contains(MARKER),
            "entry text leaked into an event line: {line}"
        );
    }
}

#[test]
fn provenance_names_turu_bundle_digest_and_repo_key() {
    let (_dir, bundle) = bundled_dir(&default_bundle());
    let digest = sha256_hex(&fs::read(&bundle).expect("read bundle bytes"));

    let events = turu::capture(&bundle, PROJECT).expect("capture succeeds");

    assert_eq!(events.len(), ENTRIES.len(), "one event per bundle entry");
    for event in &events {
        assert_eq!(event.provenance.reference, digest);
        assert_eq!(
            event.artifacts,
            vec![format!("sha256:{digest}")],
            "artifacts are path-independent"
        );
        assert_eq!(event.data["bundle_digest"], json!(digest));
        assert_eq!(event.data["repo_key"], json!(REPO_KEY));
    }
}

#[test]
fn model_builds_source_node_with_entry_attribute() {
    let (_dir, bundle) = bundled_dir(&default_bundle());

    let events = turu::capture(&bundle, PROJECT).expect("capture succeeds");
    let graph = model::build(&events).expect("captured entries enter the model");

    let entry_id = ENTRIES[0].1;
    let subject = format!("{PROJECT}/Source/entry-{entry_id}");
    let node = graph
        .node(&subject)
        .unwrap_or_else(|| panic!("a Source node exists for {subject}"));
    assert_eq!(
        node.attributes["entry"],
        json!(entry_id),
        "the Source node records the bundle entry id"
    );
}

#[test]
fn malformed_bundle_entry_aborts_atomically() {
    let mut entries = ENTRIES.iter().map(entry_value).collect::<Vec<_>>();
    if let Value::Object(entry) = &mut entries[1] {
        entry.remove("id");
    }
    let (dir, bundle) = bundled_dir(&bundle_json(&entries));
    let bundle_name = bundle
        .strip_prefix(dir.path())
        .unwrap()
        .to_string_lossy()
        .to_string();

    let error = turu::capture(&bundle, PROJECT).expect_err("a malformed entry must fail");

    assert_eq!(error.label, FAILURE_LABEL);
    assert!(
        error.message.contains("turu"),
        "message names the source: {}",
        error.message
    );
    assert!(
        error.message.contains(&bundle_name),
        "message names the bundle: {}",
        error.message
    );
    let ids: Vec<&str> = ENTRIES.iter().map(|(_, id, ..)| *id).collect();
    assert!(
        ids.iter().any(|id| error.message.contains(id)),
        "message names the offending entry: {}",
        error.message
    );
}

#[test]
fn invalid_bundle_timestamp_aborts_atomically() {
    let mut entries = ENTRIES.iter().map(entry_value).collect::<Vec<_>>();
    entries[1] = json!({
        "ts": "not-a-timestamp",
        "id": ENTRIES[1].1,
        "topic": ENTRIES[1].2,
        "text": ENTRIES[1].3,
    });
    let (dir, bundle) = bundled_dir(&bundle_json(&entries));
    let bundle_name = bundle
        .strip_prefix(dir.path())
        .unwrap()
        .to_string_lossy()
        .to_string();

    let error = turu::capture(&bundle, PROJECT).expect_err("an invalid timestamp must fail");

    assert_eq!(error.label, FAILURE_LABEL);
    assert!(
        error.message.contains("turu"),
        "message names the source: {}",
        error.message
    );
    assert!(
        error.message.contains(&bundle_name),
        "message names the bundle: {}",
        error.message
    );
    assert!(
        error.message.contains(ENTRIES[1].1),
        "message names the offending entry: {}",
        error.message
    );
}

#[test]
fn missing_bundle_file_is_a_labeled_failure() {
    let dir = tempfile::tempdir().expect("tempdir");
    let missing = dir.path().join("absent.turubundle");

    let error = turu::capture(&missing, PROJECT).expect_err("a missing bundle must fail");

    assert_eq!(error.label, FAILURE_LABEL);
    assert!(
        error.message.contains("turu"),
        "message names the source: {}",
        error.message
    );
    assert!(
        error.message.contains("absent.turubundle"),
        "message names the bundle: {}",
        error.message
    );
}

#[test]
fn registry_names_the_turu_source_contract() {
    let registration = REGISTRY
        .iter()
        .find(|entry| entry.name == "turu")
        .expect("turu is registered");

    assert!(
        registration.source_contract.contains("bundle"),
        "the contract names the turu bundle format: {}",
        registration.source_contract
    );
}

#[test]
fn cli_import_turu_appends_then_deduplicates() {
    let _ledger_dir = tempfile::tempdir().expect("ledger tempdir");
    let (_fixture_dir, bundle) = bundled_dir(&default_bundle());

    let init = Command::cargo_bin("kaya")
        .expect("kaya binary is built")
        .current_dir(_ledger_dir.path())
        .arg("init")
        .output()
        .expect("init runs");
    assert!(
        init.status.success(),
        "init failed: {}",
        String::from_utf8_lossy(&init.stderr)
    );

    let first = Command::cargo_bin("kaya")
        .expect("kaya binary is built")
        .current_dir(_ledger_dir.path())
        .args([
            "import",
            "turu",
            "--bundle",
            bundle.to_str().expect("bundle path is UTF-8"),
            "--project",
            PROJECT,
        ])
        .output()
        .expect("import runs");
    assert!(
        first.status.success(),
        "import turu failed: {}",
        String::from_utf8_lossy(&first.stderr)
    );

    let envelope: Value = serde_json::from_slice(&first.stdout).expect("a JSON envelope");
    assert_eq!(envelope["ok"], json!(true));
    assert_eq!(envelope["data"]["appended"], json!(ENTRIES.len()));

    let second = Command::cargo_bin("kaya")
        .expect("kaya binary is built")
        .current_dir(_ledger_dir.path())
        .args([
            "import",
            "turu",
            "--bundle",
            bundle.to_str().expect("bundle path is UTF-8"),
            "--project",
            PROJECT,
        ])
        .output()
        .expect("second import runs");
    assert!(
        second.status.success(),
        "second import failed: {}",
        String::from_utf8_lossy(&second.stderr)
    );

    let envelope: Value = serde_json::from_slice(&second.stdout).expect("a JSON envelope");
    assert_eq!(envelope["ok"], json!(true));
    assert_eq!(
        envelope["data"]["duplicates"],
        json!(ENTRIES.len()),
        "reimporting the same bundle deduplicates every event"
    );
    assert_eq!(envelope["data"]["appended"], json!(0));
}
