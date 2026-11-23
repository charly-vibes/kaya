//! The closed tier vocabularies, reader questions, and admission rows.

use std::collections::BTreeSet;
use std::fs;

use tempfile::TempDir;

use kaya::publication::{
    COLLECTIONS, PACKAGES, PRODUCTS, PublicationError, admission, build, reader_question,
    resolve_name,
};

use crate::common::*;

/// Every closed-vocabulary name resolves to exactly one tier, its own
/// spelling.
fn assert_every_name_resolves() {
    for (name, tier) in PRODUCTS
        .into_iter()
        .map(|name| (name, "product"))
        .chain(COLLECTIONS.into_iter().map(|name| (name, "collection")))
        .chain(PACKAGES.into_iter().map(|name| (name, "package")))
    {
        let (resolved_tier, resolved_name) = match resolve_name(name) {
            Ok(resolved) => resolved,
            Err(error) => panic!(
                "{name} must resolve to one tier: {}: {}",
                error.label, error.message
            ),
        };
        assert_eq!(
            kind_label(resolved_tier),
            tier,
            "{name} resolves to exactly one tier"
        );
        assert_eq!(resolved_name, name, "{name} resolves to its own spelling");
    }
}

/// An unknown name's refusal carries the tier label, quotes the value, and
/// lists every closed vocabulary entry.
fn assert_tier_refusal_is_complete(error: &PublicationError, value: &str) {
    assert_eq!(
        error.label, "kaya.publications.tier_resolved",
        "an unknown name is a labeled tier refusal: {}",
        error.message
    );
    assert!(
        error.message.contains(&format!("\"{value}\""))
            || error.message.contains(&format!("'{value}'")),
        "the refusal quotes the offending value `{value}`: {}",
        error.message
    );
    for entry in PRODUCTS.into_iter().chain(COLLECTIONS).chain(PACKAGES) {
        assert!(
            error.message.contains(entry),
            "the refusal lists the closed vocabularies, missing {entry}: {}",
            error.message
        );
    }
}

fn assert_unknown_names_refused() {
    // `experiment-report` is a plausible product name outside the closed
    // product vocabulary; `blog` is a plausible fourth tier name.
    for value in ["experiment-report", "blog"] {
        let error = match resolve_name(value) {
            Ok(_) => panic!("{value} is outside every closed vocabulary and must be refused"),
            Err(error) => error,
        };
        assert_tier_refusal_is_complete(&error, value);
    }
}

/// One product binds a reader question and an admission row of vocabulary kinds.
fn assert_product_admission_pinned(product: &str) {
    assert!(
        QUESTIONS.iter().any(|(name, _)| *name == product),
        "product {product} must declare a reader question"
    );
    assert!(
        reader_question(product).is_some(),
        "product {product} binds exactly one reader question"
    );
    let row = admission(product);
    assert!(
        !row.is_empty(),
        "product {product} admits at least one document kind"
    );
    for kind in row {
        assert!(
            DOCUMENT_KINDS.contains(kind),
            "the admission row for {product} names a kind outside the document vocabulary: {kind}"
        );
    }
}

/// Every product binds a reader question and an admission row of vocabulary
/// kinds, and nothing outside the vocabularies resolves.
fn assert_questions_and_admissions_pinned() {
    for product in PRODUCTS {
        assert_product_admission_pinned(product);
    }
    for (product, question) in QUESTIONS {
        assert_eq!(
            reader_question(product),
            Some(question),
            "product {product} binds its fixed reader question"
        );
    }
    assert!(
        reader_question("experiment-report").is_none(),
        "a name outside the product vocabulary has no reader question"
    );
}

#[test]
fn tier_resolved_for_every_name_and_unknown_refused() {
    assert_eq!(
        kind_label(resolve_name("overview").expect("overview resolves").0),
        "product",
        "a vocabulary name resolves to its tier"
    );
    assert_every_name_resolves();
    assert_unknown_names_refused();
    assert_questions_and_admissions_pinned();
}

// 1b. kaya.publications/Reader-question binding — the admission table is fixed
#[test]
fn admission_table_is_pinned_for_every_product() {
    for (product, row) in ADMISSIONS {
        assert_eq!(
            admission(product),
            row,
            "the admission row for {product} is fixed, in declaration order"
        );
        let mut seen = BTreeSet::new();
        for kind in row {
            assert!(
                DOCUMENT_KINDS.contains(kind),
                "{product} admits only document-vocabulary kinds, found {kind}"
            );
            assert!(
                seen.insert(*kind),
                "{product} admits {kind} once, never twice"
            );
        }
    }

    assert!(
        admission("experiment-report").is_empty(),
        "an unknown product admits nothing, so every inclusion is refused"
    );
}

// 2. kaya.publications.inadmissible_refused
#[test]
fn inadmissible_inclusion_is_refused() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    fs::create_dir(&out).expect("output directory");

    let decls = [product("faq", &[FINDINGS_KIND, SPEC_KIND])];
    let docs = documents(&[
        (FINDINGS_KIND, FINDINGS_BODY, &["evt-findings"]),
        (SPEC_KIND, SPEC_BODY, &["evt-spec"]),
    ]);

    let error = match build(&decls, &docs, &out) {
        Ok(_) => panic!("an include outside the faq admission row must be refused"),
        Err(error) => error,
    };

    assert_eq!(
        admission("faq"),
        &[
            "understanding.findings-and-analysis",
            "understanding.decision-record"
        ][..],
        "the faq admission row is fixed"
    );
    assert_eq!(
        error.label, "kaya.publications.inadmissible_refused",
        "an inadmissible inclusion is a labeled refusal: {}",
        error.message
    );
    assert!(
        error.message.contains("faq"),
        "the refusal names the product: {}",
        error.message
    );
    assert!(
        error.message.contains(SPEC_KIND),
        "the refusal names the offending kind: {}",
        error.message
    );
    for kind in admission("faq") {
        assert!(
            error.message.contains(*kind),
            "the refusal names the admission table row, missing {kind}: {}",
            error.message
        );
    }

    assert!(!out.join("faq.md").exists(), "no page is written");
    assert_no_writes(&out);
}

// 3. kaya.publications/Publication tiers — Package cannot list a family
#[test]
fn package_cannot_list_a_family() {
    let tmp = TempDir::new().expect("temp dir");
    let out = tmp.path().join("out");
    fs::create_dir(&out).expect("output directory");

    let decls = [package("project-site", &["exploration.experiment-record"])];
    let docs = documents(&[(OVERVIEW_KIND, OVERVIEW_BODY, &["evt-1"])]);

    let error = match build(&decls, &docs, &out) {
        Ok(_) => panic!("a package may only list products, never a document family"),
        Err(error) => error,
    };

    assert_eq!(
        error.label, "kaya.publications.package_members",
        "a non-product member is a labeled package refusal: {}",
        error.message
    );
    assert!(
        error.message.contains("exploration.experiment-record"),
        "the refusal names the family reference: {}",
        error.message
    );
    assert!(
        error.message.contains("package"),
        "the refusal names the package tier: {}",
        error.message
    );

    assert_no_writes(&out);
}

// 4. kaya.publications.fact_once
