//! Contract tests for the wai adapter: one event per markdown artifact,
//! deterministic replay, read-only capture, and atomic failure.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use kaya::adapters::{REGISTRY, wai};
use kaya::event::{ActorType, Event, to_line};
use kaya::model;

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
        collect_dir(root, &dir, &mut stack, &mut files);
    }
    files.sort();
    files
}

fn collect_dir(
    root: &Path,
    dir: &Path,
    stack: &mut Vec<PathBuf>,
    files: &mut Vec<(String, Vec<u8>)>,
) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            stack.push(path);
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        files.push((relative, fs::read(&path).unwrap()));
    }
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
        // kind + path always; title + title_digest join when the artifact
        // carries a `# ` heading (all fixtures do).
        assert_eq!(data.len(), 4);
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
    assert_eq!(alpha.subject.id, "kaya/Artifact/designs-2026-01-02-alpha");
    assert_eq!(alpha.data["kind"], "designs");
    assert_eq!(alpha.timestamp, "2026-01-02T00:00:00Z");
    assert_eq!(alpha.provenance.tool, "wai");
    assert_eq!(alpha.provenance.version, "2026.10.10");
    assert_eq!(alpha.provenance.reference, ALPHA);
    assert_eq!(alpha.artifacts, vec![ALPHA.to_string()]);

    let beta = by_path(&events, BETA);
    assert_eq!(
        beta.subject.id,
        "kaya/Artifact/designs-2026-01-03-beta-design"
    );
    assert_eq!(beta.data["kind"], "designs");

    let renamed = by_path(&events, RENAMED);
    assert_eq!(renamed.subject.id, "kaya/Artifact/designs-renamed-design");
    assert_eq!(renamed.timestamp, fixed_mtime());

    let research = by_path(&events, RESEARCH);
    assert_eq!(
        research.subject.id,
        "kaya/Artifact/research-2026-02-05-origin"
    );
    assert_eq!(research.data["kind"], "research");

    let handoff = by_path(&events, HANDOFF);
    assert_eq!(
        handoff.subject.id,
        "kaya/Artifact/handoffs-2026-03-06-latest"
    );
    assert_eq!(handoff.data["kind"], "handoffs");
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
    let first_lines: Vec<String> = first
        .iter()
        .map(|event| to_line(event).expect("event data is finite"))
        .collect();
    let second_lines: Vec<String> = second
        .iter()
        .map(|event| to_line(event).expect("event data is finite"))
        .collect();
    assert_eq!(first_lines, second_lines);
}

#[test]
fn same_stem_under_two_kinds_yields_distinct_nodes() {
    let temp = tempfile::tempdir().unwrap();
    let designs = ".wai/projects/kaya/designs/shared.md";
    let research = ".wai/projects/kaya/research/shared.md";
    write_file(temp.path(), designs, "# design\n");
    write_file(temp.path(), research, "# research\n");

    let events = wai::capture(temp.path(), PROJECT).unwrap();
    assert_eq!(events.len(), 2);

    let subjects: Vec<&str> = events
        .iter()
        .map(|event| event.subject.id.as_str())
        .collect();
    assert!(subjects.contains(&"kaya/Artifact/designs-shared"));
    assert!(subjects.contains(&"kaya/Artifact/research-shared"));
    assert_ne!(events[0].id, events[1].id, "the records keep distinct ids");

    let graph = model::build(&events).expect("both artifacts enter the model");
    assert_eq!(graph.nodes().count(), 2);
    assert!(graph.node("kaya/Artifact/designs-shared").is_some());
    assert!(graph.node("kaya/Artifact/research-shared").is_some());
}

#[test]
fn subject_ids_separate_wai_source_projects() {
    let temp = tempfile::tempdir().unwrap();
    write_file(
        temp.path(),
        ".wai/projects/kaya/designs/shared.md",
        "# kaya\n",
    );
    write_file(
        temp.path(),
        ".wai/projects/other/designs/shared.md",
        "# other\n",
    );

    let events = wai::capture(temp.path(), PROJECT).unwrap();
    let subjects: Vec<&str> = events
        .iter()
        .map(|event| event.subject.id.as_str())
        .collect();
    assert!(subjects.contains(&"kaya/Artifact/designs-shared"));
    assert!(subjects.contains(&"other/Artifact/designs-shared"));

    let graph = model::build(&events).expect("both source projects enter the model");
    assert_eq!(graph.nodes().count(), 2);
}

#[test]
fn event_id_digests_the_project_namespace() {
    let temp = tempfile::tempdir().unwrap();
    fixture(temp.path());

    let first = wai::capture(temp.path(), PROJECT).unwrap();
    let second = wai::capture(temp.path(), "other-project").unwrap();

    assert_eq!(first.len(), second.len());
    for event in &first {
        let reference = event.provenance.reference.as_str();
        let counterpart = second
            .iter()
            .find(|candidate| candidate.provenance.reference == reference)
            .expect("both captures cover the same records");
        assert_eq!(
            event.data, counterpart.data,
            "the same source produces the same payload"
        );
        assert_ne!(
            event.id, counterpart.id,
            "the project namespace is part of the event identity"
        );
    }
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

// kaya-q5e: content-derived capture attributes (bounded first-heading title
// and digest). RED-first pins for the planned derivation in plan.md.

#[test]
fn capture_extracts_first_hash_space_heading_as_title() {
    let temp = tempfile::tempdir().unwrap();
    let record = ".wai/projects/kaya/designs/2026-04-01-titled.md";
    write_file(temp.path(), record, "intro prose\n\n# the real heading\n\nbody stays wai's\n");

    let events = wai::capture(temp.path(), PROJECT).unwrap();
    let event = by_path(&events, record);

    assert_eq!(event.data["title"], "the real heading");
    let digest = event.data["title_digest"].as_str().expect("digest present");
    let expected = &kaya::adapters::sha256_hex("the real heading".as_bytes())[..16];
    assert_eq!(digest, expected);
    assert_eq!(event.data["title_digest"].as_str().unwrap().len(), 16);
}

#[test]
fn capture_truncates_multibyte_title_without_char_boundary_panic() {
    let temp = tempfile::tempdir().unwrap();
    let record = ".wai/projects/kaya/research/2026-04-02-unicode.md";
    // Each emoji is four bytes; byte-slicing at a non-boundary would panic.
    // 90 characters exceeds TITLE_MAX_LEN (80), so the title truncates.
    let heading = format!("# {}\n", "🖖".repeat(90));

    write_file(temp.path(), record, &heading);
    let events = wai::capture(temp.path(), PROJECT).unwrap();
    let event = by_path(&events, record);

    let title = event.data["title"].as_str().expect("title present");
    assert!(title.is_char_boundary(title.len()), "cut lands on a boundary");
    assert_eq!(title.chars().count(), 80);
}

#[test]
fn capture_omits_title_attributes_when_file_empty_or_heading_less() {
    let temp = tempfile::tempdir().unwrap();
    let empty = ".wai/projects/kaya/designs/2026-04-03-empty.md";
    let prose = ".wai/projects/kaya/designs/2026-04-04-prose-only.md";
    write_file(temp.path(), empty, "");
    write_file(temp.path(), prose, "just prose, no heading\nsecond line\n");

    let events = wai::capture(temp.path(), PROJECT).unwrap();

    for record in [empty, prose] {
        let event = by_path(&events, record);
        let data = event.data.as_object().expect("data is an object");
        assert!(!data.contains_key("title"), "no title for {record}");
        assert!(!data.contains_key("title_digest"), "no digest for {record}");
    }
}

#[test]
fn capture_truncates_long_title_to_the_bounded_constant() {
    let temp = tempfile::tempdir().unwrap();
    let record = ".wai/projects/kaya/handoffs/2026-04-05-long.md";
    // Planned bound: wai::TITLE_MAX_LEN == 80 (src/adapters/wai.rs, green).
    write_file(
        temp.path(),
        record,
        "# a very long heading that goes well beyond the bounded title length and keeps going far past eighty characters total\n",
    );

    let events = wai::capture(temp.path(), PROJECT).unwrap();
    let event = by_path(&events, record);

    let title = event.data["title"].as_str().expect("title present");
    assert_eq!(title.chars().count(), wai::TITLE_MAX_LEN);
    assert!(
        title.starts_with("a very long heading"),
        "title derives from the heading: {title}"
    );
}

#[test]
fn renamed_path_event_id_keeps_stem_digest_stability() {
    let temp = tempfile::tempdir().unwrap();
    let record = ".wai/projects/kaya/designs/2026-04-05-stability.md";
    write_file(temp.path(), record, "# first title\n");
    let first = wai::capture(temp.path(), PROJECT).unwrap();
    let before = by_path(&first, record);

    // Only the content-derived title changes; the id must not move.
    write_file(temp.path(), record, "# a completely different title\n");
    let second = wai::capture(temp.path(), PROJECT).unwrap();
    let after = by_path(&second, record);

    assert_eq!(before.id, after.id, "id stays path-derived, not content-derived");
    assert_eq!(before.subject.id, after.subject.id);
    assert_ne!(before.data["title"], after.data["title"], "the edit is real");
}
