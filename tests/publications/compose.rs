//! Composition: one fact rendered once, cross-referenced elsewhere, and
//! unbacked products omitted without stub pages.

use std::path::Path;

use tempfile::TempDir;

use kaya::publication::build;

use crate::common::*;

/// Two products sharing one overview fact, composed into the project-site
/// package.
fn shared_fact_built(out: &Path) -> kaya::publication::Built {
    let decls = [
        product("overview", &[OVERVIEW_KIND]),
        product("getting-started", &[OVERVIEW_KIND]),
        package("project-site", &["overview", "getting-started"]),
    ];
    let docs = documents(&[(OVERVIEW_KIND, OVERVIEW_BODY, &["evt-1"])]);
    match build(&decls, &docs, out) {
        Ok(built) => built,
        Err(error) => panic!(
            "the composition must build: {}: {}",
            error.label, error.message
        ),
    }
}

#[test]
fn shared_fact_is_rendered_once_with_a_cross_reference() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    let built = shared_fact_built(&out);

    let names: Vec<&str> = built.pages.iter().map(|page| page.name.as_str()).collect();
    assert_eq!(
        names,
        ["getting-started", "overview", "project-site"],
        "product and package pages are sorted by name"
    );

    let tree = read_tree(&out);
    let pages = markdown(&tree);
    let occurrences: usize = pages
        .values()
        .map(|text| text.matches(OVERVIEW_BODY).count())
        .sum();
    assert_eq!(
        occurrences, 1,
        "the shared fact body is rendered exactly once across the published tree"
    );

    let holder_file = pages
        .iter()
        .find(|(_, text)| text.contains(OVERVIEW_BODY))
        .map(|(path, _)| path.clone())
        .expect("one product page carries the fact body");
    let holder = holder_file.trim_end_matches(".md").to_owned();
    assert!(
        holder == "overview" || holder == "getting-started",
        "the fact body lives on a product page, not the package: {holder}"
    );
    let other = if holder == "overview" {
        "getting-started"
    } else {
        "overview"
    };

    assert!(
        pages[&holder_file].contains(&format!("### {OVERVIEW_KIND}")),
        "the rendering product carries the document block: {}",
        pages[&holder_file]
    );
    let other_page = pages
        .get(&format!("{other}.md"))
        .unwrap_or_else(|| panic!("{other} is emitted"));
    assert!(
        other_page.contains(&format!("See [{holder}]({holder}.md#{OVERVIEW_ANCHOR}).")),
        "the second product cross-references the first instead of repeating the body: {other_page}"
    );
    assert!(
        !other_page.contains(OVERVIEW_BODY),
        "the second product does not duplicate the shared fact body"
    );

    let package_page = pages
        .get("project-site.md")
        .expect("the package page is written");
    assert!(
        !package_page.contains(OVERVIEW_BODY),
        "the package embeds no product body text: {package_page}"
    );
    assert_package_member_rows(
        package_page,
        &[
            "- [overview](overview.md)",
            "- [getting-started](getting-started.md)",
        ],
    );
}

#[test]
fn product_without_backing_document_is_omitted_without_a_page() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");

    let decls = [
        product("tutorial", &["exploration.experiment-record"]),
        package("project-site", &["tutorial"]),
    ];
    let docs = documents(&[(OVERVIEW_KIND, OVERVIEW_BODY, &["evt-1"])]);

    let built = match build(&decls, &docs, &out) {
        Ok(built) => built,
        Err(error) => panic!(
            "the omission must build: {}: {}",
            error.label, error.message
        ),
    };

    let names: Vec<&str> = built.pages.iter().map(|page| page.name.as_str()).collect();
    assert_eq!(
        names,
        ["project-site"],
        "an unbacked product gets no page; only the package is emitted"
    );
    assert!(
        !out.join("tutorial.md").exists(),
        "no stub page exists for a product without a backing document"
    );

    let omitted = built
        .omitted
        .iter()
        .find(|entry| entry.name == "tutorial")
        .expect("the omission of tutorial is reported");
    assert_eq!(
        omitted.reason, "no-backing-document",
        "the omission names its reason"
    );
    assert!(
        omitted.tier.contains("product"),
        "the omission reports the product tier: {}",
        omitted.tier
    );

    let tree = read_tree(&out);
    let pages = markdown(&tree);
    let package_page = pages
        .get("project-site.md")
        .expect("the package page is written");
    let package_body = block_body(package_page, "publication.package.project-site");
    let rows: Vec<&str> = package_body
        .lines()
        .filter(|line| !line.is_empty())
        .collect();
    assert_eq!(
        rows.len(),
        2,
        "the package block is a heading and one row per member: {package_page}"
    );
    assert!(
        rows[0].starts_with("## "),
        "the package block opens with a heading: {package_page}"
    );
    assert_eq!(
        rows[1], "- tutorial — unavailable",
        "the package names the omitted product unavailable instead of linking a missing page"
    );
}

// 6. kaya.publications.format_attribution
