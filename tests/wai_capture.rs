//! Contract tests for the wai adapter's bounded content-derived attributes:
//! the first `# ` heading becomes a bounded title plus its digest, the
//! attributes are absent when the artifact carries no heading, and the
//! content-derived attributes never move the event id.

use std::fs;
use std::path::Path;

use kaya::adapters::wai;
use kaya::event::Event;

const PROJECT: &str = "kaya";

/// Write one artifact below the fixture root.
fn write_file(root: &Path, relative: &str, contents: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().expect("fixture file has a parent")).unwrap();
    fs::write(path, contents).unwrap();
}

/// The event captured for one relative path.
fn by_path<'a>(events: &'a [Event], relative: &str) -> &'a Event {
    events
        .iter()
        .find(|event| event.data["path"] == relative)
        .unwrap_or_else(|| panic!("no event for {relative}"))
}

#[test]
fn capture_extracts_first_hash_space_heading_as_title() {
    let temp = tempfile::tempdir().unwrap();
    let record = ".wai/projects/kaya/designs/2026-04-01-titled.md";
    write_file(
        temp.path(),
        record,
        "intro prose\n\n# the real heading\n\nbody stays wai's\n",
    );

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
