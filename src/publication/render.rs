//! Page composition and the structured export.
//!
//! Purpose: turn the validated declarations and the source map into page
//! documents and export rows. Responsibilities: compose each product's
//! kinds in declaration order, render each fact body once with later products
//! cross-referencing the first holder, render packages as member lists, render
//! the glossary collection from the compile-time vocabularies, and
//! project the export from the same source map.
//! Rationale: one rendering per fact keeps pages free of duplicated content,
//! and the export reusing the source map keeps Markdown and JSON from
//! disagreeing.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::json;

use super::build::Omitted;
use super::collection::collection_page;
use super::decls::Decl;
use super::source::Documents;
use super::vocabulary::{REASON_NO_BACKING_DOCUMENT, Tier, reader_question};

/// Compose each product's kinds: emitted products keep declaration order,
/// unbacked ones are omitted with the documented reason.
pub(super) fn compose_products(
    products: &[&Decl],
    docs: &Documents,
) -> (BTreeMap<String, Vec<String>>, Vec<Omitted>) {
    let mut composed: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut omitted: Vec<Omitted> = Vec::new();
    for product in products {
        let mut kinds: Vec<String> = Vec::new();
        for kind in &product.includes {
            if docs.0.contains_key(kind) && !kinds.contains(kind) {
                kinds.push(kind.clone());
            }
        }
        if kinds.is_empty() {
            omitted.push(Omitted {
                tier: Tier::Product.as_str(),
                name: product.name.clone(),
                reason: REASON_NO_BACKING_DOCUMENT,
            });
        } else {
            composed.insert(product.name.clone(), kinds);
        }
    }
    omitted.sort_by(|left, right| left.name.cmp(&right.name));
    (composed, omitted)
}

/// Who renders each fact, and which publication names are emitted.
pub(super) struct Ownership {
    /// The first emitted product carrying each document kind's body.
    pub(super) owners: BTreeMap<String, String>,
    /// The names of products with a backing document.
    pub(super) emitted: BTreeSet<String>,
}

/// Compose the collections: render the implemented ones whose rows come out
/// populated, and report the rest as omissions with the standard reason.
///
/// Rendering and emission share one call into [`collection_page`], so the
/// page list and the export can never disagree about whether a collection
/// emitted.
pub(super) fn compose_collections(
    collections: &[&Decl],
    docs: &Documents,
    ownership: &Ownership,
) -> (Vec<(String, String)>, Vec<Omitted>) {
    let mut pages: Vec<(String, String)> = Vec::new();
    let mut omitted: Vec<Omitted> = Vec::new();
    for collection in collections {
        match collection_page(
            &collection.name,
            docs,
            ownership
                .owners
                .get("orientation.research-map")
                .map(|owner| owner.as_str()),
        ) {
            Some(page) => pages.push((collection.name.clone(), page)),
            None => omitted.push(Omitted {
                tier: Tier::Collection.as_str(),
                name: collection.name.clone(),
                reason: REASON_NO_BACKING_DOCUMENT,
            }),
        }
    }
    (pages, omitted)
}

/// Assign each kind to the first emitted product that composes it, and
/// collect the emitted names for package member availability.
pub(super) fn assign_owners(composed: &BTreeMap<String, Vec<String>>) -> Ownership {
    let mut owners = BTreeMap::new();
    let mut emitted = BTreeSet::new();
    for (name, kinds) in composed {
        emitted.insert(name.clone());
        for kind in kinds {
            owners.entry(kind.clone()).or_insert_with(|| name.clone());
        }
    }
    Ownership { owners, emitted }
}

/// Render every page document: products first, then packages, sorted by name.
///
/// `collections` arrives already composed — `compose_collections` rendered
/// the backed ones and reported the empty ones as omissions — so this loop
/// never re-derives emission.
pub(super) fn render_pages(
    products: &[&Decl],
    packages: &[&Decl],
    collections: &[(String, String)],
    composed: &BTreeMap<String, Vec<String>>,
    ownership: &Ownership,
    docs: &Documents,
) -> Vec<(String, Tier, String)> {
    let mut rendered: Vec<(String, Tier, String)> = Vec::new();
    for product in products {
        let Some(kinds) = composed.get(&product.name) else {
            continue;
        };
        let page = render_product_page(product, kinds, &ownership.owners, docs);
        rendered.push((product.name.clone(), Tier::Product, page));
    }
    for package in packages {
        let page = render_package_page(package, &ownership.emitted);
        rendered.push((package.name.clone(), Tier::Package, page));
    }
    for (name, page) in collections {
        rendered.push((name.clone(), Tier::Collection, page.clone()));
    }
    rendered.sort_by(|left, right| left.0.cmp(&right.0));
    rendered
}

/// Render one product page: each composed kind's body, or a cross-reference
/// to the product that already carries the fact.
fn render_product_page(
    product: &Decl,
    kinds: &[String],
    owners: &BTreeMap<String, String>,
    docs: &Documents,
) -> String {
    let question = reader_question(&product.name).unwrap_or("");
    let mut page = format!(
        "## {}\n\nReader question: {question}\n\n",
        display_name(&product.name)
    );
    for kind in kinds {
        page.push_str(&format!("### {kind}\n\n"));
        let holder = &owners[kind];
        if holder == &product.name {
            let body = docs
                .0
                .get(kind)
                .map(|document| document.body.trim_end_matches('\n'))
                .unwrap_or("");
            page.push_str(body);
            page.push('\n');
        } else {
            let anchor = heading_anchor(kind);
            page.push_str(&format!("See [{holder}]({holder}.md#{anchor}).\n"));
        }
        page.push('\n');
    }
    page
}

/// Render one package page: each member as a link, or as unavailable.
fn render_package_page(package: &Decl, emitted: &BTreeSet<String>) -> String {
    let mut page = format!("## {}\n\n", display_name(&package.name));
    for member in &package.members {
        if emitted.contains(member) {
            page.push_str(&format!("- [{member}]({member}.md)\n"));
        } else {
            page.push_str(&format!("- {member} — unavailable\n"));
        }
    }
    page
}

/// The structured export, projected from the same source map as the pages.
pub(super) fn export_json(
    products: &[&Decl],
    packages: &[&Decl],
    collections: &[&Decl],
    composed: &BTreeMap<String, Vec<String>>,
    emitted: &BTreeSet<String>,
    omitted: &[Omitted],
    docs: &Documents,
) -> serde_json::Value {
    let mut product_rows: Vec<(String, serde_json::Value)> = products
        .iter()
        .map(|product| export_product_row(product, composed, docs))
        .collect();
    product_rows.sort_by(|left, right| left.0.cmp(&right.0));
    let mut package_rows: Vec<(String, serde_json::Value)> = packages
        .iter()
        .map(|package| export_package_row(package, emitted))
        .collect();
    package_rows.sort_by(|left, right| left.0.cmp(&right.0));
    // The omission summary is the single source of collection emission: a
    // declared collection absent from it emitted, one present in it carries
    // its reason into the export row.
    let omissions: BTreeMap<&str, &str> = omitted
        .iter()
        .filter(|entry| entry.tier == Tier::Collection.as_str())
        .map(|entry| (entry.name.as_str(), entry.reason))
        .collect();
    let mut collection_rows: Vec<(String, serde_json::Value)> = collections
        .iter()
        .map(|collection| match omissions.get(collection.name.as_str()) {
            Some(reason) => (
                collection.name.clone(),
                json!({
                    "emitted": false,
                    "name": collection.name,
                    "reason": reason,
                }),
            ),
            None => (
                collection.name.clone(),
                json!({
                    "emitted": true,
                    "name": collection.name,
                }),
            ),
        })
        .collect();
    collection_rows.sort_by(|left, right| left.0.cmp(&right.0));
    json!({
        "products": product_rows
            .into_iter()
            .map(|(_, row)| row)
            .collect::<Vec<_>>(),
        "packages": package_rows
            .into_iter()
            .map(|(_, row)| row)
            .collect::<Vec<_>>(),
        "collections": collection_rows
            .into_iter()
            .map(|(_, row)| row)
            .collect::<Vec<_>>(),
    })
}

/// One product's export row: name, fixed question, emission, and each
/// composed kind's event ids.
fn export_product_row(
    product: &Decl,
    composed: &BTreeMap<String, Vec<String>>,
    docs: &Documents,
) -> (String, serde_json::Value) {
    let documents: Vec<serde_json::Value> = composed
        .get(&product.name)
        .into_iter()
        .flatten()
        .map(|kind| {
            let event_ids = docs
                .0
                .get(kind)
                .map(|document| document.event_ids.clone())
                .unwrap_or_default();
            json!({ "kind": kind, "event_ids": event_ids })
        })
        .collect();
    (
        product.name.clone(),
        json!({
            "name": product.name,
            "question": reader_question(&product.name),
            "emitted": composed.contains_key(&product.name),
            "documents": documents,
        }),
    )
}

/// One package's export row: name and each member's availability.
fn export_package_row(package: &Decl, emitted: &BTreeSet<String>) -> (String, serde_json::Value) {
    let members: Vec<serde_json::Value> = package
        .members
        .iter()
        .map(|member| {
            json!({
                "name": member,
                "emitted": emitted.contains(member),
            })
        })
        .collect();
    (
        package.name.clone(),
        json!({
            "name": package.name,
            "members": members,
        }),
    )
}

/// Capitalize the first letter of a page name for its heading.
fn display_name(name: &str) -> String {
    let mut chars = name.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// The heading anchor a document kind is listed under on its holder's page.
///
/// The page lists each kind as an `### <kind>` heading, whose anchor keeps only
/// alphanumerics, hyphens and underscores — turning each space into a hyphen
/// and dropping every other character — so `orientation.project-overview`
/// anchors as `orientationproject-overview`.
pub(super) fn heading_anchor(kind: &str) -> String {
    let mut anchor = String::with_capacity(kind.len());
    for character in kind.chars() {
        if character == ' ' {
            anchor.push('-');
        } else if character.is_alphanumeric() || character == '-' || character == '_' {
            anchor.push(character);
        }
    }
    anchor
}
