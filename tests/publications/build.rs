//! The write phase and the plan-time refusals: one managed block per page,
//! stale pages removed, and every invalid declaration refused before any
//! write.

use std::collections::BTreeMap;
use std::fs;

use tempfile::TempDir;

use kaya::publication::{Decl, DocumentRef, Documents, Tier, build};

use crate::common::*;

#[test]
fn build_writes_one_managed_block_and_preserves_surrounding_bytes() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    fs::create_dir(&out).expect("output directory");

    let block = "publication.product.reference";
    let reference_path = out.join("reference.md");
    let authored = format!(
        "# Authored intro\n\nKeep this byte for byte.\n\n\
         <!-- kaya:begin generated:{block} -->\n\
         stale body\n\
         <!-- kaya:end generated:{block} -->\n\n\
         Authored footer stays.\n"
    );
    fs::write(&reference_path, &authored).expect("the authored page is writable");

    let decls = [product("reference", &[MAP_KIND])];
    let docs = documents(&[(MAP_KIND, MAP_BODY, &["evt-map"])]);
    let built = build(&decls, &docs, &out).expect("the page must build over authored bytes");
    let page = fs::read_to_string(&reference_path).expect("the page is readable");

    assert!(
        page.starts_with("# Authored intro\n\nKeep this byte for byte.\n\n"),
        "authored bytes before the block survive: {page}"
    );
    assert!(
        page.ends_with("\n\nAuthored footer stays.\n"),
        "authored bytes after the block survive: {page}"
    );
    assert_eq!(
        page.matches(&format!("kaya:begin generated:{block}"))
            .count(),
        1,
        "the page carries exactly one begin marker: {page}"
    );
    assert_eq!(
        page.matches(&format!("kaya:end generated:{block}")).count(),
        1,
        "the page carries exactly one end marker: {page}"
    );
    let expected = format!(
        "## Reference\n\nReader question: what exactly is this\n\n\
         ### {MAP_KIND}\n\n{MAP_BODY}\n\n"
    );
    assert_eq!(
        block_body(&page, block).trim_end_matches('\n'),
        expected.trim_end_matches('\n'),
        "build writes the rendered page into the managed block, not around it"
    );
    assert!(
        !page.contains("stale body"),
        "the old block body is replaced: {page}"
    );
    assert!(
        built.summary.recomputed.iter().any(|name| name == block),
        "the page block is reported recomputed: {:?}",
        built.summary.recomputed
    );
    assert!(
        built.summary.unchanged.is_empty(),
        "nothing was already current: {:?}",
        built.summary.unchanged
    );
}

// 13. kaya.publications.stub_free — a disappeared backing document removes the page
#[test]
fn stale_page_is_removed_when_its_backing_document_disappears() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    let decls = [
        product("overview", &[OVERVIEW_KIND]),
        package("project-site", &["overview"]),
    ];

    let backed = documents(&[(OVERVIEW_KIND, OVERVIEW_BODY, &["evt-overview"])]);
    let first = build(&decls, &backed, &out).expect("the backed product must build");
    assert!(
        first.pages.iter().any(|page| page.name == "overview"),
        "the backed product is emitted: {:?}",
        first.pages
    );
    assert!(out.join("overview.md").exists(), "the backed page exists");

    let second = build(&decls, &Documents::default(), &out).expect("the omission must rebuild");
    assert!(
        !out.join("overview.md").exists(),
        "a page whose backing document disappeared is removed, not left stale"
    );
    assert!(
        !second.pages.iter().any(|page| page.name == "overview"),
        "an unbacked product leaves no page entry: {:?}",
        second.pages
    );
    let omitted = second
        .omitted
        .iter()
        .find(|entry| entry.name == "overview")
        .expect("the omission is reported");
    assert_eq!(omitted.reason, "no-backing-document");
    assert!(omitted.tier.contains("product"));
    let package_page =
        fs::read_to_string(out.join("project-site.md")).expect("the package page is written");
    assert!(
        block_body(&package_page, "publication.package.project-site")
            .contains("- overview — unavailable"),
        "the package row flips to unavailable: {package_page}"
    );
}

// 14. kaya.publications.tier_resolved — declared tier must be the resolved tier
#[test]
fn declared_tier_that_differs_from_the_resolved_tier_is_refused() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    fs::create_dir(&out).expect("output directory");
    let decls = [Decl {
        tier: Tier::Package,
        name: "overview".to_owned(),
        includes: Vec::new(),
        members: Vec::new(),
    }];

    let error = build(&decls, &Documents::default(), &out)
        .expect_err("a product name declared as a package must be refused");
    assert_eq!(
        error.label, "kaya.publications.tier_resolved",
        "a declared/resolved tier mismatch is a tier refusal: {}",
        error.message
    );
    assert!(
        error.message.contains("overview"),
        "the refusal names the offending publication: {}",
        error.message
    );
    assert_no_writes(&out);
}

// 15. kaya.publications.duplicate_declaration — one declaration per name
#[test]
fn duplicate_declarations_are_refused_before_any_write() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    fs::create_dir(&out).expect("output directory");
    let decls = [
        product("overview", &[OVERVIEW_KIND]),
        product("overview", &[OVERVIEW_KIND]),
    ];

    let error = build(&decls, &Documents::default(), &out)
        .expect_err("the same publication declared twice must be refused");
    assert_eq!(
        error.label, "kaya.publications.duplicate_declaration",
        "a repeated declaration is a labeled refusal: {}",
        error.message
    );
    assert!(
        error.message.contains("overview"),
        "the refusal names the duplicate publication: {}",
        error.message
    );
    assert_no_writes(&out);
}

// 16. kaya.publications.tier_fields — each tier owns exactly one field
#[test]
fn product_with_members_and_package_with_includes_are_refused() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    fs::create_dir(&out).expect("output directory");

    let mut product_with_members = product("overview", &[OVERVIEW_KIND]);
    product_with_members.members = strings(&["reference"]);
    let error = build(&[product_with_members], &Documents::default(), &out)
        .expect_err("a product does not list package members");
    assert_eq!(
        error.label, "kaya.publications.tier_fields",
        "a stray field on a product is a labeled refusal: {}",
        error.message
    );
    for needle in ["overview", "product", "member"] {
        assert!(
            error.message.contains(needle),
            "the refusal names the publication, tier and stray field ({needle}): {}",
            error.message
        );
    }
    assert_no_writes(&out);

    let mut package_with_includes = package("project-site", &["overview"]);
    package_with_includes.includes = strings(&[OVERVIEW_KIND]);
    let docs = documents(&[(OVERVIEW_KIND, OVERVIEW_BODY, &["evt-1"])]);
    let decls = [product("overview", &[OVERVIEW_KIND]), package_with_includes];
    let error = build(&decls, &docs, &out).expect_err("a package does not compose document kinds");
    assert_eq!(
        error.label, "kaya.publications.tier_fields",
        "a stray field on a package is a labeled refusal: {}",
        error.message
    );
    for needle in ["project-site", "package", "include"] {
        assert!(
            error.message.contains(needle),
            "the refusal names the publication, tier and stray field ({needle}): {}",
            error.message
        );
    }
    assert_no_writes(&out);
}

// 17. kaya.publications.tier_unimplemented — collections are not silently dropped
#[test]
fn collection_declaration_is_refused_as_unimplemented() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    fs::create_dir(&out).expect("output directory");
    let decls = [Decl {
        tier: Tier::Collection,
        name: "glossary".to_owned(),
        includes: Vec::new(),
        members: Vec::new(),
    }];

    let error = build(&decls, &Documents::default(), &out)
        .expect_err("a collection declaration must not be silently dropped");
    assert_eq!(
        error.label, "kaya.publications.tier_unimplemented",
        "an unimplemented tier is a labeled refusal: {}",
        error.message
    );
    assert!(
        error.message.contains("glossary"),
        "the refusal names the collection: {}",
        error.message
    );
    assert!(
        error.message.contains("implemented"),
        "the refusal states collections are not implemented yet: {}",
        error.message
    );
    assert_no_writes(&out);
}

// 18. kaya.publications.package_members — members are declared products
#[test]
fn package_member_must_be_a_declared_product() {
    let tmp = TempDir::new().expect("temp dir");

    for member in ["glossary", "project-site", "experiment-report"] {
        let out = tmp.path().join(format!("out-{member}"));
        fs::create_dir(&out).expect("output directory");
        let decls = [package("project-site", &[member])];
        let error = build(&decls, &Documents::default(), &out)
            .expect_err("only products may be package members");
        assert_eq!(
            error.label, "kaya.publications.package_members",
            "a non-product member is a labeled refusal: {}",
            error.message
        );
        assert!(
            error.message.contains(member),
            "the refusal names the offending member: {}",
            error.message
        );
        assert!(
            error.message.contains("project-site"),
            "the refusal names the package: {}",
            error.message
        );
        assert_no_writes(&out);
    }

    // A name in the product vocabulary is still refused when the build never
    // declared that product.
    let out = tmp.path().join("out-undeclared");
    fs::create_dir(&out).expect("output directory");
    let decls = [
        product("overview", &[OVERVIEW_KIND]),
        package("project-site", &["reference"]),
    ];
    let error = build(&decls, &Documents::default(), &out)
        .expect_err("a package cannot reference an undeclared product");
    assert_eq!(
        error.label, "kaya.publications.package_members",
        "an undeclared product member is a labeled refusal: {}",
        error.message
    );
    assert!(
        error.message.contains("reference"),
        "the refusal names the undeclared member: {}",
        error.message
    );
    assert!(
        error.message.contains("project-site"),
        "the refusal names the package: {}",
        error.message
    );
    assert_no_writes(&out);
}

// 19. kaya.publications.document_kind — map key and DocumentRef agree
#[test]
fn document_kind_mismatch_is_refused_before_any_write() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    fs::create_dir(&out).expect("output directory");

    let mut map = BTreeMap::new();
    map.insert(
        OVERVIEW_KIND.to_owned(),
        DocumentRef {
            kind: MAP_KIND.to_owned(),
            body: OVERVIEW_BODY.to_owned(),
            event_ids: Vec::new(),
        },
    );
    let docs = Documents(map);
    let decls = [product("overview", &[OVERVIEW_KIND])];

    let error = build(&decls, &docs, &out)
        .expect_err("a document whose kind differs from its map key must be refused");
    assert_eq!(
        error.label, "kaya.publications.document_kind",
        "a kind/key mismatch is a labeled refusal: {}",
        error.message
    );
    assert!(
        error.message.contains(OVERVIEW_KIND),
        "the refusal names the map key: {}",
        error.message
    );
    assert!(
        error.message.contains(MAP_KIND),
        "the refusal names the differing DocumentRef kind: {}",
        error.message
    );
    assert_no_writes(&out);
}

// 20. kaya.publications.stub_free — only present kinds get a heading
