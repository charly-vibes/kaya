//! Determinism: identical inputs rebuild identically, and the export
//! carries the same facts the pages do.

use std::collections::BTreeMap;
use std::path::Path;

use serde_json::{Value, json};
use tempfile::TempDir;

use kaya::publication::{Decl, Documents, build};

use crate::common::*;

fn exported_kind<'a>(objects: &[&'a Value], kind: &str) -> &'a Value {
    objects
        .iter()
        .copied()
        .find(|object| object.get("kind").and_then(Value::as_str) == Some(kind))
        .unwrap_or_else(|| panic!("the export attributes every source kind, missing {kind}"))
}

/// The exported package member naming `name`; members carry only a name and
/// an `emitted` flag, unlike product rows which also carry `documents`.
fn exported_member<'a>(objects: &[&'a Value], name: &str) -> &'a Value {
    objects
        .iter()
        .copied()
        .find(|object| {
            object.get("name").and_then(Value::as_str) == Some(name)
                && object.get("emitted").is_some()
                && object.get("documents").is_none()
                && object.get("question").is_none()
        })
        .unwrap_or_else(|| panic!("the export lists package member {name}"))
}

/// Collect every JSON object reachable from `value`, whatever its nesting.
fn collect_objects<'a>(value: &'a Value, objects: &mut Vec<&'a Value>) {
    match value {
        Value::Object(map) => {
            objects.push(value);
            for child in map.values() {
                collect_objects(child, objects);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_objects(item, objects);
            }
        }
        _ => {}
    }
}

// 7. kaya.publications/Publication tiers — Product and package composed
/// The project-site declaration over the technical and map facts, built into
/// both directories; returns both trees.
fn build_project_site_twice(
    first: &Path,
    second: &Path,
) -> (BTreeMap<String, Vec<u8>>, BTreeMap<String, Vec<u8>>) {
    let decls = [
        product("overview", &[OVERVIEW_KIND]),
        product("reference", &[TECH_KIND, MAP_KIND]),
        package("project-site", &["overview", "reference"]),
    ];
    let docs = documents(&[
        (TECH_KIND, TECH_BODY, &["evt-tech"]),
        (MAP_KIND, MAP_BODY, &["evt-map"]),
    ]);
    for out in [first, second] {
        match build(&decls, &docs, out) {
            Ok(_) => {}
            Err(error) => panic!(
                "the project site must build: {}: {}",
                error.label, error.message
            ),
        }
    }
    (read_tree(first), read_tree(second))
}

/// The export attributes each fact to its event ids, binds the reader
/// question, and reports package member availability.
fn assert_export_shape(objects: &[&Value]) {
    assert_eq!(
        exported_kind(objects, MAP_KIND).get("event_ids"),
        Some(&json!(["evt-map"])),
        "the export attributes the research-map fact to its event ids"
    );
    assert_eq!(
        exported_kind(objects, TECH_KIND).get("event_ids"),
        Some(&json!(["evt-tech"])),
        "the export attributes the technical-reference fact to its event ids"
    );

    assert!(
        objects
            .iter()
            .any(|object| object.get("question").and_then(Value::as_str)
                == Some("what exactly is this")),
        "the export binds the reference product to its reader question"
    );

    assert_eq!(
        exported_member(objects, "overview")
            .get("emitted")
            .and_then(Value::as_bool),
        Some(false),
        "the export reports the overview member as not emitted"
    );
    assert_eq!(
        exported_member(objects, "reference")
            .get("emitted")
            .and_then(Value::as_bool),
        Some(true),
        "the export reports the reference member as emitted"
    );
    assert!(
        objects.iter().any(|object| {
            object.get("name").and_then(Value::as_str) == Some("project-site")
                && object.get("members").is_some_and(Value::is_array)
        }),
        "the export carries the project-site package with its members"
    );
}

#[test]
fn markdown_and_export_share_one_source_and_rebuild_identically() {
    let tmp = TempDir::new().expect("temp dir");
    let (first, second) = (tmp.path().join("first"), tmp.path().join("second"));
    let (first_tree, second_tree) = build_project_site_twice(&first, &second);

    assert_eq!(
        first_tree.keys().collect::<Vec<_>>(),
        second_tree.keys().collect::<Vec<_>>(),
        "both builds project the same files"
    );
    for (path, bytes) in &first_tree {
        assert_eq!(
            bytes, &second_tree[path],
            "{path} rebuilds byte-identically from unchanged inputs"
        );
    }

    let decls_docs = project_site_inputs();
    match build(&decls_docs.0, &decls_docs.1, &first) {
        Ok(_) => {}
        Err(error) => panic!(
            "the project site must rebuild over its own output: {}: {}",
            error.label, error.message
        ),
    }
    assert_eq!(
        first_tree,
        read_tree(&first),
        "a rebuild over unchanged inputs leaves every already-written byte in place"
    );

    let pages = markdown(&first_tree);
    let reference_page = pages
        .get("reference.md")
        .expect("the reference product page is written");
    assert!(
        reference_page.contains(MAP_BODY),
        "the reference page carries the research-map body: {reference_page}"
    );
    assert!(
        reference_page.contains(TECH_BODY),
        "the reference page carries the technical-reference body: {reference_page}"
    );

    let export_bytes = first_tree
        .get("publication.json")
        .expect("the structured export is written");
    let export: Value =
        serde_json::from_slice(export_bytes).expect("the export is a JSON document");
    let mut objects = Vec::new();
    collect_objects(&export, &mut objects);
    assert_export_shape(&objects);
}

/// The project-site declaration and the technical/map documents it composes.
fn project_site_inputs() -> ([Decl; 3], Documents) {
    let decls = [
        product("overview", &[OVERVIEW_KIND]),
        product("reference", &[TECH_KIND, MAP_KIND]),
        package("project-site", &["overview", "reference"]),
    ];
    let docs = documents(&[
        (TECH_KIND, TECH_BODY, &["evt-tech"]),
        (MAP_KIND, MAP_BODY, &["evt-map"]),
    ]);
    (decls, docs)
}

/// The project site built without a backing overview document: the overview
/// product is omitted, the reference product and the package are emitted.
fn project_site_built(out: &Path) -> kaya::publication::Built {
    let (decls, docs) = project_site_inputs();
    match build(&decls, &docs, out) {
        Ok(built) => built,
        Err(error) => panic!(
            "the project site must build: {}: {}",
            error.label, error.message
        ),
    }
}

#[test]
fn project_site_over_overview_and_reference() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");

    let built = project_site_built(&out);

    let names: Vec<&str> = built.pages.iter().map(|page| page.name.as_str()).collect();
    assert_eq!(
        names,
        ["project-site", "reference"],
        "the unbacked overview is omitted; the package and reference are emitted"
    );

    let reference = built
        .pages
        .iter()
        .find(|page| page.name == "reference")
        .expect("the reference product is emitted");
    assert_eq!(
        reference.question,
        Some("what exactly is this"),
        "the reference page names its reader question"
    );
    assert_eq!(
        reference.documents,
        [TECH_KIND, MAP_KIND],
        "the reference page admits its declared documents in declaration order"
    );
    assert!(
        reference.path.ends_with("reference.md"),
        "the reference page is written as reference.md"
    );

    let omitted = built
        .omitted
        .iter()
        .find(|entry| entry.name == "overview")
        .expect("the omission of overview is reported");
    assert_eq!(omitted.reason, "no-backing-document");
    assert!(
        omitted.tier.contains("product"),
        "the omission reports the product tier: {}",
        omitted.tier
    );

    assert!(
        !out.join("overview.md").exists(),
        "no overview page exists without a backing document"
    );

    let pages = markdown(&read_tree(&out));
    let reference_page = pages
        .get("reference.md")
        .expect("the reference page is written");
    let expected = format!(
        "## Reference\n\nReader question: what exactly is this\n\n\
         ### {TECH_KIND}\n\n{TECH_BODY}\n\n\
         ### {MAP_KIND}\n\n{MAP_BODY}\n\n"
    );
    assert_eq!(
        block_body(reference_page, "publication.product.reference").trim_end_matches('\n'),
        expected.trim_end_matches('\n'),
        "the reference page is the exact product block over its admitted documents"
    );

    let package_page = pages
        .get("project-site.md")
        .expect("the package page is written");
    assert_package_member_rows(
        package_page,
        &[
            "- overview \u{2014} unavailable",
            "- [reference](reference.md)",
        ],
    );
    assert!(
        !package_page.contains(MAP_BODY) && !package_page.contains(TECH_BODY),
        "the package embeds neither product's content: {package_page}"
    );
}
