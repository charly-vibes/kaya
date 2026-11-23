//! Publication declarations: the fixed set `kaya publish` composes.
//!
//! Purpose: declare, once, which products and packages the CLI publishes.
//! Responsibilities: name the default products and packages and bind each
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
/// developer-guide products, and the status, release, and architecture
/// products. Every include is the product's admission row, so a declaration can
/// never widen a product beyond its contract. `tutorial` stays undeclared until
/// its backing exploration documents have producers.
const DEFAULT_PRODUCTS: [&str; 11] = [
    "overview",
    "reference",
    "getting-started",
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

/// The declarations `kaya publish` composes, and the ones `kaya check`
/// expects: eleven products and two packages over the compiled research
/// documents.
///
/// The products are `overview`, `reference`, `getting-started`, `how-to`,
/// `concept`, `example`, `troubleshooting`, `faq`, `status-compatibility`,
/// `release-migration`, and `architecture-design`. The packages are
/// `project-site` (over `overview` and `reference`) and `developer-guide`
/// (over the six `getting-started` through `faq` products).
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
        .collect()
}
