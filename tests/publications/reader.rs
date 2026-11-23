//! `read_documents`: the vocabulary filter, first-wins across files,
//! duplicate refusal, malformed markup, and whitespace-only blocks.

use std::fs;
use std::path::Path;

use tempfile::TempDir;

use kaya::publication::{Documents, build, read_documents};

use crate::common::*;

/// Seed the research tree with an overview document, an index carrying the
/// first research-map block plus an out-of-vocabulary block, and a handoff
/// carrying a duplicate research-map block.
fn seed_vocabulary_files(research: &Path) {
    let overview = format!(
        "<!-- kaya:begin generated:{OVERVIEW_KIND} -->\n\
         {OVERVIEW_BODY} cites kaya/Overview/o-one\n\
         <!-- kaya:end generated:{OVERVIEW_KIND} -->\n"
    );
    fs::write(research.join("overview.md"), overview).expect("overview is writable");

    let index = format!(
        "# Research index\n\n\
         <!-- kaya:begin generated:{MAP_KIND} -->\n\
         index map cites kaya/Orientation/m-two, kaya/Orientation/m-one and kaya/Orientation/m-one\n\
         <!-- kaya:end generated:{MAP_KIND} -->\n\n\
         <!-- kaya:begin generated:orientation.other-block -->\n\
         outside the vocabulary, cites kaya/Orientation/ignored\n\
         <!-- kaya:end generated:orientation.other-block -->\n"
    );
    fs::write(research.join("index.md"), index).expect("index is writable");
    let handoff = format!(
        "<!-- kaya:begin generated:{MAP_KIND} -->\n\
         handoff map cites kaya/Orientation/h-one\n\
         <!-- kaya:end generated:{MAP_KIND} -->\n"
    );
    fs::write(research.join("handoff.md"), handoff).expect("handoff is writable");
}

/// The overview product and the project-site package, built over `docs`.
fn build_overview_site(tmp: &TempDir, docs: &Documents) -> kaya::publication::Built {
    let out = tmp.path().join("out");
    let decls = [
        product("overview", &[OVERVIEW_KIND]),
        package("project-site", &["overview"]),
    ];
    let built = build(&decls, docs, &out).expect("the overview document backs its product");
    assert!(
        out.join("overview.md").exists(),
        "the backed overview page is written"
    );
    built
}

#[test]
fn read_documents_keeps_vocabulary_kinds_first_wins_across_files() {
    let tmp = TempDir::new().expect("temp dir");
    let research = tmp.path().join("research");
    fs::create_dir_all(research.join("experiments")).expect("research tree is creatable");
    seed_vocabulary_files(&research);

    let docs = read_documents(&research).expect("the compiled documents are readable");

    assert_eq!(
        docs.0.len(),
        2,
        "only document-vocabulary kinds are kept: {:?}",
        docs.0.keys().collect::<Vec<_>>()
    );
    let map = &docs.0[MAP_KIND];
    assert_eq!(map.kind, MAP_KIND);
    assert!(
        map.body.contains("index map cites"),
        "the first compiled kind in read order wins: {}",
        map.body
    );
    assert!(
        !map.body.contains("handoff map cites"),
        "a duplicate kind in a later file never overrides the first: {}",
        map.body
    );
    assert_eq!(
        map.event_ids,
        ["kaya/Orientation/m-one", "kaya/Orientation/m-two"],
        "event ids are sorted and deduplicated from the winning body"
    );
    let overview = &docs.0[OVERVIEW_KIND];
    assert_eq!(overview.kind, OVERVIEW_KIND);
    assert!(
        overview.body.contains(OVERVIEW_BODY),
        "the overview document is read from overview.md: {}",
        overview.body
    );
    assert_eq!(
        overview.event_ids,
        ["kaya/Overview/o-one"],
        "event ids are sorted and deduplicated from the overview body"
    );
    assert!(
        !docs.0.contains_key("orientation.other-block"),
        "a generated block outside the document vocabulary is not a document"
    );
    assert!(
        !research.join("reproduction-package.md").exists(),
        "missing compiled files are tolerated, not fabricated"
    );

    // The document read from overview.md is the overview product's backing.
    let built = build_overview_site(&tmp, &docs);
    assert!(
        built.pages.iter().any(|page| page.name == "overview"),
        "the overview product is emitted: {:?}",
        built.pages
    );
    let out = tmp.path().join("out");
    let page = fs::read_to_string(out.join("overview.md")).expect("the overview page is readable");
    assert!(
        page.contains(OVERVIEW_BODY),
        "the page renders the compiled overview body: {page}"
    );
    assert!(
        !built.omitted.iter().any(|entry| entry.name == "overview"),
        "the overview document leaves the product unomitted: {:?}",
        built.omitted
    );
}

// 9. kaya.publications.duplicate_block — one file, one block per name
#[test]
fn read_documents_refuses_duplicate_generated_blocks_in_one_file() {
    let tmp = TempDir::new().expect("temp dir");
    let research = tmp.path().join("research");
    fs::create_dir_all(&research).expect("research tree is creatable");
    let index = format!(
        "<!-- kaya:begin generated:{MAP_KIND} -->\n\
         first map\n\
         <!-- kaya:end generated:{MAP_KIND} -->\n\n\
         <!-- kaya:begin generated:{MAP_KIND} -->\n\
         second map\n\
         <!-- kaya:end generated:{MAP_KIND} -->\n"
    );
    fs::write(research.join("index.md"), index).expect("index is writable");

    let error =
        read_documents(&research).expect_err("a repeated block in one file must be refused");
    assert_eq!(
        error.label, "kaya.publications.duplicate_block",
        "a repeated generated block is a labeled refusal: {}",
        error.message
    );
    assert!(
        error.message.contains("index.md"),
        "the refusal names the file: {}",
        error.message
    );
    assert!(
        error.message.contains(MAP_KIND),
        "the refusal names the repeated block: {}",
        error.message
    );
}

// 10. kaya.publications.io — malformed markup is refused with the file name
#[test]
fn read_documents_refuses_malformed_markup_naming_the_file() {
    let tmp = TempDir::new().expect("temp dir");
    let research = tmp.path().join("research");
    fs::create_dir_all(&research).expect("research tree is creatable");
    fs::write(
        research.join("index.md"),
        "<!-- kaya:begin generated:orientation.research-map -->\nbody without a closing marker\n",
    )
    .expect("index is writable");

    let error =
        read_documents(&research).expect_err("broken managed-block grammar must be refused");
    assert_eq!(
        error.label, "kaya.publications.io",
        "a malformed compiled document is an IO refusal: {}",
        error.message
    );
    assert!(
        error.message.contains("index.md"),
        "the refusal names the malformed file: {}",
        error.message
    );
}

// 11. kaya.publications.stub_free — an empty block is no backing document
#[test]
fn whitespace_only_block_is_treated_as_absent() {
    let tmp = TempDir::new().expect("temp dir");
    let research = tmp.path().join("research");
    fs::create_dir_all(&research).expect("research tree is creatable");
    let index = format!(
        "<!-- kaya:begin generated:{OVERVIEW_KIND} -->\n   \n\n\
         <!-- kaya:end generated:{OVERVIEW_KIND} -->\n"
    );
    fs::write(research.join("index.md"), index).expect("index is writable");

    let docs = read_documents(&research).expect("the compiled documents are readable");
    assert!(
        !docs.0.contains_key(OVERVIEW_KIND),
        "a whitespace-only generated block backs no document"
    );

    let out = tmp.path().join("out");
    let decls = [
        product("overview", &[OVERVIEW_KIND]),
        package("project-site", &["overview"]),
    ];
    let built = build(&decls, &docs, &out).expect("the omission must build");
    assert!(
        !out.join("overview.md").exists(),
        "an empty backing document produces no stub page"
    );
    assert!(
        built
            .omitted
            .iter()
            .any(|entry| { entry.name == "overview" && entry.reason == "no-backing-document" }),
        "the empty-backing product is reported omitted: {:?}",
        built.omitted
    );
    let package_page =
        fs::read_to_string(out.join("project-site.md")).expect("the package page is written");
    assert!(
        block_body(&package_page, "publication.package.project-site")
            .contains("- overview — unavailable"),
        "the package names the omitted product unavailable: {package_page}"
    );
}

// 12. kaya.publications.format_attribution — build writes one managed block

// 26. kaya.publications.io — the block name, not the file name, names the kind
#[test]
fn read_documents_reads_a_vocabulary_kind_from_a_non_canonical_file() {
    let tmp = TempDir::new().expect("temp dir");
    let research = tmp.path().join("research");
    fs::create_dir_all(&research).expect("research tree is creatable");
    // A reuse-handoff kind compiled into index.md — the file a reader keyed on
    // canonical per-kind paths would never consult.
    let index = format!(
        "# Research index\n\n\
         <!-- kaya:begin generated:{TECH_KIND} -->\n\
         {TECH_BODY} cites kaya/Document/technical-reference\n\
         <!-- kaya:end generated:{TECH_KIND} -->\n\n\
         <!-- kaya:begin generated:reuse-handoff.other-block -->\n\
         outside the vocabulary, cites kaya/Document/ignored\n\
         <!-- kaya:end generated:reuse-handoff.other-block -->\n"
    );
    fs::write(research.join("index.md"), index).expect("index is writable");

    let docs = read_documents(&research).expect("the compiled documents are readable");

    assert_eq!(
        docs.0.len(),
        1,
        "only document-vocabulary kinds are kept: {:?}",
        docs.0.keys().collect::<Vec<_>>()
    );
    let technical = &docs.0[TECH_KIND];
    assert_eq!(
        technical.kind, TECH_KIND,
        "the DocumentRef names the kind its block declares"
    );
    assert!(
        technical.body.contains(TECH_BODY),
        "the block is read from index.md even though no technical-reference.md exists: {}",
        technical.body
    );
    assert_eq!(
        technical.event_ids,
        ["kaya/Document/technical-reference"],
        "event ids come from the block body"
    );
    assert!(
        !docs.0.contains_key("reuse-handoff.other-block"),
        "a generated block outside the document vocabulary is not a document"
    );
    assert!(
        !research.join("technical-reference.md").exists(),
        "the kind is not restricted to a canonical filename"
    );

    // The reference product takes its backing from the block in index.md.
    let out = tmp.path().join("out");
    let decls = [
        product("reference", &[TECH_KIND]),
        package("project-site", &["reference"]),
    ];
    let built = build(&decls, &docs, &out).expect("the non-canonical file backs its product");
    assert!(
        built.pages.iter().any(|page| page.name == "reference"),
        "the reference product is emitted: {:?}",
        built.pages
    );
    assert!(
        !built.omitted.iter().any(|entry| entry.name == "reference"),
        "the technical-reference block leaves the product unomitted: {:?}",
        built.omitted
    );
    let page = fs::read_to_string(out.join("reference.md")).expect("the reference page is written");
    assert!(
        page.contains(TECH_BODY),
        "the page renders the compiled technical-reference body: {page}"
    );
}

// 12. kaya.publications.block_vocabulary — a publication page's managed block
// name is inside the publication vocabulary, and a stale or misspelled one is
// refused before anything is written.
