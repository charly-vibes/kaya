//! The publication write phase.
//!
//! Purpose: turn an approved plan into tree bytes and the build report.
//! Responsibilities: write only pages whose bytes change, count unchanged
//! blocks, remove a planned page only after every page and the export are in
//! place, and report pages, omissions, and the change summary.
//! Rationale: a failed write never leaves the tree missing a page it failed
//! to replace, and the summary names exactly which blocks the build touched.

use std::fs;
use std::path::Path;

use crate::projection::Summary;

use super::decls::Decl;
use super::error::PublicationError;
use super::io::{read_bytes, read_text, write_atomic};
use super::plan::{EXPORT_FILE, Plan};
use super::source::Documents;

/// A product omitted from the tree because it has no backing document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Omitted {
    /// Tier of the omitted publication.
    pub tier: &'static str,
    /// Name of the omitted publication.
    pub name: String,
    /// Why it was omitted, for example `no-backing-document`.
    pub reason: &'static str,
}

/// One emitted page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page {
    /// Publication name, also the file stem.
    pub name: String,
    /// Path written for this page.
    pub path: std::path::PathBuf,
    /// Fixed reader question, for products only.
    pub question: Option<&'static str>,
    /// Composed document kinds, in declaration order.
    pub documents: Vec<String>,
}

/// The result of one publication build.
#[derive(Debug, Clone, Default)]
pub struct Built {
    /// Emitted pages, sorted by name.
    pub pages: Vec<Page>,
    /// Products omitted for want of a backing document.
    pub omitted: Vec<Omitted>,
    /// Blocks newly written or found already current.
    pub summary: Summary,
}

/// Build a publication tree: refuse invalid declarations before any write,
/// emit backed products and every declared package, and write pages plus the
/// structured export only when their bytes change.
///
/// Every page is written as one `publication.<tier>.<name>` managed block
/// inside whatever document already exists at that path, so authored bytes
/// outside the block survive and only the block is compiler-owned.
pub fn build(decls: &[Decl], docs: &Documents, out: &Path) -> Result<Built, PublicationError> {
    let plan = super::plan::plan(decls, docs, out, read_text, read_bytes)?;

    fs::create_dir_all(out).map_err(|error| PublicationError::io("create", out, error))?;

    let mut summary = Summary::default();
    write_pages(&plan, &mut summary)?;

    // Only after every page and the export are in place are pages for omitted
    // products removed, so a failed write never leaves the tree missing a page
    // it failed to replace.
    apply_removals(&plan)?;

    Ok(Built {
        pages: plan.pages.into_iter().map(|page| page.page).collect(),
        omitted: plan.omitted,
        summary,
    })
}

/// Write every planned page and the export whose bytes changed, counting
/// unchanged blocks as the summary's unchanged set.
fn write_pages(plan: &Plan, summary: &mut Summary) -> Result<(), PublicationError> {
    for page in &plan.pages {
        if page.previous.as_deref() == Some(page.document.as_str()) {
            summary.unchanged.push(page.block.clone());
        } else {
            write_atomic(&page.path, page.document.as_bytes())?;
            summary.recomputed.push(page.block.clone());
        }
    }
    if plan.export_previous.as_deref() == Some(plan.export.as_slice()) {
        summary.unchanged.push(EXPORT_FILE.to_owned());
    } else {
        write_atomic(&plan.export_path, &plan.export)?;
        summary.recomputed.push(EXPORT_FILE.to_owned());
    }
    Ok(())
}

/// Apply every planned removal: unlink block-only documents, rewrite the
/// others to their stripped form, and leave authored documents untouched.
fn apply_removals(plan: &Plan) -> Result<(), PublicationError> {
    for removal in &plan.removals {
        match (&removal.previous, &removal.document) {
            (Some(_), None) => {
                fs::remove_file(&removal.path)
                    .map_err(|error| PublicationError::io("remove", &removal.path, error))?;
            }
            (Some(previous), Some(document)) if previous != document => {
                write_atomic(&removal.path, document.as_bytes())?;
            }
            _ => {}
        }
    }
    Ok(())
}
