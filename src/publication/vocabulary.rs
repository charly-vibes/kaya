//! Closed publication vocabularies and the fixed metadata bound to them.
//!
//! Purpose: name the closed product/collection/package vocabularies and the
//! labels, reasons, reader questions, and admission rows attached to them.
//! Responsibilities: resolve a name to exactly one tier or refuse it, bind
//! each product to its fixed reader question and admission row, and expose
//! the refusal-label vocabulary the rest of the module reports through.
//! Rationale: closed vocabularies make every publication name a checkable
//! token, and fixed admission rows make a product's composition a contract
//! rather than a per-build choice.

use super::error::PublicationError;

/// Closed product vocabulary: reader-facing pages, one per reader question.
pub const PRODUCTS: [&str; 12] = [
    "overview",
    "getting-started",
    "tutorial",
    "how-to",
    "concept",
    "reference",
    "example",
    "troubleshooting",
    "faq",
    "status-compatibility",
    "release-migration",
    "architecture-design",
];

/// Closed collection vocabulary: cross-cutting groupings of products or
/// documents.
pub const COLLECTIONS: [&str; 9] = [
    "glossary",
    "example-gallery",
    "api-catalogue",
    "capability-catalogue",
    "compatibility-matrix",
    "feature-index",
    "knowledge-index",
    "related-projects",
    "topic-map",
];

/// Closed package vocabulary: navigation bundles over products.
pub const PACKAGES: [&str; 8] = [
    "project-site",
    "developer-guide",
    "user-guide",
    "api-documentation",
    "integration-guide",
    "research-background",
    "ai-knowledge-package",
    "offline-bundle",
];

/// Label for an unknown tier or name.
pub const LABEL_TIER_RESOLVED: &str = "kaya.publications.tier_resolved";
/// Label for a document included outside a product's admission row.
pub const LABEL_INADMISSIBLE: &str = "kaya.publications.inadmissible_refused";
/// Label for a package member that is not a declared product.
pub const LABEL_PACKAGE_MEMBERS: &str = "kaya.publications.package_members";
/// Label for a publication name declared more than once.
pub const LABEL_DUPLICATE_DECLARATION: &str = "kaya.publications.duplicate_declaration";
/// Label for a declaration carrying a field its tier does not use.
pub const LABEL_TIER_FIELDS: &str = "kaya.publications.tier_fields";
/// Label for a declared collection, a tier with no implementation yet.
pub const LABEL_TIER_UNIMPLEMENTED: &str = "kaya.publications.tier_unimplemented";
/// Label for a document whose kind disagrees with its source-map key.
pub const LABEL_DOCUMENT_KIND: &str = "kaya.publications.document_kind";
/// Label for a generated block repeated inside one compiled document.
pub const LABEL_DUPLICATE_BLOCK: &str = "kaya.publications.duplicate_block";
/// Label for a managed block name outside the publication block vocabulary.
pub const LABEL_BLOCK_VOCABULARY: &str = "kaya.publications.block_vocabulary";
/// Label for document read and publication write failures.
pub const LABEL_IO: &str = "kaya.publications.io";

/// Reason reported for a product with no admissible backing document.
pub const REASON_NO_BACKING_DOCUMENT: &str = "no-backing-document";

/// The one tier a publication name resolves to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tier {
    /// Reader-facing product.
    Product,
    /// Cross-cutting collection.
    Collection,
    /// Navigation package over products.
    Package,
}

impl Tier {
    /// The tier's vocabulary name.
    pub fn as_str(self) -> &'static str {
        match self {
            Tier::Product => "product",
            Tier::Collection => "collection",
            Tier::Package => "package",
        }
    }
}

/// Resolve a name against the three closed vocabularies.
///
/// The pair resolves to exactly one tier or is refused with a labeled error
/// quoting the value and listing every vocabulary entry.
pub fn resolve_name(name: &str) -> Result<(Tier, &'static str), PublicationError> {
    if let Some(found) = PRODUCTS.iter().find(|product| **product == name) {
        return Ok((Tier::Product, found));
    }
    if let Some(found) = COLLECTIONS.iter().find(|collection| **collection == name) {
        return Ok((Tier::Collection, found));
    }
    if let Some(found) = PACKAGES.iter().find(|package| **package == name) {
        return Ok((Tier::Package, found));
    }
    Err(PublicationError::unknown_tier(name))
}

/// The tier a tier-word names, if it names one.
pub(super) fn tier_for_name(name: &str) -> Option<Tier> {
    match name {
        "product" => Some(Tier::Product),
        "collection" => Some(Tier::Collection),
        "package" => Some(Tier::Package),
        _ => None,
    }
}

/// The fixed reader question bound to a product.
pub fn reader_question(product: &str) -> Option<&'static str> {
    Some(match product {
        "overview" => "what is this and why does it matter",
        "getting-started" => "how do I reach a first result",
        "tutorial" => "can you teach me through a guided outcome",
        "how-to" => "how do I accomplish this task",
        "concept" => "why does it work this way",
        "reference" => "what exactly is this",
        "example" => "what can I do with it",
        "troubleshooting" => "why is this broken and how do I fix it",
        "faq" => "what do people keep asking",
        "status-compatibility" => "what can I rely on",
        "release-migration" => "what changed and what must I do",
        "architecture-design" => "how is it structured and why",
        _ => return None,
    })
}

/// The admission rows: the document kinds each product may compose.
///
/// Naming an unknown product yields no row, so [`admission`] returns an empty
/// slice and every inclusion is refused rather than silently admitted.
const ADMISSION_ROWS: [(&str, &[&str]); 12] = [
    ("overview", &["orientation.project-overview"]),
    (
        "getting-started",
        &[
            "orientation.project-overview",
            "reuse-handoff.reproduction-package",
        ],
    ),
    (
        "tutorial",
        &[
            "exploration.experiment-record",
            "exploration.exploration-brief",
        ],
    ),
    (
        "how-to",
        &[
            "reuse-handoff.technical-reference",
            "reuse-handoff.reproduction-package",
        ],
    ),
    (
        "concept",
        &[
            "understanding.decision-record",
            "understanding.findings-and-analysis",
        ],
    ),
    (
        "reference",
        &[
            "reuse-handoff.technical-reference",
            "orientation.research-map",
        ],
    ),
    ("example", &["exploration.experiment-record"]),
    ("troubleshooting", &["understanding.findings-and-analysis"]),
    (
        "faq",
        &[
            "understanding.findings-and-analysis",
            "understanding.decision-record",
        ],
    ),
    (
        "status-compatibility",
        &[
            "orientation.project-overview",
            "exploration.experiment-record",
        ],
    ),
    (
        "release-migration",
        &[
            "orientation.project-overview",
            "understanding.decision-record",
        ],
    ),
    (
        "architecture-design",
        &["understanding.decision-record", "orientation.research-map"],
    ),
];

/// The document kinds admissible for a product.
///
/// Naming an unknown product yields an empty row, so every inclusion is
/// refused rather than silently admitted.
pub fn admission(product: &str) -> &'static [&'static str] {
    ADMISSION_ROWS
        .iter()
        .find(|(name, _)| *name == product)
        .map(|(_, row)| *row)
        .unwrap_or(&[])
}
