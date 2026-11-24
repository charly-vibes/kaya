//! The default declaration set tracks the closed product vocabulary: every
//! vocabulary product is declared with its spec admission row, so a new
//! generator's records surface in publish without a declarations edit.

use kaya::publication::{PRODUCTS, Tier, admission, default_declarations};

#[test]
fn default_declarations_cover_the_whole_product_vocabulary() {
    let declared = default_declarations();
    let names: Vec<&str> = declared
        .iter()
        .filter(|decl| decl.tier == Tier::Product)
        .map(|decl| decl.name.as_str())
        .collect();

    assert_eq!(
        names.len(),
        PRODUCTS.len(),
        "one declaration per vocabulary product: {names:?}"
    );
    for product in PRODUCTS {
        assert!(
            names.contains(&product),
            "vocabulary product \"{product}\" must be declared: {names:?}"
        );
    }

    for decl in declared.iter().filter(|decl| decl.tier == Tier::Product) {
        assert_eq!(
            decl.tier,
            Tier::Product,
            "default product declarations are product-tier: {}",
            decl.name
        );
        assert_eq!(
            decl.includes.as_slice(),
            admission(&decl.name),
            "declaration \"{}\" includes its spec admission row",
            decl.name
        );
        assert!(
            decl.members.is_empty(),
            "product declarations take no members: {}",
            decl.name
        );
    }
}

#[test]
fn tutorial_is_declared_with_its_spec_admission_row() {
    let tutorial = default_declarations()
        .into_iter()
        .find(|decl| decl.name == "tutorial")
        .expect("tutorial is a vocabulary product and must be declared");
    assert_eq!(
        tutorial.includes.as_slice(),
        &[
            "exploration.experiment-record",
            "exploration.exploration-brief"
        ],
        "tutorial admits its spec exploration rows"
    );
}
