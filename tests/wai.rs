//! Contract tests for the wai adapter: one event per markdown artifact,
//! deterministic replay, read-only capture, and atomic failure.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use kaya::adapters::{REGISTRY, wai};
use kaya::event::{ActorType, Event, to_line};

const PROJECT: &str = "kaya";

/// Fixed modification time for the artifact that has no date in its name.
const FIXED_MTIME_SECS: u64 = 1_760_000_000;

const ALPHA: &str = ".wai/projects/kaya/designs/2026-01-02-alpha.md";
const BETA: &str = ".wai/projects/kaya/designs/2026-01-03-beta design.md";
const RENAMED: &str = ".wai/projects/kaya/designs/renamed-design.md";
const RESEARCH: &str = ".wai/projects/kaya/research/2026-02-05-origin.md";
const HANDOFF: &str = ".wai/projects/kaya/handoffs/2026-03-06-latest.md";

fn write_file(root: &Path, relative: &str, contents: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().expect("fixture file has a parent")).unwrap();
    fs::write(path, contents).unwrap();
}

/// Three designs, one research, one handoff; the renamed design has no date.
fn fixture(root: &Path) {
    for (relative, body) in [
        (ALPHA, "# alpha\n"),
        (BETA, "# beta\n"),
        (RENAMED, "# renamed\n"),
        (RESEARCH, "# origin\n"),
        (HANDOFF, "# latest\n"),
    ] {
        write_file(root, relative, body);
    }

    // Neither a non-markdown file nor an unknown kind directory is an artifact.
    write_file(root, ".wai/projects/kaya/designs/notes.txt", "not markdown");
    write_file(root, ".wai/projects/kaya/scratch/draft.md", "# draft\n");

    let renamed = root.join(RENAMED);
    let file = fs::File::options().write(true).open(&renamed).unwrap();
    file.set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(FIXED_MTIME_SECS))
        .unwrap();
}

fn fixed_mtime() -> String {
    chrono::DateTime::<chrono::Utc>::from(
        SystemTime::UNIX_EPOCH + Duration::from_secs(FIXED_MTIME_SECS),
    )
    .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

fn by_path<'a>(events: &'a [Event], relative: &str) -> &'a Event {
    events
        .iter()
        .find(|event| event.data["path"] == relative)
        .unwrap_or_else(|| panic!("no event for {relative}"))
}

/// Every file under `root`, relative path -> bytes, sorted.
fn snapshot(root: &Path) -> Vec<(String, Vec<u8>)> {
    let mut files = Vec::new();
    let mut stack: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else {
                let relative = path
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                files.push((relative, fs::read(&path).unwrap()));
            }
        }
    }
    files.sort();
    files
}

#[test]
fn capture_emits_one_event_per_markdown_artifact() {
    let temp = tempfile::tempdir().unwrap();
    fixture(temp.path());

    let events = wai::capture(temp.path(), PROJECT).unwrap();
    assert_eq!(events.len(), 5);

    for event in &events {
        assert_eq!(event.schema, "kaya.event/v0");
        assert_eq!(event.r#type, "capture.artifact.observed");
        assert_eq!(event.actor.r#type, ActorType::Tool);
        assert_eq!(event.actor.name, "wai");
        assert_eq!(event.subject.r#type, "Artifact");

        let data = event.data.as_object().expect("data is an object");
        assert_eq!(data.len(), 2);
        assert!(data.contains_key("kind"));
        assert!(data.contains_key("path"));
        for forbidden in [
            "verdict",
            "conclusion",
            "interpretation",
            "hypothesis_verdict",
            "hypothesis_status",
        ] {
            assert!(
                !data.contains_key(forbidden),
                "{forbidden} is interpretation"
            );
        }
    }

    let alpha = by_path(&events, ALPHA);
    assert_eq!(alpha.subject.id, "kaya/Artifact/2026-01-02-alpha");
    assert_eq!(alpha.data["kind"], "designs");
    assert_eq!(alpha.timestamp, "2026-01-02T00:00:00Z");
    assert_eq!(alpha.provenance.tool, "wai");
    assert_eq!(alpha.provenance.version, "2026.10.10");
    assert_eq!(alpha.provenance.reference, ALPHA);
    assert_eq!(alpha.artifacts, vec![ALPHA.to_string()]);

    let beta = by_path(&events, BETA);
    assert_eq!(beta.subject.id, "kaya/Artifact/2026-01-03-beta-design");
    assert_eq!(beta.data["kind"], "designs");

    let renamed = by_path(&events, RENAMED);
    assert_eq!(renamed.subject.id, "kaya/Artifact/renamed-design");
    assert_eq!(renamed.timestamp, fixed_mtime());

    assert_eq!(by_path(&events, RESEARCH).data["kind"], "research");
    assert_eq!(by_path(&events, HANDOFF).data["kind"], "handoffs");
}

#[test]
fn capture_sorts_by_id_and_replays_byte_identically() {
    let temp = tempfile::tempdir().unwrap();
    fixture(temp.path());

    let first = wai::capture(temp.path(), PROJECT).unwrap();
    let second = wai::capture(temp.path(), PROJECT).unwrap();

    assert!(
        first.windows(2).all(|pair| pair[0].id < pair[1].id),
        "events are sorted by id"
    );
    for event in &first {
        assert!(event.id.starts_with("evt-"));
        assert_eq!(event.id.len(), 4 + 26);
        assert!(
            event.id[4..]
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)),
            "id suffix is lowercase hex"
        );
    }

    assert_eq!(first, second);
    let first_lines: Vec<String> = first.iter().map(to_line).collect();
    let second_lines: Vec<String> = second.iter().map(to_line).collect();
    assert_eq!(first_lines, second_lines);
}

#[test]
fn capture_writes_nothing_to_the_source_tree() {
    let temp = tempfile::tempdir().unwrap();
    fixture(temp.path());

    let before = snapshot(temp.path());
    wai::capture(temp.path(), PROJECT).unwrap();
    let after = snapshot(temp.path());

    assert_eq!(before, after);
}

#[test]
fn missing_wai_tree_yields_no_events() {
    let temp = tempfile::tempdir().unwrap();
    assert!(wai::capture(temp.path(), PROJECT).unwrap().is_empty());

    fs::create_dir_all(temp.path().join(".wai")).unwrap();
    assert!(wai::capture(temp.path(), PROJECT).unwrap().is_empty());
}

#[cfg(unix)]
#[test]
fn unreadable_record_aborts_atomically() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempfile::tempdir().unwrap();
    fixture(temp.path());

    let victim = temp.path().join(BETA);
    fs::set_permissions(&victim, fs::Permissions::from_mode(0o000)).unwrap();

    // A privileged runner can still read mode 000; the property needs an
    // actually unreadable record.
    if fs::read(&victim).is_ok() {
        fs::set_permissions(&victim, fs::Permissions::from_mode(0o644)).unwrap();
        return;
    }

    let error = wai::capture(temp.path(), PROJECT).unwrap_err();

    fs::set_permissions(&victim, fs::Permissions::from_mode(0o644)).unwrap();

    assert_eq!(error.label, "kaya.adapters.failure_atomic");
    assert!(error.message.contains("wai"), "names the source: {error}");
    assert!(error.message.contains(BETA), "names the record: {error}");
    assert!(
        error
            .to_string()
            .starts_with("kaya.adapters.failure_atomic: ")
    );
}

#[test]
fn unparsable_record_aborts_atomically() {
    let temp = tempfile::tempdir().unwrap();
    fixture(temp.path());

    let record = ".wai/projects/kaya/research/undated-binary.md";
    fs::write(temp.path().join(record), [0xff_u8, 0xfe]).unwrap();

    let error = wai::capture(temp.path(), PROJECT).unwrap_err();
    assert_eq!(error.label, "kaya.adapters.failure_atomic");
    assert!(error.message.contains("wai"), "names the source: {error}");
    assert!(error.message.contains(record), "names the record: {error}");
}

#[test]
fn registry_names_the_wai_source_contract() {
    let registration = REGISTRY
        .iter()
        .find(|entry| entry.name == "wai")
        .expect("wai is registered");

    assert!(registration.source_contract.contains(".wai"));
    assert!(registration.source_contract.contains("artifact tree"));
}
