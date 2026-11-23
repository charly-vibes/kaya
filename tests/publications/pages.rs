//! Rendering order and the page-block vocabulary: partial includes render
//! only present kinds, the understanding document backs its products once,
//! and a page carrying a block outside the publication vocabulary is
//! refused.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use tempfile::TempDir;

use kaya::publication::{Documents, build, read_documents};

use crate::common::*;

#[test]
fn partial_includes_render_only_present_kinds() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    let decls = [product("reference", &[TECH_KIND, MAP_KIND])];
    let docs = documents(&[(MAP_KIND, MAP_BODY, &["evt-map"])]);

    build(&decls, &docs, &out).expect("a partially backed product still builds");
    let page = fs::read_to_string(out.join("reference.md")).expect("the page is written");
    let body = block_body(&page, "publication.product.reference");

    assert!(
        body.contains(&format!("### {MAP_KIND}")),
        "the present kind renders: {body}"
    );
    assert!(
        !body.contains(TECH_KIND),
        "the absent kind renders nothing: {body}"
    );
    assert_eq!(
        body.matches("### ").count(),
        1,
        "no empty heading is left for an absent kind: {body}"
    );
}

// 21. kaya.publications.fact_once — a duplicated include renders once
#[test]
fn duplicate_include_renders_the_body_once() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    let decls = [product("reference", &[MAP_KIND, MAP_KIND])];
    let docs = documents(&[(MAP_KIND, MAP_BODY, &["evt-map"])]);

    let built = build(&decls, &docs, &out).expect("the duplicated include still builds");
    assert!(
        built.pages.iter().any(|page| page.name == "reference"),
        "the reference product is emitted"
    );
    let page = fs::read_to_string(out.join("reference.md")).expect("the page is written");
    let body = block_body(&page, "publication.product.reference");
    assert_eq!(
        body.matches(&format!("### {MAP_KIND}")).count(),
        1,
        "the duplicated include gets one heading: {body}"
    );
    assert_eq!(
        body.matches(MAP_BODY).count(),
        1,
        "the duplicated include renders its body once: {body}"
    );
}

// 22. kaya.publications.format_attribution — exact export shape
/// The understanding document with one findings and one decision block,
/// read back as compiled documents.
fn understanding_docs(research: &Path) -> Documents {
    let findings = format!(
        "<!-- kaya:begin generated:{FINDINGS_KIND} -->\n\
         ### Findings\n\n\
         - kaya/Finding/f-hit-rate \u{2014} {FINDINGS_BODY}\n\
         <!-- kaya:end generated:{FINDINGS_KIND} -->\n"
    );
    let decision = format!(
        "<!-- kaya:begin generated:{DECISION_KIND} -->\n\
         ### Decisions\n\n\
         - kaya/Decision/d-ship-cache \u{2014} {DECISION_BODY}\n\
         <!-- kaya:end generated:{DECISION_KIND} -->\n"
    );
    fs::create_dir_all(research).expect("research tree is creatable");
    fs::write(
        research.join("understanding.md"),
        format!("# Understanding\n\n{findings}\n{decision}"),
    )
    .expect("understanding is writable");
    read_documents(research).expect("the compiled documents are readable")
}

/// Each fact body renders once, on the concept page; every later product page
/// cross-references it instead of repeating the body.
fn assert_rendered_once_per_page(pages: &BTreeMap<String, String>) {
    for (kind, anchor, body) in [
        (FINDINGS_KIND, FINDINGS_ANCHOR, FINDINGS_BODY),
        (DECISION_KIND, DECISION_ANCHOR, DECISION_BODY),
    ] {
        let occurrences: usize = pages.values().map(|page| page.matches(body).count()).sum();
        assert_eq!(
            occurrences, 1,
            "{body} is rendered exactly once across the product pages: {pages:?}"
        );
        assert!(
            pages["concept.md"].contains(&format!("### {kind}")),
            "the concept page carries the {kind} section: {}",
            pages["concept.md"]
        );
        assert!(
            pages["faq.md"].contains(&format!("See [concept](concept.md#{anchor}).")),
            "the faq page cross-references the {kind} body: {}",
            pages["faq.md"]
        );
    }
}
#[test]
fn understanding_document_backs_concept_faq_and_troubleshooting_once() {
    let tmp = TempDir::new().expect("temp dir");
    let research = tmp.path().join("research");
    let docs = understanding_docs(&research);

    let kinds: Vec<&str> = docs.0.keys().map(String::as_str).collect();
    assert_eq!(
        kinds,
        vec![DECISION_KIND, FINDINGS_KIND],
        "both understanding blocks are compiled documents"
    );
    assert_eq!(
        docs.0[FINDINGS_KIND].event_ids,
        ["kaya/Finding/f-hit-rate"],
        "the findings document attributes its rendered id"
    );
    assert!(
        docs.0[FINDINGS_KIND].body.contains(FINDINGS_BODY),
        "the findings document keeps its body: {}",
        docs.0[FINDINGS_KIND].body
    );
    assert_eq!(
        docs.0[DECISION_KIND].event_ids,
        ["kaya/Decision/d-ship-cache"],
        "the decision document attributes its rendered id"
    );
    assert!(
        docs.0[DECISION_KIND].body.contains(DECISION_BODY),
        "the decision document keeps its body: {}",
        docs.0[DECISION_KIND].body
    );

    let out = tmp.path().join("out");
    let decls = [
        product("concept", &[DECISION_KIND, FINDINGS_KIND]),
        product("faq", &[FINDINGS_KIND, DECISION_KIND]),
        product("troubleshooting", &[FINDINGS_KIND]),
    ];
    let built = build(&decls, &docs, &out).expect("the understanding products must build");
    assert!(
        built.omitted.is_empty(),
        "every declared product has an admissible backing document: {:?}",
        built.omitted
    );

    let pages = markdown(&read_tree(&out));
    for name in ["concept.md", "faq.md", "troubleshooting.md"] {
        assert!(
            pages.contains_key(name),
            "a backed product writes its page, missing {name}: {:?}",
            pages.keys().collect::<Vec<_>>()
        );
    }

    // Each document body is rendered once, in the first product page that
    // admits it; every later page cross-references it.
    assert_rendered_once_per_page(&pages);
    let troubleshooting = &pages["troubleshooting.md"];
    assert!(
        troubleshooting.contains(&format!("See [concept](concept.md#{FINDINGS_ANCHOR}).")),
        "the troubleshooting page cross-references the findings body: {troubleshooting}"
    );
    assert!(
        !troubleshooting.contains(DECISION_BODY),
        "troubleshooting admits no decision record: {troubleshooting}"
    );
}

#[test]
fn publication_page_with_a_misspelled_block_is_refused() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    fs::create_dir(&out).expect("output directory");

    let stale = "publication.produkt.reference";
    let page = format!(
        "<!-- kaya:begin generated:{stale} -->\n\
         stale body from a misspelled tier\n\
         <!-- kaya:end generated:{stale} -->\n"
    );
    let page_path = out.join("reference.md");
    fs::write(&page_path, &page).expect("the page is writable");

    let decls = [product("reference", &[MAP_KIND])];
    let docs = documents(&[(MAP_KIND, MAP_BODY, &["evt-map"])]);

    let error = build(&decls, &docs, &out)
        .expect_err("a page outside the publication vocabulary is refused");
    assert_eq!(
        error.label, "kaya.publications.block_vocabulary",
        "a block outside the publication vocabulary is a labeled refusal: {}",
        error.message
    );
    assert!(
        error.message.contains(stale),
        "the refusal names the offending block: {}",
        error.message
    );
    assert!(
        error.message.contains("reference.md"),
        "the refusal names the page it refused: {}",
        error.message
    );
    assert_eq!(
        fs::read_to_string(&page_path).expect("the page is readable"),
        page,
        "a refused build leaves the page byte-identical"
    );
}

#[test]
fn publication_page_with_a_document_block_is_refused() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    fs::create_dir(&out).expect("output directory");

    let foreign = MAP_KIND;
    let page = format!(
        "<!-- kaya:begin generated:{foreign} -->\n\
         a research document block inside a publication page\n\
         <!-- kaya:end generated:{foreign} -->\n"
    );
    let page_path = out.join("reference.md");
    fs::write(&page_path, &page).expect("the page is writable");

    let decls = [product("reference", &[MAP_KIND])];
    let docs = documents(&[(MAP_KIND, MAP_BODY, &["evt-map"])]);

    let error = build(&decls, &docs, &out)
        .expect_err("a research document block inside a publication page is refused");
    assert_eq!(
        error.label, "kaya.publications.block_vocabulary",
        "a document-vocabulary block in a publication page is refused: {}",
        error.message
    );
    assert!(
        error.message.contains(foreign),
        "the refusal names the offending block: {}",
        error.message
    );
    assert_eq!(
        fs::read_to_string(&page_path).expect("the page is readable"),
        page,
        "a refused build leaves the page byte-identical"
    );
}

#[test]
fn publication_page_with_a_name_outside_its_tier_is_refused() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    fs::create_dir(&out).expect("output directory");

    // "glossary" is a collection name, so it does not belong under the product
    // tier: the vocabulary is tier-scoped, not a flat name list.
    let wrong_tier = "publication.product.glossary";
    let page = format!(
        "<!-- kaya:begin generated:{wrong_tier} -->\n\
         a collection name under the product tier\n\
         <!-- kaya:end generated:{wrong_tier} -->\n"
    );
    let page_path = out.join("reference.md");
    fs::write(&page_path, &page).expect("the page is writable");

    let decls = [product("reference", &[MAP_KIND])];
    let docs = documents(&[(MAP_KIND, MAP_BODY, &["evt-map"])]);

    let error =
        build(&decls, &docs, &out).expect_err("a name outside the tier's vocabulary is refused");
    assert_eq!(
        error.label, "kaya.publications.block_vocabulary",
        "a name outside the tier vocabulary is refused: {}",
        error.message
    );
    assert!(
        error.message.contains(wrong_tier),
        "the refusal names the offending block: {}",
        error.message
    );
}
