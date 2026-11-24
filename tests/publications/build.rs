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

// 17. kaya.publications.tier_unimplemented — the glossary collection renders a page
#[test]
fn glossary_declaration_renders_a_collection_page() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    fs::create_dir(&out).expect("output directory");
    let decls = [Decl {
        tier: Tier::Collection,
        name: "glossary".to_owned(),
        includes: Vec::new(),
        members: Vec::new(),
    }];
    let docs = documents(&[(OVERVIEW_KIND, OVERVIEW_BODY, &["evt-1"])]);

    let built = build(&decls, &docs, &out).expect("a collection declaration must build");
    assert!(
        built.pages.iter().any(|page| page.name == "glossary"),
        "the collection is emitted as a page: {:?}",
        built.pages
    );
    let page = fs::read_to_string(out.join("glossary.md")).expect("the glossary page is written");
    let body = block_body(&page, "publication.collection.glossary");
    assert!(
        !body.is_empty(),
        "the glossary collection block renders a non-empty body: {page}"
    );
}

// 17. kaya.publications.tier_unimplemented — refused kinds name what exists
#[test]
fn refused_collection_names_the_declared_kind_and_the_implemented_kinds() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    fs::create_dir(&out).expect("output directory");
    let decls = [collection("feature-index")];

    let error = build(&decls, &Documents::default(), &out)
        .expect_err("a collection declaration must not be silently dropped");
    assert_eq!(
        error.label, "kaya.publications.tier_unimplemented",
        "an unimplemented tier is a labeled refusal: {}",
        error.message
    );
    assert!(
        error.message.contains("feature-index"),
        "the refusal names the declared collection: {}",
        error.message
    );
    for implemented in [
        "api-catalogue",
        "example-gallery",
        "glossary",
        "knowledge-index",
        "topic-map",
    ] {
        assert!(
            error.message.contains(implemented),
            "the refusal names the implemented collection ({implemented}): {}",
            error.message
        );
    }
    assert_no_writes(&out);
}

// 17. kaya.publications.tier_fields — a collection carries no fields
#[test]
fn collection_declaring_fields_refused() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    fs::create_dir(&out).expect("output directory");

    let mut collection_with_includes = collection("topic-map");
    collection_with_includes.includes = strings(&[MAP_KIND]);
    let error = build(&[collection_with_includes], &Documents::default(), &out)
        .expect_err("a collection does not compose document kinds");
    assert_eq!(
        error.label, "kaya.publications.tier_fields",
        "a stray field on a collection is a labeled refusal: {}",
        error.message
    );
    for needle in ["topic-map", "collection", "include"] {
        assert!(
            error.message.contains(needle),
            "the refusal names the collection, tier and stray field ({needle}): {}",
            error.message
        );
    }

    let mut collection_with_members = collection("glossary");
    collection_with_members.members = strings(&["overview"]);
    let error = build(&[collection_with_members], &Documents::default(), &out)
        .expect_err("a collection does not list package members");
    assert_eq!(
        error.label, "kaya.publications.tier_fields",
        "a stray field on a collection is a labeled refusal: {}",
        error.message
    );
    for needle in ["glossary", "collection", "member"] {
        assert!(
            error.message.contains(needle),
            "the refusal names the collection, tier and stray field ({needle}): {}",
            error.message
        );
    }
    assert_no_writes(&out);
}

// Collection composition contracts — topic-map composes the research map
#[test]
fn topic_map_renders_one_row_per_topic_with_type_and_relationship_types() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    fs::create_dir(&out).expect("output directory");
    let map_body = "### Relationships\n\n\
                    - kaya/Question/why -[cites]-> kaya/Source/docs-src-index.md@43b606f\n\
                    - kaya/Question/why -[investigates]-> kaya/Model/cache\n\n\
                    ### Isolated\n\n\
                    - kaya/Decision/keep — Decision — keep the gate\n";
    let docs = documents(&[(MAP_KIND, map_body, &[])]);
    let decls = [collection("topic-map")];

    let built = build(&decls, &docs, &out).expect("the topic-map must build from the map");
    assert!(
        built.pages.iter().any(|page| page.name == "topic-map"),
        "the topic-map collection is emitted as a page: {:?}",
        built.pages
    );
    let page = fs::read_to_string(out.join("topic-map.md")).expect("the topic-map page is written");
    let body = block_body(&page, "publication.collection.topic-map");
    for row in [
        "- kaya/Question/why — Question — cites, investigates",
        "- kaya/Model/cache — Model — investigates",
        "- kaya/Source/docs-src-index.md@43b606f — Source — cites",
        "- kaya/Decision/keep — Decision",
    ] {
        assert!(
            body.contains(row),
            "one row per distinct topic id with its type and relationship types ({row}): {body}"
        );
    }
}

#[test]
fn topic_map_cross_references_the_page_that_embeds_the_research_map() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    fs::create_dir(&out).expect("output directory");
    let map_body = "### Relationships\n\n\
                    - kaya/Question/why -[investigates]-> kaya/Model/cache\n";
    let docs = documents(&[(MAP_KIND, map_body, &[])]);
    let decls = [
        product("architecture-design", &[MAP_KIND]),
        collection("topic-map"),
    ];

    build(&decls, &docs, &out).expect("the tree must build");
    let page = fs::read_to_string(out.join("topic-map.md")).expect("the topic-map page is written");
    let body = block_body(&page, "publication.collection.topic-map");
    assert!(
        body.contains("See [architecture-design](architecture-design.md#orientationresearch-map)"),
        "the topic-map references the page embedding the research-map body: {body}"
    );
    assert!(
        !body.contains("-[investigates]->"),
        "cross-referenced edges are never embedded: {body}"
    );
}

#[test]
fn topic_map_embeds_the_edge_statements_when_no_page_composes_the_research_map() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    fs::create_dir(&out).expect("output directory");
    let map_body = "### Relationships\n\n\
                    - kaya/Question/why -[investigates]-> kaya/Model/cache\n";
    let docs = documents(&[(MAP_KIND, map_body, &[])]);
    let decls = [collection("topic-map")];

    build(&decls, &docs, &out).expect("the tree must build");
    let page = fs::read_to_string(out.join("topic-map.md")).expect("the topic-map page is written");
    let body = block_body(&page, "publication.collection.topic-map");
    assert!(
        body.contains("- kaya/Question/why -[investigates]-> kaya/Model/cache"),
        "with no owner the topic-map embeds the statements as their only rendering: {body}"
    );
}

// Collection composition contracts — knowledge-index composes the census
#[test]
fn knowledge_index_renders_the_per_type_census_of_the_named_node_ids() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    fs::create_dir(&out).expect("output directory");
    let docs = documents(&[
        (
            OVERVIEW_KIND,
            OVERVIEW_BODY,
            &["kaya/Question/why", "kaya/Model/cache"],
        ),
        (
            MAP_KIND,
            MAP_BODY,
            &["kaya/Model/cache", "kaya/Widget/x", "kaya/Run/r1"],
        ),
    ]);
    let decls = [collection("knowledge-index")];

    let built = build(&decls, &docs, &out).expect("the knowledge-index must build");
    assert!(
        built
            .pages
            .iter()
            .any(|page| page.name == "knowledge-index"),
        "the knowledge-index collection is emitted as a page: {:?}",
        built.pages
    );
    let page =
        fs::read_to_string(out.join("knowledge-index.md")).expect("the index page is written");
    let body = block_body(&page, "publication.collection.knowledge-index");
    assert!(
        body.contains("- kaya/Question/why"),
        "each named id is listed once under its type: {body}"
    );
    assert!(
        !body.contains("kaya/Widget/x"),
        "ids whose type segment is outside the closed set are not census rows: {body}"
    );
    for counted in ["Question", "Model", "Run"] {
        assert!(
            body.contains(counted),
            "the census carries a section per closed entity type ({counted}): {body}"
        );
    }
    assert!(
        body.contains("_(none)_"),
        "types with no rows report none: {body}"
    );
}

// Collection composition contracts — producer-backed, omission-tolerant
#[test]
fn example_gallery_renders_a_row_from_an_experiment_record() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    fs::create_dir(&out).expect("output directory");
    let record_body = "### Runs\n\n\
                       - kaya/Run/r1 — passed 2, failed 0, ignored 0\n\n\
                       ### Artifacts\n\n\
                       - kaya/Artifact/kaya-src-lib-rs — src/lib.rs\n\n\
                       ### Observations\n\n\
                       - kaya/Observation/o1 — cache hit rate doubled\n";
    let docs = documents(&[("exploration.experiment-record", record_body, &[])]);
    let decls = [collection("example-gallery")];

    let built = build(&decls, &docs, &out).expect("the gallery must build from the record");
    assert!(
        built
            .pages
            .iter()
            .any(|page| page.name == "example-gallery"),
        "a recorded experiment emits the gallery: {:?}",
        built.pages
    );
    let page =
        fs::read_to_string(out.join("example-gallery.md")).expect("the gallery page is written");
    let body = block_body(&page, "publication.collection.example-gallery");
    for named in [
        "kaya/Run/r1",
        "kaya/Artifact/kaya-src-lib-rs",
        "kaya/Observation/o1",
    ] {
        assert!(
            body.contains(named),
            "the gallery row carries the references the record names ({named}): {body}"
        );
    }
}

#[test]
fn gallery_sections_match_at_line_starts() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    fs::create_dir(&out).expect("output directory");
    // The prose paragraph mentions the heading texts mid-line; only the real
    // headings open the sections the gallery reads.
    let record_body = "# Experiment\n\n\
                       The ### Runs section lists run ids; ### Artifacts names artifacts.\n\n\
                       ### Observations\n\n\
                       - kaya/Observation/o1 — cache hit rate doubled\n\n\
                       ### Runs\n\n\
                       - kaya/Run/r1 — passed 2, failed 0, ignored 0\n";
    let docs = documents(&[("exploration.experiment-record", record_body, &[])]);
    let decls = [collection("example-gallery")];

    let built = build(&decls, &docs, &out).expect("the gallery must build");
    assert!(
        built
            .pages
            .iter()
            .any(|page| page.name == "example-gallery"),
        "the real sections back the row: {:?}",
        built.pages
    );
    let page =
        fs::read_to_string(out.join("example-gallery.md")).expect("the gallery page is written");
    let body = block_body(&page, "publication.collection.example-gallery");
    assert!(
        body.contains("runs kaya/Run/r1"),
        "a mid-line heading mention must not shadow the real section: {body}"
    );
    assert!(
        body.contains("observations kaya/Observation/o1"),
        "the real observations section still feeds the row: {body}"
    );
}

#[test]
fn gallery_and_catalogue_without_producer_rows_are_omitted() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    fs::create_dir(&out).expect("output directory");
    let docs = documents(&[(OVERVIEW_KIND, OVERVIEW_BODY, &["evt-1"])]);
    let decls = [collection("example-gallery"), collection("api-catalogue")];

    let built = build(&decls, &docs, &out).expect("the omissions must build");
    for name in ["example-gallery", "api-catalogue"] {
        assert!(
            !built.pages.iter().any(|page| page.name == name),
            "an unbacked collection writes no page: {name} in {:?}",
            built.pages
        );
        let omitted = built
            .omitted
            .iter()
            .find(|entry| entry.name == name)
            .expect("the collection omission is reported");
        assert_eq!(omitted.reason, "no-backing-document");
        assert_eq!(omitted.tier, "collection");
        assert!(
            !out.join(format!("{name}.md")).exists(),
            "no page of empty sections is written: {name}"
        );
    }
}

#[test]
fn api_catalogue_renders_one_row_per_spec_id_from_the_technical_reference() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    fs::create_dir(&out).expect("output directory");
    let tech_body = "### Specs\n\n\
                     - kaya/Model/cache — SPEC-1\n\
                     - kaya/Source/docs-src-index.md@43b606f — SPEC-2\n";
    let docs = documents(&[(TECH_KIND, tech_body, &[])]);
    let decls = [collection("api-catalogue")];

    let built = build(&decls, &docs, &out).expect("the catalogue must build from the reference");
    assert!(
        built.pages.iter().any(|page| page.name == "api-catalogue"),
        "spec rows emit the catalogue: {:?}",
        built.pages
    );
    let page =
        fs::read_to_string(out.join("api-catalogue.md")).expect("the catalogue page is written");
    let body = block_body(&page, "publication.collection.api-catalogue");
    for row in [
        "- SPEC-1 — kaya/Model/cache",
        "- SPEC-2 — kaya/Source/docs-src-index.md@43b606f",
    ] {
        assert!(
            body.contains(row),
            "one row per spec id with its resolvable reference ({row}): {body}"
        );
    }
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
