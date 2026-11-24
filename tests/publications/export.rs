//! The structured export: exact row keys, rebuild-stable bytes and mtimes,
//! and page/export attribution agreement.

use std::fs;
use std::path::Path;
use std::time::SystemTime;

use serde_json::{Value, json};
use tempfile::TempDir;

use kaya::publication::{Decl, Documents, build};

use crate::common::*;

/// The project site over one omitted overview and one emitted reference,
/// with its export parsed from disk.
fn export_site(tmp: &TempDir) -> Value {
    let out = tmp.path().join("out");
    let decls = [
        product("overview", &[OVERVIEW_KIND]),
        product("reference", &[MAP_KIND]),
        package("project-site", &["overview", "reference"]),
    ];
    let docs = documents(&[(MAP_KIND, MAP_BODY, &["evt-map"])]);
    build(&decls, &docs, &out).expect("the project site must build");
    serde_json::from_str(
        &fs::read_to_string(out.join("publication.json")).expect("the export is written"),
    )
    .expect("the export is JSON")
}

/// The exported product rows carry exactly their keys and emission facts.
fn assert_product_rows(export: &Value) {
    let products = export["products"]
        .as_array()
        .expect("the export lists products");
    assert_eq!(
        products.len(),
        2,
        "only declared products are exported: {export}"
    );
    assert_eq!(products[0]["name"], json!("overview"));
    assert_eq!(products[1]["name"], json!("reference"));
    for row in products {
        assert_eq!(
            object_keys(row),
            ["documents", "emitted", "name", "question"],
            "a product row carries exactly its name, question, emitted flag and documents: {row}"
        );
    }

    assert_eq!(
        products[0]["emitted"],
        json!(false),
        "an omitted product is exported with emitted=false"
    );
    assert_eq!(
        products[0]["documents"],
        json!([]),
        "an omitted product keeps an empty documents list"
    );
    assert_eq!(
        products[0]["question"],
        json!("what is this and why does it matter")
    );
    assert_eq!(
        products[1]["emitted"],
        json!(true),
        "an emitted product is exported with emitted=true"
    );
    let documents = products[1]["documents"]
        .as_array()
        .expect("the emitted product lists its documents");
    assert_eq!(documents.len(), 1);
    assert_eq!(
        object_keys(&documents[0]),
        ["event_ids", "kind"],
        "a document row carries exactly its kind and event ids: {}",
        documents[0]
    );
    assert_eq!(documents[0]["kind"], json!(MAP_KIND));
    assert_eq!(documents[0]["event_ids"], json!(["evt-map"]));
}

/// The exported package rows carry exactly their keys and member facts.
fn assert_package_rows_export(export: &Value) {
    let packages = export["packages"]
        .as_array()
        .expect("the export lists packages");
    assert_eq!(
        packages.len(),
        1,
        "only declared packages are exported: {export}"
    );
    assert_eq!(object_keys(&packages[0]), ["members", "name"]);
    let members = packages[0]["members"]
        .as_array()
        .expect("the package lists its members");
    assert_eq!(members.len(), 2);
    for member in members {
        assert_eq!(
            object_keys(member),
            ["emitted", "name"],
            "a package member row carries exactly its name and emitted flag: {member}"
        );
    }
    assert_eq!(members[0]["name"], json!("overview"));
    assert_eq!(members[0]["emitted"], json!(false));
    assert_eq!(members[1]["name"], json!("reference"));
    assert_eq!(members[1]["emitted"], json!(true));
}

#[test]
fn export_reports_emitted_products_with_exact_keys() {
    let tmp = TempDir::new().expect("temp dir");
    let export = export_site(&tmp);
    assert_eq!(
        export["products"]
            .as_array()
            .expect("products listed")
            .len(),
        2,
        "only declared products are exported: {export}"
    );
    assert_product_rows(&export);
    assert_package_rows_export(&export);
}

/// The project-site declaration over the single map fact.
fn single_fact_site() -> ([Decl; 2], Documents) {
    let decls = [
        product("reference", &[MAP_KIND]),
        package("project-site", &["reference"]),
    ];
    let docs = documents(&[(MAP_KIND, MAP_BODY, &["evt-map"])]);
    (decls, docs)
}

/// The mtimes of the reference page, the package page, and the export.
fn tree_mtimes(out: &Path) -> [SystemTime; 3] {
    let paths = [
        out.join("reference.md"),
        out.join("project-site.md"),
        out.join("publication.json"),
    ];
    paths.map(|path| {
        fs::metadata(&path)
            .unwrap_or_else(|error| panic!("{} is readable: {error}", path.display()))
            .modified()
            .expect("every written file carries an mtime")
    })
}

#[test]
fn rebuild_reports_unchanged_blocks_and_leaves_bytes_and_mtime_alone() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    let (decls, docs) = single_fact_site();

    let first = build(&decls, &docs, &out).expect("the project site must build");
    let blocks = [
        "publication.product.reference",
        "publication.package.project-site",
        "publication.json",
    ];
    for name in blocks {
        assert!(
            first
                .summary
                .recomputed
                .iter()
                .any(|candidate| candidate == name),
            "a fresh build recomputes {name}: {:?}",
            first.summary.recomputed
        );
    }
    assert!(
        first.summary.unchanged.is_empty(),
        "a fresh build has nothing already current: {:?}",
        first.summary.unchanged
    );

    let package_path = out.join("project-site.md");
    let expected_package = wrapped_page(
        "publication.package.project-site",
        "## Project-site\n\n- [reference](reference.md)\n",
    );
    assert_eq!(
        fs::read_to_string(&package_path).expect("the package page is written"),
        expected_package,
        "the package page is exactly one managed block with the navigation body"
    );
    let mtimes = tree_mtimes(&out);

    let second = build(&decls, &docs, &out).expect("the rebuild must build");
    assert!(
        second.summary.recomputed.is_empty(),
        "an unchanged rebuild recomputes nothing: {:?}",
        second.summary.recomputed
    );
    for name in blocks {
        assert!(
            second
                .summary
                .unchanged
                .iter()
                .any(|candidate| candidate == name),
            "an unchanged rebuild reports {name} unchanged: {:?}",
            second.summary.unchanged
        );
    }
    assert_eq!(
        fs::read_to_string(&package_path).expect("the package page is readable"),
        expected_package,
        "an unchanged rebuild leaves the page bytes alone"
    );
    assert_eq!(
        tree_mtimes(&out),
        mtimes,
        "unchanged files are not rewritten, so their mtimes stay put"
    );

    let tree = read_tree(&out);
    let refused = [product("faq", &[SPEC_KIND])];
    let error =
        build(&refused, &docs, &out).expect_err("faq cannot include an experiment specification");
    assert_eq!(error.label, "kaya.publications.inadmissible_refused");
    assert_eq!(
        read_tree(&out),
        tree,
        "a refusal leaves an existing tree byte-identical"
    );
}

/// The reference product built twice: first with both facts, then with the
/// research map reattributed to a new event. Returns the rebuilt page and the
/// parsed export.
fn attribution_site(tmp: &TempDir) -> (String, Value) {
    let out = tmp.path().join("out");
    let decls = [product("reference", &[TECH_KIND, MAP_KIND])];

    let first = documents(&[
        (
            TECH_KIND,
            "Technical reference cites kaya/Evidence/t-one see above",
            &["kaya/Evidence/t-one"],
        ),
        (
            MAP_KIND,
            "Research map cites kaya/Evidence/m-one see above",
            &["kaya/Evidence/m-one"],
        ),
    ]);
    build(&decls, &first, &out).expect("the first build succeeds");

    let second = documents(&[
        (
            TECH_KIND,
            "Technical reference cites kaya/Evidence/t-one see above",
            &["kaya/Evidence/t-one"],
        ),
        (
            MAP_KIND,
            "Research map now cites kaya/Evidence/m-two see above",
            &["kaya/Evidence/m-two"],
        ),
    ]);
    build(&decls, &second, &out).expect("the changed document rebuilds");

    let page = fs::read_to_string(out.join("reference.md")).expect("the page is written");
    let export: Value = serde_json::from_str(
        &fs::read_to_string(out.join("publication.json")).expect("the export is written"),
    )
    .expect("the export is JSON");
    (page, export)
}

/// The exported document rows as `(kind, event_ids)` pairs.
fn exported_document_rows(export: &Value) -> Vec<(String, Vec<String>)> {
    export["products"]
        .as_array()
        .expect("the export lists products")
        .iter()
        .find(|product| product["name"] == json!("reference"))
        .expect("the export names the reference product")["documents"]
        .as_array()
        .expect("the reference product lists its documents")
        .iter()
        .map(|document| {
            let kind = document["kind"]
                .as_str()
                .expect("every document kind is a string")
                .to_owned();
            let ids = document["event_ids"]
                .as_array()
                .expect("every document lists its event ids")
                .iter()
                .map(|id| {
                    id.as_str()
                        .expect("every exported event id is a string")
                        .to_owned()
                })
                .collect();
            (kind, ids)
        })
        .collect()
}

#[test]
fn export_and_page_attribution_agree_after_one_document_changes() {
    let tmp = TempDir::new().expect("temp dir");
    let (page, export) = attribution_site(&tmp);
    let sections = rendered_sections(&block_body(&page, "publication.product.reference"));
    assert_eq!(
        sections.len(),
        2,
        "both declared documents render after the rebuild: {sections:?}"
    );

    for (kind, exported) in exported_document_rows(&export) {
        let section = sections
            .get(&kind)
            .unwrap_or_else(|| panic!("the page renders {kind}: {sections:?}"));
        assert_eq!(
            body_node_ids(section),
            exported,
            "the page and export name the same event ids for {kind}"
        );
    }
    let map = exported_document_rows(&export)
        .into_iter()
        .find(|(kind, _)| kind == MAP_KIND)
        .expect("the export names the changed research map");
    assert_eq!(
        map.1,
        vec!["kaya/Evidence/m-two".to_owned()],
        "the rebuilt attribution carries the changed document's new ids"
    );
}

#[test]
fn export_lists_collection_rows() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    let decls = [
        collection("glossary"),
        collection("example-gallery"),
        collection("api-catalogue"),
    ];
    let docs = documents(&[(OVERVIEW_KIND, OVERVIEW_BODY, &["evt-1"])]);
    build(&decls, &docs, &out).expect("the collections must build");

    let export: Value = serde_json::from_str(
        &fs::read_to_string(out.join("publication.json")).expect("the export is written"),
    )
    .expect("the export is JSON");
    let collections = export["collections"]
        .as_array()
        .expect("the export lists collections");
    assert_eq!(
        collections.len(),
        3,
        "only declared collections are exported: {export}"
    );
    assert_eq!(
        object_keys(&collections[0]),
        ["emitted", "name", "reason"],
        "an omitted collection row carries the omission reason: {}",
        collections[0]
    );
    assert_eq!(collections[0]["name"], json!("api-catalogue"));
    assert_eq!(
        collections[0]["emitted"],
        json!(false),
        "an omitted collection is exported with emitted=false"
    );
    assert_eq!(
        collections[0]["reason"],
        json!("no-backing-document"),
        "the omission reason mirrors the summary"
    );
    assert_eq!(
        collections[1]["name"],
        json!("example-gallery"),
        "rows sort by name: {collections:?}"
    );
    assert_eq!(
        object_keys(&collections[2]),
        ["emitted", "name"],
        "an emitted collection row carries exactly its name and emitted flag: {}",
        collections[2]
    );
    assert_eq!(collections[2]["name"], json!("glossary"));
    assert_eq!(
        collections[2]["emitted"],
        json!(true),
        "an emitted collection is exported with emitted=true"
    );
}
