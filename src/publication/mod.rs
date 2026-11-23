//! Client-facing publications composed from compiled documents.
//!
//! Purpose: project one document source into a publication tree — Markdown
//! pages plus a structured export — without duplicating a fact or inventing a
//! stub page.
//! Responsibilities: resolve the closed product/collection/package
//! vocabularies, bind each product to its fixed reader question and admission
//! row, refuse invalid declarations, inadmissible inclusions, non-product
//! package members, and duplicate or empty compiled documents before any
//! write, emit only backed products, render each fact body once with
//! cross-references elsewhere, and write deterministic pages — each a managed
//! block inside the document already at its path — plus a JSON export from the
//! same source map.
//! Rationale: a publication is a projection, not a second knowledge store.
//! Same inputs produce the same bytes, an unbacked product is omitted rather
//! than stubbed, and every exported attribution is the page's own event ids.

mod build;
mod decls;
mod error;
mod io;
mod plan;
mod render;
mod source;
mod vocabulary;

pub use build::{Built, Omitted, Page, build};
pub use decls::{Decl, default_declarations};
pub use error::PublicationError;
pub use plan::{Plan, PlannedPage, PlannedRemoval, plan};
pub use source::{DocumentRef, Documents, read_documents, read_documents_from};
pub use vocabulary::{
    COLLECTIONS, LABEL_BLOCK_VOCABULARY, LABEL_DOCUMENT_KIND, LABEL_DUPLICATE_BLOCK,
    LABEL_DUPLICATE_DECLARATION, LABEL_INADMISSIBLE, LABEL_IO, LABEL_PACKAGE_MEMBERS,
    LABEL_TIER_FIELDS, LABEL_TIER_RESOLVED, LABEL_TIER_UNIMPLEMENTED, PACKAGES, PRODUCTS,
    REASON_NO_BACKING_DOCUMENT, Tier, admission, reader_question, resolve_name,
};
