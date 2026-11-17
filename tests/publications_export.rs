//! Contract tests for the three remaining publications: each emits only from
//! its admission row, the Markdown sections and the structured export trace
//! every document to the same event ids, and two publishes over unchanged
//! inputs are byte-identical.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde_json::{Value, json};
use tempfile::TempDir;

use kaya::projection::{BlockKind, scan};
use kaya::publication::{Decl, DocumentRef, Documents, Tier, admission, build};

const OVERVIEW_KIND: &str = "orientation.project-overview";
const RECORD_KIND: &str = "exploration.experiment-record";
const DECISION_KIND: &str = "understanding.decision-record";
const MAP_KIND: &str = "orientation.research-map";

const OVERVIEW_BODY: &str = "The overview places kaya/Question/q-one at the centre of the record.";
const RECORD_BODY: &str = "The record lists kaya/Experiment/exp-one and kaya/Run/run-one";
const DECISION_BODY: &str = "The decision pins kaya/Decision/dec-cache";
const MAP_BODY: &str = "The map links kaya/Question/q-one to kaya/Experiment/exp-one";

/// The three remaining products, with the admission row each one is bound to,
/// verbatim from the capability spec and `docs/src/document-taxonomy.md`.
const REMAINING_PRODUCTS: [(&str, &[&str]); 3] = [
    ("status-compatibility", &[OVERVIEW_KIND, RECORD_KIND]),
    ("release-migration", &[OVERVIEW_KIND, DECISION_KIND]),
    ("architecture-design", &[DECISION_KIND, MAP_KIND]),
];

/// The compiled documents the three products admit, each attributing exactly
/// the node ids its body names.
const DOCUMENTS: [(&str, &str, &[&str]); 4] = [
    (OVERVIEW_KIND, OVERVIEW_BODY, &["kaya/Question/q-one"]),
    (
        RECORD_KIND,
        RECORD_BODY,
        &["kaya/Experiment/exp-one", "kaya/Run/run-one"],
    ),
    (DECISION_KIND, DECISION_BODY, &["kaya/Decision/dec-cache"]),
    (
        MAP_KIND,
        MAP_BODY,
        &["kaya/Experiment/exp-one", "kaya/Question/q-one"],
    ),
];

/// The document source keyed by generated-block name (the document kind).
fn documents(entries: &[(&str, &str, &[&str])]) -> Documents {
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

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

fn product(name: &str, includes: &[&str]) -> Decl {
    Decl {
        tier: Tier::Product,
        name: name.to_owned(),
        includes: strings(includes),
        members: Vec::new(),
    }
}

/// The three remaining products, each including exactly its admission row.
fn remaining_declarations() -> Vec<Decl> {
    REMAINING_PRODUCTS
        .iter()
        .map(|(name, row)| product(name, row))
        .collect()
}

/// Every regular file under `root`, keyed by its relative path.
fn read_tree(root: &Path) -> BTreeMap<String, Vec<u8>> {
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
fn markdown(tree: &BTreeMap<String, Vec<u8>>) -> BTreeMap<String, String> {
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

/// The body of the single generated block named `block` in `document`.
fn block_body(document: &str, block: &str) -> String {
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

/// The `### <kind>` sections of a rendered product block body, in page order.
fn rendered_sections(body: &str) -> Vec<(String, String)> {
    let mut sections: Vec<(String, String)> = Vec::new();
    for line in body.lines() {
        if let Some(kind) = line.strip_prefix("### ") {
            sections.push((kind.trim().to_owned(), String::new()));
        } else if let Some((_, buffer)) = sections.last_mut() {
            buffer.push_str(line);
            buffer.push('\n');
        }
    }
    sections
}

/// The `kaya/...` node ids named in a body, sorted and deduplicated.
///
/// `@` stays inside the token so a Source reference keeps its `@commit` suffix:
/// `kaya/Source/docs-src-index.md@43b606f` is one id, never the bare path.
fn body_node_ids(body: &str) -> Vec<String> {
    let mut ids: Vec<String> = body
        .split(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '/' | '-' | '_' | '.' | '@')))
        .filter(|token| token.starts_with("kaya/"))
        .map(str::to_owned)
        .collect();
    ids.sort();
    ids.dedup();
    ids
}

// 1. kaya.publications/Reader-question binding — the remaining three products
#[test]
fn remaining_products_emit_from_admission_rows() {
    for (name, row) in REMAINING_PRODUCTS {
        assert_eq!(
            admission(name),
            row,
            "the admission row for {name} is fixed, in declaration order"
        );
    }

    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    let decls = remaining_declarations();
    let docs = documents(&DOCUMENTS);

    let built = match build(&decls, &docs, &out) {
        Ok(built) => built,
        Err(error) => panic!(
            "the remaining products must build: {}: {}",
            error.label, error.message
        ),
    };

    let names: Vec<&str> = built.pages.iter().map(|page| page.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "architecture-design",
            "release-migration",
            "status-compatibility"
        ],
        "the three backed products get a page each, sorted by name"
    );
    assert!(
        built.omitted.is_empty(),
        "every product has an admissible backing document, so none is omitted: {:?}",
        built.omitted
    );

    // One rendering per fact: page order decides the holder, and every later
    // product cross-references it instead of repeating the body.
    let expected = [
        (
            "architecture-design",
            "publication.product.architecture-design",
            format!(
                concat!(
                    "## Architecture-design\n\n",
                    "Reader question: how is it structured and why\n\n",
                    "### understanding.decision-record\n\n",
                    "{}\n\n",
                    "### orientation.research-map\n\n",
                    "{}\n\n",
                ),
                DECISION_BODY, MAP_BODY
            ),
        ),
        (
            "release-migration",
            "publication.product.release-migration",
            format!(
                concat!(
                    "## Release-migration\n\n",
                    "Reader question: what changed and what must I do\n\n",
                    "### orientation.project-overview\n\n",
                    "{}\n\n",
                    "### understanding.decision-record\n\n",
                    "See [architecture-design](architecture-design.md#understandingdecision-record).\n\n",
                ),
                OVERVIEW_BODY
            ),
        ),
        (
            "status-compatibility",
            "publication.product.status-compatibility",
            format!(
                concat!(
                    "## Status-compatibility\n\n",
                    "Reader question: what can I rely on\n\n",
                    "### orientation.project-overview\n\n",
                    "See [release-migration](release-migration.md#orientationproject-overview).\n\n",
                    "### exploration.experiment-record\n\n",
                    "{}\n\n",
                ),
                RECORD_BODY
            ),
        ),
    ];

    let tree = read_tree(&out);
    let pages = markdown(&tree);
    for (name, block, content) in &expected {
        let path = format!("{name}.md");
        let page = pages
            .get(&path)
            .unwrap_or_else(|| panic!("the {name} page is written"));
        let expected_file = format!(
            "<!-- kaya:begin generated:{block} -->\n{content}<!-- kaya:end generated:{block} -->\n"
        );
        assert_eq!(
            page.as_str(),
            expected_file.as_str(),
            "the {name} page is exactly its managed block: {page}"
        );
        assert_eq!(
            block_body(page, block).as_str(),
            content.as_str(),
            "the {name} block renders its admitted documents in declaration order: {page}"
        );

        let kinds: Vec<String> = rendered_sections(content)
            .into_iter()
            .map(|(kind, _)| kind)
            .collect();
        assert_eq!(
            kinds,
            strings(admission(name)),
            "the {name} page names exactly its admission row, no kind outside it: {page}"
        );
    }

    // Each admitted body is rendered exactly once, so the cross-references
    // above point at the only page that carries the facts.
    for body in [OVERVIEW_BODY, RECORD_BODY, DECISION_BODY, MAP_BODY] {
        let occurrences: usize = pages.values().map(|text| text.matches(body).count()).sum();
        assert_eq!(
            occurrences, 1,
            "the body {body:?} is rendered exactly once across the published tree"
        );
    }

    // A product without an admissible backing document is omitted, not
    // stubbed: a decision record backs the two products that admit it, while
    // status-compatibility admits only the overview and the experiment record.
    let partial_out = tmp.path().join("partial");
    let partial_docs = documents(&[(DECISION_KIND, DECISION_BODY, &["kaya/Decision/dec-cache"])]);
    let partial = match build(&decls, &partial_docs, &partial_out) {
        Ok(partial) => partial,
        Err(error) => panic!(
            "the partial composition must build: {}: {}",
            error.label, error.message
        ),
    };
    let partial_pages: Vec<&str> = partial
        .pages
        .iter()
        .map(|page| page.name.as_str())
        .collect();
    assert_eq!(
        partial_pages,
        ["architecture-design", "release-migration"],
        "only the products with a backing document emit"
    );
    let omitted: Vec<(&str, &str, &str)> = partial
        .omitted
        .iter()
        .map(|entry| (entry.tier, entry.name.as_str(), entry.reason))
        .collect();
    assert_eq!(
        omitted,
        [("product", "status-compatibility", "no-backing-document")],
        "the unbacked product is reported omitted with its reason"
    );
    assert!(
        !partial_out.join("status-compatibility.md").exists(),
        "an omitted product is never stubbed into a page"
    );
}

// 2. kaya.publications/One source, many formats — format attribution
#[test]
fn format_attribution_matches_every_page_section() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    let decls = remaining_declarations();
    let docs = documents(&DOCUMENTS);

    let built = match build(&decls, &docs, &out) {
        Ok(built) => built,
        Err(error) => panic!(
            "the attribution composition must build: {}: {}",
            error.label, error.message
        ),
    };

    let export: Value = serde_json::from_str(
        &fs::read_to_string(out.join("publication.json")).expect("the export is written"),
    )
    .expect("the export is JSON");
    let products = export["products"]
        .as_array()
        .expect("the export lists products");

    let emitted: Vec<&str> = built
        .pages
        .iter()
        .filter(|page| page.question.is_some())
        .map(|page| page.name.as_str())
        .collect();
    let exported: Vec<&str> = products
        .iter()
        .filter(|product| product["emitted"] == json!(true))
        .map(|product| {
            product["name"]
                .as_str()
                .expect("each exported product names itself")
        })
        .collect();
    assert_eq!(
        exported, emitted,
        "the export names exactly the pages this one build emitted"
    );

    // The document attribution the export carries, kind by kind, kept per
    // product as its own row in declaration order. Every product composing the
    // same document must agree on its event ids; that union is only the
    // cross-product check and the final all-pages comparison, never the
    // expected composition of one product.
    let mut rows: BTreeMap<String, Vec<(String, Vec<String>)>> = BTreeMap::new();
    let mut declared: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for product in products {
        let name = product["name"]
            .as_str()
            .expect("each exported product names itself");
        let documents = product["documents"]
            .as_array()
            .expect("each exported product lists its documents");
        if product["emitted"] == json!(false) {
            assert!(
                documents.is_empty(),
                "an omitted product attributes no document: {product}"
            );
            continue;
        }
        assert!(
            !documents.is_empty(),
            "an emitted product attributes at least one document: {product}"
        );
        let mut row: Vec<(String, Vec<String>)> = Vec::new();
        for document in documents {
            let kind = document["kind"]
                .as_str()
                .expect("each exported document names its kind")
                .to_owned();
            let ids: Vec<String> = document["event_ids"]
                .as_array()
                .expect("each exported document lists its event ids")
                .iter()
                .map(|id| {
                    id.as_str()
                        .expect("every exported event id is a string")
                        .to_owned()
                })
                .collect();
            assert!(
                !ids.is_empty(),
                "{name} attributes {kind} to at least one event id"
            );
            let existing = declared.entry(kind.clone()).or_insert_with(|| ids.clone());
            assert_eq!(
                existing.as_slice(),
                ids.as_slice(),
                "every product composing {kind} attributes the same event ids"
            );
            row.push((kind, ids));
        }
        rows.insert(name.to_owned(), row);
    }
    assert_eq!(
        declared.len(),
        DOCUMENTS.len(),
        "the export attributes exactly the compiled documents the products compose: {declared:?}"
    );

    // The documents this build can compose, and so the most any product's
    // export row may attribute: every product declares its full admission row
    // and this fixture backs every admitted document.
    let backed: BTreeSet<&str> = DOCUMENTS.iter().map(|(kind, _, _)| *kind).collect();

    // The same attribution parsed back out of the Markdown: each document is
    // rendered as a `### <kind>` section on exactly one page, and that
    // section's node ids are the ids its own product's export row carries.
    let tree = read_tree(&out);
    let pages = markdown(&tree);
    let mut rendered: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for name in &emitted {
        let page = pages
            .get(&format!("{name}.md"))
            .unwrap_or_else(|| panic!("the {name} page is written"));
        let block = format!("publication.product.{name}");
        let sections = rendered_sections(&block_body(page, &block));
        let row = rows
            .get(*name)
            .unwrap_or_else(|| panic!("the export attributes a document to the {name} page"));
        let expected_kinds: Vec<&str> = admission(name)
            .iter()
            .copied()
            .filter(|kind| backed.contains(*kind))
            .collect();
        let exported_kinds: Vec<&str> = row.iter().map(|(kind, _)| kind.as_str()).collect();
        assert_eq!(
            exported_kinds, expected_kinds,
            "the {name} export row composes exactly the admitted documents this build backs, in declaration order"
        );
        let kinds: Vec<String> = sections.iter().map(|(kind, _)| kind.clone()).collect();
        assert_eq!(
            kinds, expected_kinds,
            "the {name} page names each composed document once, in declaration order"
        );
        // `kinds == exported_kinds` above, so each section pairs with its own
        // row entry in the same position.
        for ((kind, section), (_, exported_ids)) in sections.iter().zip(row.iter()) {
            let ids = body_node_ids(section);
            if ids.is_empty() {
                assert!(
                    section.trim_start().starts_with("See ["),
                    "a section without ids cross-references the page rendering {kind}: {section}"
                );
                continue;
            }
            assert_eq!(
                ids.as_slice(),
                exported_ids.as_slice(),
                "the {name} page section for {kind} cites exactly its own product's exported event ids"
            );
            assert!(
                rendered.insert(kind.clone(), ids).is_none(),
                "exactly one page renders {kind}, so its facts appear once"
            );
        }
    }
    assert_eq!(
        rendered, declared,
        "the pages and the export trace every document to the same event ids"
    );
}

// 3. kaya.publications/One source, many formats — deterministic rebuild
#[test]
fn repeat_publish_is_byte_identical() {
    let tmp = TempDir::new().expect("temp dir");
    let decls = remaining_declarations();
    let docs = documents(&DOCUMENTS);

    let out = tmp.path().join("out");
    let first = match build(&decls, &docs, &out) {
        Ok(first) => first,
        Err(error) => panic!(
            "the first publish must build: {}: {}",
            error.label, error.message
        ),
    };
    assert!(
        first
            .summary
            .recomputed
            .contains(&"publication.json".to_owned()),
        "a fresh publish recomputes the export: {:?}",
        first.summary.recomputed
    );
    assert!(
        first.summary.unchanged.is_empty(),
        "a fresh publish has nothing already current: {:?}",
        first.summary.unchanged
    );

    let tree = read_tree(&out);
    let export = fs::read(out.join("publication.json")).expect("the export is readable");
    assert_eq!(
        markdown(&tree).len(),
        REMAINING_PRODUCTS.len(),
        "the tree holds one page per emitted product"
    );

    // Publishing again from unchanged inputs touches no file and leaves every
    // byte, export included, exactly as the first publish wrote it.
    let second = match build(&decls, &docs, &out) {
        Ok(second) => second,
        Err(error) => panic!(
            "the second publish must build: {}: {}",
            error.label, error.message
        ),
    };
    assert!(
        second.summary.recomputed.is_empty(),
        "a second publish recomputes nothing: {:?}",
        second.summary.recomputed
    );
    assert_eq!(
        second.summary.unchanged.len(),
        REMAINING_PRODUCTS.len() + 1,
        "a second publish reports every page and the export unchanged: {:?}",
        second.summary.unchanged
    );
    assert_eq!(
        read_tree(&out),
        tree,
        "publishing twice leaves every page and the export byte-identical"
    );
    assert_eq!(
        fs::read(out.join("publication.json")).expect("the export is readable"),
        export,
        "publishing twice leaves the export byte-identical"
    );

    // Two independent publishes from the same inputs produce the same tree:
    // attribution is a function of the source, never of build history.
    let fresh = tmp.path().join("fresh");
    match build(&decls, &docs, &fresh) {
        Ok(_) => {}
        Err(error) => panic!(
            "the independent publish must build: {}: {}",
            error.label, error.message
        ),
    }
    assert_eq!(
        read_tree(&fresh),
        tree,
        "two publishes from unchanged inputs produce identical pages and export"
    );
}
