//! The expected publication tree, computed without writing a byte.
//!
//! Purpose: resolve, validate, and render every page and the export before
//! any target is touched. Responsibilities: refuse documents filed under
//! keys they do not name, resolve declarations against the closed
//! vocabularies, check admission rows and package members, plan page
//! documents against the existing tree, and plan the removal of pages an
//! omitted product would have occupied.
//! Rationale: a refusal leaves the tree byte-untouched, an encode failure
//! cannot desynchronise the Markdown pages from the structured export, and
//! `kaya check` compares against the canonical expected tree rather than an
//! edit of whatever happens to exist.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::projection::{BlockKind, remove_generated, replace_generated, scan};

use super::build::{Omitted, Page};
use super::decls::Decl;
use super::error::PublicationError;
use super::io::compose_error;
use super::render::{assign_owners, compose_products, export_json, render_pages};
use super::source::Documents;
use super::vocabulary::{LABEL_IO, PRODUCTS, Tier, reader_question, resolve_name, tier_for_name};

/// The export file written beside the published pages.
pub(super) const EXPORT_FILE: &str = "publication.json";

/// One page's planned document, before the write phase.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedPage {
    /// Page metadata as the build reports it.
    pub page: Page,
    /// The managed block the page's content belongs to.
    pub block: String,
    /// Final path of the page.
    pub path: PathBuf,
    /// The whole document to write, block included.
    pub document: String,
    /// The document already on disk, when there is one.
    pub previous: Option<String>,
}

/// One omitted page's planned removal, before the write phase.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedRemoval {
    /// Path of the page an omitted publication would have occupied.
    pub path: PathBuf,
    /// The bytes already on disk, when there is a file.
    pub previous: Option<String>,
    /// The document to leave behind; `None` unlinks the page.
    pub document: Option<String>,
}

/// Every byte one publication build produces, before anything is written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    /// Emitted pages, sorted by name, with their bytes before and after.
    pub pages: Vec<PlannedPage>,
    /// Omitted products' page removals, sorted by name.
    pub removals: Vec<PlannedRemoval>,
    /// The structured export exactly as it is written.
    pub export: Vec<u8>,
    /// Path of the structured export.
    pub export_path: PathBuf,
    /// The export bytes already on disk, when there is a file.
    pub export_previous: Option<Vec<u8>>,
    /// Products omitted for want of a backing document.
    pub omitted: Vec<Omitted>,
}

/// The expected publication tree, computed without writing a byte.
///
/// Every declaration is resolved, every include checked against its admission
/// row, every page rendered, and the export encoded before any target is
/// touched, so an invalid declaration or an encode failure refuses the whole
/// tree. The plan carries, per target, the bytes already on disk and the bytes
/// the build would leave there — which is exactly what `kaya check` compares.
/// `read_text` and `read_bytes` supply the existing tree; `kaya check` passes
/// closures that report nothing, so the plan is the canonical expected tree
/// rather than an edit of whatever happens to exist.
pub fn plan(
    decls: &[Decl],
    docs: &Documents,
    out: &Path,
    mut read_text: impl FnMut(&Path) -> Result<Option<String>, PublicationError>,
    mut read_bytes: impl FnMut(&Path) -> Result<Option<Vec<u8>>, PublicationError>,
) -> Result<Plan, PublicationError> {
    check_source_keys(docs)?;
    // Resolve every declaration against its closed vocabulary before touching
    // the output directory. A refusal must leave the tree byte-untouched.
    let (products, packages) = partition_declarations(decls)?;
    check_admission(&products)?;
    check_package_members(&packages, &products)?;
    let (composed, omitted) = compose_products(&products, docs);
    let ownership = assign_owners(&composed);
    // One rendering per fact: later products cross-reference the first holder.
    let rendered = render_pages(&products, &packages, &composed, &ownership, docs);

    // Phase one computes every byte this build publishes before any write.
    let pages = plan_pages(rendered, &composed, out, &mut read_text)?;
    let export = export_json(&products, &packages, &composed, &ownership.emitted, docs);
    let (export, export_path, export_previous) = plan_export(export, out, &mut read_bytes)?;

    // Phase two plans the pages an omitted product would have occupied: a
    // document the build never wrote is not ours and is left untouched, while
    // a document carrying the page's managed block is stripped or unlinked.
    let removals = plan_removals(&omitted, out, &mut read_text)?;

    Ok(Plan {
        pages,
        removals,
        export,
        export_path,
        export_previous,
        omitted,
    })
}

/// Refuse a document filed under a key it does not name itself: the source
/// map is keyed by kind, and relabeling would silently falsify attribution.
fn check_source_keys(docs: &Documents) -> Result<(), PublicationError> {
    for (key, document) in &docs.0 {
        if &document.kind != key {
            return Err(PublicationError::document_kind(key, &document.kind));
        }
    }
    Ok(())
}

/// Resolve every declaration against its closed vocabulary, refusing tier
/// mismatches, duplicates, stray fields, and unimplemented collections.
fn partition_declarations(decls: &[Decl]) -> Result<(Vec<&Decl>, Vec<&Decl>), PublicationError> {
    let mut products: Vec<&Decl> = Vec::new();
    let mut packages: Vec<&Decl> = Vec::new();
    let mut declared: BTreeSet<&str> = BTreeSet::new();
    for decl in decls {
        let (tier, _) = resolve_name(&decl.name)?;
        if tier != decl.tier {
            return Err(PublicationError::tier_mismatch(&decl.name, decl.tier, tier));
        }
        if !declared.insert(decl.name.as_str()) {
            return Err(PublicationError::duplicate_declaration(&decl.name));
        }
        match tier {
            Tier::Product => {
                if !decl.members.is_empty() {
                    return Err(PublicationError::tier_fields(&decl.name, tier, "members"));
                }
                products.push(decl);
            }
            Tier::Package => {
                if !decl.includes.is_empty() {
                    return Err(PublicationError::tier_fields(&decl.name, tier, "includes"));
                }
                packages.push(decl);
            }
            // Collections resolve, but no collection renderer exists yet. A
            // declaration is refused rather than silently dropped, because a
            // dropped declaration reads exactly like an emitted one.
            Tier::Collection => return Err(PublicationError::tier_unimplemented(&decl.name)),
        }
    }
    Ok((products, packages))
}

/// Check every declared include against its product's admission row, present
/// or not, so an invalid composition can never slip through by accident.
fn check_admission(products: &[&Decl]) -> Result<(), PublicationError> {
    for product in products {
        let row = super::vocabulary::admission(&product.name);
        for kind in &product.includes {
            if !row.contains(&kind.as_str()) {
                return Err(PublicationError::inadmissible(&product.name, kind, row));
            }
        }
    }
    Ok(())
}

/// Check every package member: a package lists declared products, nothing else.
fn check_package_members(packages: &[&Decl], products: &[&Decl]) -> Result<(), PublicationError> {
    let declared_products: BTreeSet<&str> =
        products.iter().map(|decl| decl.name.as_str()).collect();
    for package in packages {
        for member in &package.members {
            if !PRODUCTS.contains(&member.as_str()) || !declared_products.contains(member.as_str())
            {
                return Err(PublicationError::package_member(&package.name, member));
            }
        }
    }
    Ok(())
}

/// Phase one for pages: compute every page document against the existing
/// tree, refusing an out-of-vocabulary block before anything is written.
fn plan_pages(
    rendered: Vec<(String, Tier, String)>,
    composed: &std::collections::BTreeMap<String, Vec<String>>,
    out: &Path,
    read_text: &mut impl FnMut(&Path) -> Result<Option<String>, PublicationError>,
) -> Result<Vec<PlannedPage>, PublicationError> {
    let mut planned: Vec<PlannedPage> = Vec::new();
    for (name, tier, content) in rendered {
        let block = block_name(tier, &name);
        let path = out.join(format!("{name}.md"));
        let previous = read_text(&path)?;
        // One managed block per page, named inside the publication vocabulary:
        // a stale block from an earlier scheme would otherwise survive beside
        // the one this build writes. Bytes outside the block are the human's
        // and survive untouched; the block always holds exactly one rendering.
        if let Some(existing) = previous.as_deref() {
            validate_page_blocks(&path, existing)?;
        }
        let document = replace_generated(previous.as_deref().unwrap_or(""), &block, &content)
            .map_err(|error| compose_error(&path, error))?;
        planned.push(PlannedPage {
            page: Page {
                name: name.clone(),
                path: path.clone(),
                question: match tier {
                    Tier::Product => reader_question(&name),
                    _ => None,
                },
                documents: composed.get(&name).cloned().unwrap_or_default(),
            },
            block,
            path,
            document,
            previous,
        });
    }
    Ok(planned)
}

/// The export bytes, their path, and the bytes already on disk.
type PlannedExport = (Vec<u8>, PathBuf, Option<Vec<u8>>);

/// Phase one for the export: encode the structured export and read the bytes
/// already on disk.
fn plan_export(
    export: serde_json::Value,
    out: &Path,
    read_bytes: &mut impl FnMut(&Path) -> Result<Option<Vec<u8>>, PublicationError>,
) -> Result<PlannedExport, PublicationError> {
    let mut bytes = serde_json::to_vec_pretty(&export).map_err(|error| {
        PublicationError::new(LABEL_IO, format!("failed to encode export: {error}"))
    })?;
    bytes.push(b'\n');
    let export_path = out.join(EXPORT_FILE);
    let export_previous = read_bytes(&export_path)?;
    Ok((bytes, export_path, export_previous))
}

/// Phase two: plan the removal of every omitted product's page.
fn plan_removals(
    omitted: &[Omitted],
    out: &Path,
    read_text: &mut impl FnMut(&Path) -> Result<Option<String>, PublicationError>,
) -> Result<Vec<PlannedRemoval>, PublicationError> {
    let mut removals = Vec::new();
    for entry in omitted {
        removals.push(plan_removal(out, Tier::Product, &entry.name, read_text)?);
    }
    Ok(removals)
}

/// Plan the removal of an omitted publication's page without destroying authored bytes.
///
/// A document that does not carry this page's managed block is not ours: a
/// file the build never wrote at that path is authored, even when its name
/// matches a now-omitted product, so its bytes are planned unchanged. A
/// document carrying the block is planned unlinked when the block is all it
/// holds and otherwise keeps every byte outside the block.
fn plan_removal(
    out: &Path,
    tier: Tier,
    name: &str,
    read_text: &mut impl FnMut(&Path) -> Result<Option<String>, PublicationError>,
) -> Result<PlannedRemoval, PublicationError> {
    let path = out.join(format!("{name}.md"));
    let block = block_name(tier, name);
    let previous = read_text(&path)?;
    let Some(document) = previous.as_deref() else {
        return Ok(PlannedRemoval {
            path,
            previous: None,
            document: None,
        });
    };
    let stripped = strip_managed_block(&path, document, &block)?;
    let document = match stripped {
        // The document does not carry the block, so it is not ours: its
        // bytes are planned unchanged.
        None => Some(document.to_owned()),
        Some(stripped) if stripped.trim().is_empty() => None,
        Some(stripped) => Some(stripped),
    };
    Ok(PlannedRemoval {
        path,
        previous,
        document,
    })
}

/// Strip one managed block from a document, or plan it unchanged when the
/// document does not carry the block. A page about to lose its managed block
/// has any other block it carries named before the write-back.
fn strip_managed_block(
    path: &Path,
    document: &str,
    block: &str,
) -> Result<Option<String>, PublicationError> {
    validate_page_blocks(path, document)?;
    let carries_block = scan(document)
        .map_err(|error| compose_error(path, error))?
        .iter()
        .any(|found| matches!(&found.kind, BlockKind::Generated(existing) if existing == block));
    if !carries_block {
        return Ok(None);
    }
    Ok(Some(
        remove_generated(document, block).map_err(|error| compose_error(path, error))?,
    ))
}

/// The managed-block name a page's content belongs to.
pub(super) fn block_name(tier: Tier, name: &str) -> String {
    format!("publication.{}.{name}", tier.as_str())
}

/// Whether a managed block name is inside the publication block vocabulary.
///
/// A publication page carries one `publication.<tier>.<name>` block, with the
/// tier and the name from that tier's closed vocabulary. A name outside it —
/// a misspelled tier, a name from another tier, or a document-vocabulary kind —
/// is refused rather than left in the page beside the block the build writes.
fn is_publication_block(name: &str) -> bool {
    let Some(rest) = name.strip_prefix("publication.") else {
        return false;
    };
    let Some((tier, entry)) = rest.split_once('.') else {
        return false;
    };
    let Some(resolved) = tier_for_name(tier) else {
        return false;
    };
    match resolve_name(entry) {
        Ok((found, _)) => found == resolved,
        Err(_) => false,
    }
}

/// Refuse a publication page carrying a managed block outside the vocabulary.
pub(super) fn validate_page_blocks(file: &Path, document: &str) -> Result<(), PublicationError> {
    let blocks = scan(document).map_err(|error| compose_error(file, error))?;
    for block in blocks {
        let BlockKind::Generated(name) = block.kind else {
            continue;
        };
        if !is_publication_block(&name) {
            return Err(PublicationError::block_vocabulary(file, &name));
        }
    }
    Ok(())
}
