//! Publication declarations: the fixed set `kaya publish` composes.
//!
//! Purpose: declare, once, which products, packages, and collections the CLI
//! publishes. Responsibilities: name the default products, packages, and the
//! glossary collection and bind each
//! product's includes to its admission row. Rationale: declarations derived
//! from the admission table can never widen a product beyond its contract,
//! and `kaya check` expects exactly what `kaya publish` declares.

use super::vocabulary::{Tier, admission};

/// One publication declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decl {
    /// Declared tier.
    pub tier: Tier,
    /// Declared name from the tier's vocabulary.
    pub name: String,
    /// Document kinds composed by a product, in declaration order.
    pub includes: Vec<String>,
    /// Product names referenced by a package, in declaration order.
    pub members: Vec<String>,
}

/// Products `kaya publish` declares: the project-site pair, the six
/// developer-guide products, the status and release products, and the
/// architecture product — now including `tutorial`, whose admission rows
/// (exploration.experiment-record, exploration.exploration-brief) have
/// producers in the research projection. Every include is the product's
/// admission row, so a declaration can never widen a product beyond its
/// contract. Until exploration documents carry records, tutorial publishes
/// as the spec-pinned `no-backing-document` omission instead of a page.
const DEFAULT_PRODUCTS: [&str; 12] = [
    "overview",
    "reference",
    "getting-started",
    "tutorial",
    "how-to",
    "concept",
    "example",
    "troubleshooting",
    "faq",
    "status-compatibility",
    "release-migration",
    "architecture-design",
];

/// Packages `kaya publish` declares, with their members in declaration order.
const DEFAULT_PACKAGES: [(&str, &[&str]); 2] = [
    ("project-site", &["overview", "reference"]),
    (
        "developer-guide",
        &[
            "getting-started",
            "how-to",
            "concept",
            "example",
            "troubleshooting",
            "faq",
        ],
    ),
];

/// The collections `kaya publish` declares: the glossary, the only collection
/// with a renderer.
const DEFAULT_COLLECTIONS: [&str; 1] = ["glossary"];

/// The declarations `kaya publish` composes, and the ones `kaya check`
/// expects: twelve products, two packages, and the glossary collection over
/// the compiled research documents.
///
/// The products are `overview`, `reference`, `getting-started`, `tutorial`,
/// `how-to`, `concept`, `example`, `troubleshooting`, `faq`,
/// `status-compatibility`, `release-migration`, and `architecture-design`. The packages are
/// `project-site` (over `overview` and `reference`) and `developer-guide`
/// (over the six `getting-started` through `faq` products). The collection is
/// `glossary`, rendered from the compile-time vocabularies.
pub fn default_declarations() -> Vec<Decl> {
    DEFAULT_PRODUCTS
        .iter()
        .map(|name| Decl {
            tier: Tier::Product,
            name: (*name).to_owned(),
            includes: admission(name)
                .iter()
                .map(|kind| (*kind).to_owned())
                .collect(),
            members: Vec::new(),
        })
        .chain(DEFAULT_PACKAGES.iter().map(|(name, members)| Decl {
            tier: Tier::Package,
            name: (*name).to_owned(),
            includes: Vec::new(),
            members: members.iter().map(|member| (*member).to_owned()).collect(),
        }))
        .chain(DEFAULT_COLLECTIONS.iter().map(|name| Decl {
            tier: Tier::Collection,
            name: (*name).to_owned(),
            includes: Vec::new(),
            members: Vec::new(),
        }))
        .collect()
}
