//! Labeled publication refusals.
//!
//! Purpose: carry every publication refusal as one labeled error naming the
//! offending value. Responsibilities: construct the refusal for each failure
//! class the module detects, quoting the exact vocabularies or rows involved.
//! Rationale: a refusal a reader can act on names the tier, name, row, or
//! block it rejects — never a bare message.

use std::io;
use std::path::Path;

use super::vocabulary::{
    COLLECTIONS, LABEL_BLOCK_VOCABULARY, LABEL_DOCUMENT_KIND, LABEL_DUPLICATE_BLOCK,
    LABEL_DUPLICATE_DECLARATION, LABEL_INADMISSIBLE, LABEL_IO, LABEL_PACKAGE_MEMBERS,
    LABEL_TIER_FIELDS, LABEL_TIER_RESOLVED, LABEL_TIER_UNIMPLEMENTED, PACKAGES, PRODUCTS, Tier,
};

/// A labeled publication refusal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicationError {
    /// Failure class, for example `kaya.publications.tier_resolved`.
    pub label: String,
    /// Human-readable explanation naming the offending value.
    pub message: String,
}

impl PublicationError {
    pub(super) fn new(label: &str, message: String) -> Self {
        Self {
            label: label.to_owned(),
            message,
        }
    }

    pub(super) fn io(action: &str, path: &Path, error: io::Error) -> Self {
        Self::new(
            LABEL_IO,
            format!("failed to {action} \"{}\": {error}", path.display()),
        )
    }

    pub(super) fn unknown_tier(value: &str) -> Self {
        Self::new(
            LABEL_TIER_RESOLVED,
            format!(
                "unknown publication tier or name \"{value}\"; products: {}; \
                 collections: {}; packages: {}",
                PRODUCTS.join(", "),
                COLLECTIONS.join(", "),
                PACKAGES.join(", ")
            ),
        )
    }

    pub(super) fn tier_mismatch(value: &str, declared: Tier, resolved: Tier) -> Self {
        Self::new(
            LABEL_TIER_RESOLVED,
            format!(
                "publication \"{value}\" resolves to the {} tier, not the declared {} tier; \
                 products: {}; collections: {}; packages: {}",
                resolved.as_str(),
                declared.as_str(),
                PRODUCTS.join(", "),
                COLLECTIONS.join(", "),
                PACKAGES.join(", ")
            ),
        )
    }

    pub(super) fn inadmissible(product: &str, kind: &str, row: &[&'static str]) -> Self {
        Self::new(
            LABEL_INADMISSIBLE,
            format!(
                "product \"{product}\" cannot include \"{kind}\": the admission row is {}",
                row.join(", ")
            ),
        )
    }

    pub(super) fn package_member(package: &str, member: &str) -> Self {
        Self::new(
            LABEL_PACKAGE_MEMBERS,
            format!(
                "package \"{package}\" cannot list \"{member}\" as a member: the package \
                 tier lists declared products, not documents, collections, or undeclared \
                 names; products are: {}",
                PRODUCTS.join(", ")
            ),
        )
    }

    pub(super) fn duplicate_declaration(name: &str) -> Self {
        Self::new(
            LABEL_DUPLICATE_DECLARATION,
            format!(
                "publication \"{name}\" is declared more than once: a name resolves to one \
                 tier and is declared once"
            ),
        )
    }

    pub(super) fn tier_fields(name: &str, tier: Tier, field: &str) -> Self {
        Self::new(
            LABEL_TIER_FIELDS,
            format!(
                "publication \"{name}\" declares the {} tier but carries \"{field}\": a \
                 product lists \"includes\" and a package lists \"members\"",
                tier.as_str()
            ),
        )
    }

    pub(super) fn tier_unimplemented(name: &str) -> Self {
        Self::new(
            LABEL_TIER_UNIMPLEMENTED,
            format!(
                "collection \"{name}\" cannot be declared: only the glossary collection is \
                 implemented, so this release refuses every other collection"
            ),
        )
    }

    pub(super) fn document_kind(key: &str, kind: &str) -> Self {
        Self::new(
            LABEL_DOCUMENT_KIND,
            format!(
                "document source key \"{key}\" holds a document of kind \"{kind}\": the map \
                 key and the document's kind must name the same document"
            ),
        )
    }

    pub(super) fn duplicate_block(file: &Path, block: &str) -> Self {
        Self::new(
            LABEL_DUPLICATE_BLOCK,
            format!(
                "failed to read \"{}\": generated block \"{block}\" appears more than once \
                 in one compiled document",
                file.display()
            ),
        )
    }

    pub(super) fn block_vocabulary(file: &Path, block: &str) -> Self {
        Self::new(
            LABEL_BLOCK_VOCABULARY,
            format!(
                "failed to read \"{}\": generated block \"{block}\" is outside the publication \
                 block vocabulary: a publication page carries one publication.<tier>.<name> block, \
                 with the tier and the name from that tier's closed vocabulary",
                file.display()
            ),
        )
    }
}

impl std::fmt::Display for PublicationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.label, self.message)
    }
}

impl std::error::Error for PublicationError {}
