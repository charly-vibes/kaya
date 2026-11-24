//! Shared fixtures, constants, and tree readers for the publication suite.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde_json::Value;

use kaya::projection::{BlockKind, scan};
use kaya::publication::{Decl, DocumentRef, Documents, Tier};

pub(crate) const OVERVIEW_KIND: &str = "orientation.project-overview";
pub(crate) const MAP_KIND: &str = "orientation.research-map";
pub(crate) const TECH_KIND: &str = "reuse-handoff.technical-reference";
pub(crate) const SPEC_KIND: &str = "exploration.experiment-specification";
pub(crate) const FINDINGS_KIND: &str = "understanding.findings-and-analysis";
pub(crate) const DECISION_KIND: &str = "understanding.decision-record";

/// The heading anchors cross-references point at: the document kind with every
/// character other than an alphanumeric, hyphen, or underscore removed.
pub(crate) const OVERVIEW_ANCHOR: &str = "orientationproject-overview";
pub(crate) const FINDINGS_ANCHOR: &str = "understandingfindings-and-analysis";
pub(crate) const DECISION_ANCHOR: &str = "understandingdecision-record";

pub(crate) const OVERVIEW_BODY: &str = "Kaya publishes client-facing documents from one source.";
pub(crate) const MAP_BODY: &str = "The research map links each fact to its event.";
pub(crate) const TECH_BODY: &str = "Reuse begins from the recorded handoff.";
pub(crate) const SPEC_BODY: &str = "The experiment specification states the intended change.";
pub(crate) const FINDINGS_BODY: &str = "The findings record what the runs taught the project.";
pub(crate) const DECISION_BODY: &str = "The decision record pins the shipped cache revision.";

/// The closed document vocabulary an admission row may name.
pub(crate) const DOCUMENT_KINDS: [&str; 11] = [
    "orientation.project-overview",
    "orientation.research-map",
    "exploration.exploration-brief",
    "exploration.experiment-specification",
    "exploration.experiment-record",
    "understanding.findings-and-analysis",
    "understanding.decision-record",
    "reuse-handoff.technical-reference",
    "reuse-handoff.reproduction-package",
    "reuse-handoff.knowledge-brief",
    "reuse-handoff.handoff",
];

/// The fixed reader-question binding, verbatim from the capability spec.
pub(crate) const QUESTIONS: [(&str, &str); 12] = [
    ("overview", "what is this and why does it matter"),
    ("getting-started", "how do I reach a first result"),
    ("tutorial", "can you teach me through a guided outcome"),
    ("how-to", "how do I accomplish this task"),
    ("concept", "why does it work this way"),
    ("reference", "what exactly is this"),
    ("example", "what can I do with it"),
    ("troubleshooting", "why is this broken and how do I fix it"),
    ("faq", "what do people keep asking"),
    ("status-compatibility", "what can I rely on"),
    ("release-migration", "what changed and what must I do"),
    ("architecture-design", "how is it structured and why"),
];

/// The full admission table, pinned verbatim from the capability spec and
/// `docs/src/document-taxonomy.md`: each product admits exactly these document
/// kinds, in this order.
pub(crate) const ADMISSIONS: [(&str, &[&str]); 12] = [
    ("overview", &["orientation.project-overview"]),
    (
        "getting-started",
        &[
            "orientation.project-overview",
            "reuse-handoff.reproduction-package",
        ],
    ),
    (
        "tutorial",
        &[
            "exploration.experiment-record",
            "exploration.exploration-brief",
        ],
    ),
    (
        "how-to",
        &[
            "reuse-handoff.technical-reference",
            "reuse-handoff.reproduction-package",
        ],
    ),
    (
        "concept",
        &[
            "understanding.decision-record",
            "understanding.findings-and-analysis",
        ],
    ),
    (
        "reference",
        &[
            "reuse-handoff.technical-reference",
            "orientation.research-map",
        ],
    ),
    ("example", &["exploration.experiment-record"]),
    ("troubleshooting", &["understanding.findings-and-analysis"]),
    (
        "faq",
        &[
            "understanding.findings-and-analysis",
            "understanding.decision-record",
        ],
    ),
    (
        "status-compatibility",
        &[
            "orientation.project-overview",
            "exploration.experiment-record",
        ],
    ),
    (
        "release-migration",
        &[
            "orientation.project-overview",
            "understanding.decision-record",
        ],
    ),
    (
        "architecture-design",
        &["understanding.decision-record", "orientation.research-map"],
    ),
];
pub(crate) fn kind_label(tier: Tier) -> &'static str {
    match tier {
        Tier::Product => "product",
        Tier::Collection => "collection",
        Tier::Package => "package",
    }
}

/// The document source keyed by generated-block name (the document kind).
pub(crate) fn documents(entries: &[(&str, &str, &[&str])]) -> Documents {
    let mut map = BTreeMap::new();
    for (kind, body, event_ids) in entries {
        map.insert(
            (*kind).to_owned(),
            DocumentRef {
                kind: (*kind).to_owned(),
                body: (*body).to_owned(),
                event_ids: event_ids.iter().map(|id| (*id).to_owned()).collect(),
            },
        );
    }
    Documents(map)
}

pub(crate) fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

pub(crate) fn product(name: &str, includes: &[&str]) -> Decl {
    Decl {
        tier: Tier::Product,
        name: name.to_owned(),
        includes: strings(includes),
        members: Vec::new(),
    }
}

pub(crate) fn package(name: &str, members: &[&str]) -> Decl {
    Decl {
        tier: Tier::Package,
        name: name.to_owned(),
        includes: Vec::new(),
        members: strings(members),
    }
}

/// A collection declaration: collections take no fields, so only the name.
pub(crate) fn collection(name: &str) -> Decl {
    Decl {
        tier: Tier::Collection,
        name: name.to_owned(),
        includes: Vec::new(),
        members: Vec::new(),
    }
}

/// Every regular file under `root`, keyed by its relative path.
pub(crate) fn read_tree(root: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(root: &Path, dir: &Path, files: &mut BTreeMap<String, Vec<u8>>) {
        let mut paths: Vec<_> = fs::read_dir(dir)
            .expect("publication tree is readable")
            .map(|entry| entry.expect("directory entry is readable").path())
            .collect();
        paths.sort();
        for path in paths {
            if path.is_dir() {
                walk(root, &path, files);
            } else {
                let relative = path
                    .strip_prefix(root)
                    .expect("tree paths stay under the output root")
                    .to_string_lossy()
                    .into_owned();
                files.insert(relative, fs::read(&path).expect("tree file is readable"));
            }
        }
    }

    let mut files = BTreeMap::new();
    walk(root, root, &mut files);
    files
}

/// The Markdown pages of a built tree, keyed by relative path.
pub(crate) fn markdown(tree: &BTreeMap<String, Vec<u8>>) -> BTreeMap<String, String> {
    tree.iter()
        .filter(|(path, _)| path.ends_with(".md"))
        .map(|(path, bytes)| {
            (
                (*path).clone(),
                String::from_utf8((*bytes).clone()).expect("generated pages are UTF-8"),
            )
        })
        .collect()
}

pub(crate) fn assert_no_writes(out: &Path) {
    assert!(
        !out.join("publication.json").exists(),
        "a refused build must not write the export"
    );
    let entries: Vec<_> = fs::read_dir(out)
        .expect("the pre-created output directory is readable")
        .collect();
    assert!(
        entries.is_empty(),
        "a refused build must not write a single page"
    );
}

/// The body of the single generated block named `block` in `document`.
pub(crate) fn block_body(document: &str, block: &str) -> String {
    let mut bodies = scan(document)
        .expect("a written page has a valid managed-block grammar")
        .into_iter()
        .filter_map(|entry| match entry.kind {
            BlockKind::Generated(name) if name == block => Some(entry.body),
            _ => None,
        });
    let body = bodies
        .next()
        .unwrap_or_else(|| panic!("the page carries the {block} block: {document}"));
    assert!(
        bodies.next().is_none(),
        "the page carries exactly one {block} block: {document}"
    );
    body
}

/// The bytes `replace_generated` produces for a new page with `body`.
pub(crate) fn wrapped_page(block: &str, body: &str) -> String {
    format!("<!-- kaya:begin generated:{block} -->\n{body}<!-- kaya:end generated:{block} -->\n")
}

/// The `### <kind>` sections of a rendered product block body.
pub(crate) fn rendered_sections(body: &str) -> BTreeMap<String, String> {
    let mut sections = BTreeMap::new();
    for (kind, section) in section_pairs(body) {
        sections.insert(kind, section);
    }
    sections
}

/// Split a rendered body into `(kind, section)` pairs in document order: each
/// `### <kind>` header starts a section holding the lines after it.
fn section_pairs(body: &str) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    for line in body.lines() {
        if let Some(kind) = line.strip_prefix("### ") {
            pairs.push((kind.trim().to_owned(), String::new()));
        } else {
            append_to_last_section(&mut pairs, line);
        }
    }
    pairs
}

/// Append one body line to the section being built, ignoring bytes before the
/// first `### <kind>` header.
fn append_to_last_section(pairs: &mut [(String, String)], line: &str) {
    if let Some((_, section)) = pairs.last_mut() {
        section.push_str(line);
        section.push('\n');
    }
}

/// The `kaya/...` node ids named in a body, sorted and deduplicated.
pub(crate) fn body_node_ids(body: &str) -> Vec<String> {
    let mut ids: Vec<String> = body
        .split(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '/' | '-' | '_' | '.')))
        .filter(|token| token.starts_with("kaya/"))
        .map(str::to_owned)
        .collect();
    ids.sort();
    ids.dedup();
    ids
}

/// A JSON object's keys, sorted.
pub(crate) fn object_keys(value: &Value) -> Vec<&str> {
    let mut keys: Vec<&str> = value
        .as_object()
        .unwrap_or_else(|| panic!("expected a JSON object: {value}"))
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort();
    keys
}

/// The project-site package block: a heading plus exactly one row per member,
/// in order.
pub(crate) fn assert_package_member_rows(package_page: &str, expected_rows: &[&str]) {
    let package_body = block_body(package_page, "publication.package.project-site");
    let rows: Vec<&str> = package_body
        .lines()
        .filter(|line| !line.is_empty())
        .collect();
    assert_eq!(
        rows.len(),
        expected_rows.len() + 1,
        "the package block is a heading and one row per member: {package_page}"
    );
    assert!(
        rows[0].starts_with("## "),
        "the package block opens with a heading: {package_page}"
    );
    for (row, expected) in rows[1..].iter().zip(expected_rows.iter()) {
        assert_eq!(
            row, expected,
            "the package lists its members: {package_page}"
        );
    }
}
