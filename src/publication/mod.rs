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

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde_json::json;

use crate::projection::{
    BlockKind, ProjectionError, Summary, remove_generated, replace_generated, scan,
};

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

/// The closed document vocabulary an admission row may name.
const DOCUMENT_KINDS: [&str; 11] = [
    "orientation.project-overview",
    "orientation.research-map",
    "exploration.exploration-brief",
    "exploration.experiment-specification",
    "exploration.experiment-record",
    "understanding.findings-and-analysis",
    "understanding.decision-record",
    "reuse-handoff.technical-reference",
    "reuse-handoff.reproduction-package",
    "reuse-handoff.knowledge-brief",
    "reuse-handoff.handoff",
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

/// A labeled publication refusal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicationError {
    /// Failure class, for example `kaya.publications.tier_resolved`.
    pub label: String,
    /// Human-readable explanation naming the offending value.
    pub message: String,
}

impl PublicationError {
    fn new(label: &str, message: String) -> Self {
        Self {
            label: label.to_owned(),
            message,
        }
    }

    fn io(action: &str, path: &Path, error: io::Error) -> Self {
        Self::new(
            LABEL_IO,
            format!("failed to {action} \"{}\": {error}", path.display()),
        )
    }

    fn unknown_tier(value: &str) -> Self {
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

    fn tier_mismatch(value: &str, declared: Tier, resolved: Tier) -> Self {
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

    fn inadmissible(product: &str, kind: &str, row: &[&'static str]) -> Self {
        Self::new(
            LABEL_INADMISSIBLE,
            format!(
                "product \"{product}\" cannot include \"{kind}\": the admission row is {}",
                row.join(", ")
            ),
        )
    }

    fn package_member(package: &str, member: &str) -> Self {
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

    fn duplicate_declaration(name: &str) -> Self {
        Self::new(
            LABEL_DUPLICATE_DECLARATION,
            format!(
                "publication \"{name}\" is declared more than once: a name resolves to one \
                 tier and is declared once"
            ),
        )
    }

    fn tier_fields(name: &str, tier: Tier, field: &str) -> Self {
        Self::new(
            LABEL_TIER_FIELDS,
            format!(
                "publication \"{name}\" declares the {} tier but carries \"{field}\": a \
                 product lists \"includes\" and a package lists \"members\"",
                tier.as_str()
            ),
        )
    }

    fn tier_unimplemented(name: &str) -> Self {
        Self::new(
            LABEL_TIER_UNIMPLEMENTED,
            format!(
                "collection \"{name}\" cannot be declared: collections are not implemented \
                 yet, so this release composes products and packages only"
            ),
        )
    }

    fn document_kind(key: &str, kind: &str) -> Self {
        Self::new(
            LABEL_DOCUMENT_KIND,
            format!(
                "document source key \"{key}\" holds a document of kind \"{kind}\": the map \
                 key and the document's kind must name the same document"
            ),
        )
    }

    fn duplicate_block(file: &Path, block: &str) -> Self {
        Self::new(
            LABEL_DUPLICATE_BLOCK,
            format!(
                "failed to read \"{}\": generated block \"{block}\" appears more than once \
                 in one compiled document",
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

/// The document kinds admissible for a product.
///
/// Naming an unknown product yields an empty row, so every inclusion is
/// refused rather than silently admitted.
pub fn admission(product: &str) -> &'static [&'static str] {
    match product {
        "overview" => &["orientation.project-overview"],
        "getting-started" => &[
            "orientation.project-overview",
            "reuse-handoff.reproduction-package",
        ],
        "tutorial" => &[
            "exploration.experiment-record",
            "exploration.exploration-brief",
        ],
        "how-to" => &[
            "reuse-handoff.technical-reference",
            "reuse-handoff.reproduction-package",
        ],
        "concept" => &[
            "understanding.decision-record",
            "understanding.findings-and-analysis",
        ],
        "reference" => &[
            "reuse-handoff.technical-reference",
            "orientation.research-map",
        ],
        "example" => &["exploration.experiment-record"],
        "troubleshooting" => &["understanding.findings-and-analysis"],
        "faq" => &[
            "understanding.findings-and-analysis",
            "understanding.decision-record",
        ],
        "status-compatibility" => &[
            "orientation.project-overview",
            "exploration.experiment-record",
        ],
        "release-migration" => &[
            "orientation.project-overview",
            "understanding.decision-record",
        ],
        "architecture-design" => &["understanding.decision-record", "orientation.research-map"],
        _ => &[],
    }
}

/// One compiled document: its kind, its authored/generated body, and the event
/// ids the body's node ids resolve through.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentRef {
    /// Generated block name, `<family>.<kind>`.
    pub kind: String,
    /// The block body exactly as compiled.
    pub body: String,
    /// Sorted, deduplicated `kaya/...` node ids named in the body.
    pub event_ids: Vec<String>,
}

/// The document source keyed by generated block name (the document kind).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Documents(pub BTreeMap<String, DocumentRef>);

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
    pub path: PathBuf,
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

/// Read the compiled documents under `research_dir`.
///
/// Files are read in a fixed order — `index.md`, `overview.md`,
/// `understanding.md`, `reproduction-package.md`, `handoff.md`, then the
/// experiment dossiers sorted by name — and only generated blocks whose
/// names are in the document vocabulary are kept. A repeated block name
/// inside one file is refused, because the file's grammar then admits two
/// answers; across files the first populated block wins. Whitespace-only
/// blocks are skipped, so an empty compilation never shadows a populated one.
/// The block body's `kaya/...` node ids become its event attribution. No
/// ledger access is needed.
pub fn read_documents(research_dir: &Path) -> Result<Documents, PublicationError> {
    let mut documents = BTreeMap::new();

    let mut files: Vec<PathBuf> = Vec::new();
    for name in [
        "index.md",
        "overview.md",
        "understanding.md",
        "reproduction-package.md",
        "handoff.md",
    ] {
        files.push(research_dir.join(name));
    }
    let experiments = research_dir.join("experiments");
    match fs::read_dir(&experiments) {
        Ok(entries) => {
            let mut paths: Vec<PathBuf> = Vec::new();
            for entry in entries {
                let entry =
                    entry.map_err(|error| PublicationError::io("read", &experiments, error))?;
                let path = entry.path();
                if path.extension().is_some_and(|extension| extension == "md") {
                    paths.push(path);
                }
            }
            paths.sort();
            files.extend(paths);
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(PublicationError::io("read", &experiments, error)),
    }

    for path in files {
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(PublicationError::io("read", &path, error)),
        };
        let blocks = scan(&text).map_err(|error| {
            PublicationError::new(
                LABEL_IO,
                format!(
                    "failed to parse \"{}\": {}: {}",
                    path.display(),
                    error.label,
                    error.message
                ),
            )
        })?;
        let mut seen: BTreeSet<String> = BTreeSet::new();
        for block in blocks {
            let BlockKind::Generated(name) = block.kind else {
                continue;
            };
            if !seen.insert(name.clone()) {
                return Err(PublicationError::duplicate_block(&path, &name));
            }
            if !DOCUMENT_KINDS.contains(&name.as_str()) {
                continue;
            }
            // An empty block is no more a document than a missing one: it must
            // neither back a product nor shadow a populated block later in the
            // read order.
            if block.body.trim().is_empty() {
                continue;
            }
            if documents.contains_key(&name) {
                continue;
            }
            let event_ids = node_ids(&block.body);
            documents.insert(
                name.clone(),
                DocumentRef {
                    kind: name,
                    body: block.body,
                    event_ids,
                },
            );
        }
    }

    Ok(Documents(documents))
}

/// The `kaya/...` node ids named in a block body, sorted and deduplicated.
///
/// `@` stays inside a token because a `Source` reference carries its commit
/// suffix: `kaya/Source/docs-src-index.md@43b606f` is one node id, not the
/// path plus a trailing fragment.
fn node_ids(body: &str) -> Vec<String> {
    let ids: BTreeSet<String> = body
        .split(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '/' | '-' | '_' | '.' | '@')))
        .filter(|token| token.starts_with("kaya/"))
        .map(str::to_owned)
        .collect();
    ids.into_iter().collect()
}

/// Build a publication tree: refuse invalid declarations before any write,
/// emit backed products and every declared package, and write pages plus the
/// structured export only when their bytes change.
///
/// Every page is written as one `publication.<tier>.<name>` managed block
/// inside whatever document already exists at that path, so authored bytes
/// outside the block survive and only the block is compiler-owned.
pub fn build(decls: &[Decl], docs: &Documents, out: &Path) -> Result<Built, PublicationError> {
    // The source map is a map from kind to document; a document filed under a
    // key it does not name itself is a refusal, never a silent relabeling.
    for (key, document) in &docs.0 {
        if &document.kind != key {
            return Err(PublicationError::document_kind(key, &document.kind));
        }
    }

    // Resolve every declaration against its closed vocabulary before touching
    // the output directory. A refusal must leave the tree byte-untouched.
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

    // The admission table is checked for every declared include, present or
    // not, so an invalid composition can never slip through by accident.
    for product in &products {
        let row = admission(&product.name);
        for kind in &product.includes {
            if !row.contains(&kind.as_str()) {
                return Err(PublicationError::inadmissible(&product.name, kind, row));
            }
        }
    }

    // A package lists declared products and nothing else.
    let declared_products: BTreeSet<&str> =
        products.iter().map(|decl| decl.name.as_str()).collect();
    for package in &packages {
        for member in &package.members {
            if !PRODUCTS.contains(&member.as_str()) || !declared_products.contains(member.as_str())
            {
                return Err(PublicationError::package_member(&package.name, member));
            }
        }
    }

    // Emission: a product is emitted iff at least one declared include is a
    // compiled document. Composed documents keep declaration order.
    let mut composed: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut omitted: Vec<Omitted> = Vec::new();
    for product in &products {
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

    // One rendering per fact: the first emitted product in page order carries
    // the body; every later product cross-references it.
    let mut owners: BTreeMap<String, String> = BTreeMap::new();
    let mut emitted_names: BTreeSet<String> = BTreeSet::new();
    for name in composed.keys() {
        emitted_names.insert(name.clone());
        for kind in &composed[name] {
            owners.entry(kind.clone()).or_insert_with(|| name.clone());
        }
    }

    let mut rendered: Vec<(String, Tier, String)> = Vec::new();
    for product in &products {
        let Some(kinds) = composed.get(&product.name) else {
            continue;
        };
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
        rendered.push((product.name.clone(), Tier::Product, page));
    }

    for package in &packages {
        let mut page = format!("## {}\n\n", display_name(&package.name));
        for member in &package.members {
            if emitted_names.contains(member) {
                page.push_str(&format!("- [{member}]({member}.md)\n"));
            } else {
                page.push_str(&format!("- {member} — unavailable\n"));
            }
        }
        rendered.push((package.name.clone(), Tier::Package, page));
    }

    rendered.sort_by(|left, right| left.0.cmp(&right.0));

    // Phase one: compute every byte this build publishes. Reading the existing
    // tree is safe here, and nothing is written until every page document and
    // the export are in hand, so an encode failure cannot desynchronise the
    // Markdown pages from the structured export.
    let mut planned: Vec<Planned> = Vec::new();
    let mut pages: Vec<Page> = Vec::new();
    for (name, tier, content) in rendered {
        let block = block_name(tier, &name);
        let path = out.join(format!("{name}.md"));
        let previous = read_text(&path)?;
        // A page is a managed block inside whatever document already lives at
        // its path: bytes outside the block are the human's and survive
        // untouched, while the block always holds exactly one rendering.
        let document = replace_generated(previous.as_deref().unwrap_or(""), &block, &content)
            .map_err(|error| compose_error(&path, error))?;
        planned.push(Planned {
            block,
            path: path.clone(),
            document,
            previous,
        });
        pages.push(Page {
            name: name.clone(),
            path,
            question: match tier {
                Tier::Product => reader_question(&name),
                _ => None,
            },
            documents: composed.get(&name).cloned().unwrap_or_default(),
        });
    }

    let export = export_json(&products, &packages, &composed, &emitted_names, docs);
    let mut bytes = serde_json::to_vec_pretty(&export).map_err(|error| {
        PublicationError::new(LABEL_IO, format!("failed to encode export: {error}"))
    })?;
    bytes.push(b'\n');
    let export_path = out.join(EXPORT_FILE);
    let export_previous = read_bytes(&export_path)?;

    // Phase two: write each changed target through a sibling temporary file
    // and rename it into place. Unchanged targets are not touched at all, so a
    // second build over the same inputs rewrites nothing.
    fs::create_dir_all(out).map_err(|error| PublicationError::io("create", out, error))?;

    let mut summary = Summary::default();
    for page in planned {
        if page.previous.as_deref() == Some(page.document.as_str()) {
            summary.unchanged.push(page.block);
        } else {
            write_atomic(&page.path, page.document.as_bytes())?;
            summary.recomputed.push(page.block);
        }
    }
    if export_previous.as_deref() == Some(bytes.as_slice()) {
        summary.unchanged.push(EXPORT_FILE.to_owned());
    } else {
        write_atomic(&export_path, &bytes)?;
        summary.recomputed.push(EXPORT_FILE.to_owned());
    }

    // Only after every page and the export are in place are pages for omitted
    // products removed, so a failed write never leaves the tree missing a page
    // it failed to replace.
    for entry in &omitted {
        remove_page(out, Tier::Product, &entry.name)?;
    }

    Ok(Built {
        pages,
        omitted,
        summary,
    })
}

/// The export file written beside the published pages.
const EXPORT_FILE: &str = "publication.json";

/// One page's planned document, before the write phase.
struct Planned {
    /// The managed block the page's content belongs to.
    block: String,
    /// Final path of the page.
    path: PathBuf,
    /// The whole document to write, block included.
    document: String,
    /// The document already on disk, when there is one.
    previous: Option<String>,
}

/// The managed-block name a page's content belongs to.
fn block_name(tier: Tier, name: &str) -> String {
    format!("publication.{}.{name}", tier.as_str())
}

/// The heading anchor a document kind is listed under on its holder's page.
///
/// The page lists each kind as an `### <kind>` heading, whose anchor keeps only
/// alphanumerics, hyphens and underscores — turning each space into a hyphen
/// and dropping every other character — so `orientation.project-overview`
/// anchors as `orientationproject-overview`.
fn heading_anchor(kind: &str) -> String {
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

/// Read a page, treating an absent file as an absent document.
fn read_text(path: &Path) -> Result<Option<String>, PublicationError> {
    match fs::read_to_string(path) {
        Ok(document) => Ok(Some(document)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(PublicationError::io("read", path, error)),
    }
}

/// Read a target byte-for-byte, treating an absent file as absent bytes.
fn read_bytes(path: &Path) -> Result<Option<Vec<u8>>, PublicationError> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(PublicationError::io("read", path, error)),
    }
}

/// Write `bytes` beside `path` and rename them into place.
///
/// The rename is the only visible step: a reader of the tree sees the previous
/// bytes or the complete new ones, never a truncated file.
fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), PublicationError> {
    let mut temporary = path.as_os_str().to_owned();
    temporary.push(".tmp");
    let temporary = PathBuf::from(temporary);
    fs::write(&temporary, bytes)
        .map_err(|error| PublicationError::io("write", &temporary, error))?;
    fs::rename(&temporary, path).map_err(|error| {
        PublicationError::new(
            LABEL_IO,
            format!(
                "failed to rename \"{}\" to \"{}\": {error}",
                temporary.display(),
                path.display()
            ),
        )
    })
}

/// Remove the page of an omitted publication without destroying authored bytes.
///
/// A document that does not carry this page's managed block is not ours: a
/// file the build never wrote at that path is authored, even when its name
/// matches a now-omitted product, so it is left byte-identical. A document
/// carrying the block is unlinked when the block is all it holds and otherwise
/// keeps every byte outside the block.
fn remove_page(out: &Path, tier: Tier, name: &str) -> Result<(), PublicationError> {
    let path = out.join(format!("{name}.md"));
    let Some(document) = read_text(&path)? else {
        return Ok(());
    };
    let block = block_name(tier, name);
    let carries_block = scan(&document)
        .map_err(|error| compose_error(&path, error))?
        .iter()
        .any(|found| matches!(&found.kind, BlockKind::Generated(existing) if existing == &block));
    if !carries_block {
        return Ok(());
    }
    let stripped =
        remove_generated(&document, &block).map_err(|error| compose_error(&path, error))?;
    if stripped.trim().is_empty() {
        fs::remove_file(&path).map_err(|error| PublicationError::io("remove", &path, error))?;
    } else {
        write_atomic(&path, stripped.as_bytes())?;
    }
    Ok(())
}

/// Render a managed-block refusal as a publication refusal naming the file.
fn compose_error(path: &Path, error: ProjectionError) -> PublicationError {
    PublicationError::new(
        LABEL_IO,
        format!(
            "failed to manage \"{}\": {}: {}",
            path.display(),
            error.label,
            error.message
        ),
    )
}

/// The structured export, projected from the same source map as the pages.
fn export_json(
    products: &[&Decl],
    packages: &[&Decl],
    composed: &BTreeMap<String, Vec<String>>,
    emitted: &BTreeSet<String>,
    docs: &Documents,
) -> serde_json::Value {
    let mut product_rows: Vec<(String, serde_json::Value)> = products
        .iter()
        .map(|product| {
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
        })
        .collect();
    product_rows.sort_by(|left, right| left.0.cmp(&right.0));

    let mut package_rows: Vec<(String, serde_json::Value)> = packages
        .iter()
        .map(|package| {
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
        })
        .collect();
    package_rows.sort_by(|left, right| left.0.cmp(&right.0));

    json!({
        "products": product_rows
            .into_iter()
            .map(|(_, row)| row)
            .collect::<Vec<_>>(),
        "packages": package_rows
            .into_iter()
            .map(|(_, row)| row)
            .collect::<Vec<_>>(),
    })
}

/// Capitalize the first letter of a page name for its heading.
fn display_name(name: &str) -> String {
    let mut chars = name.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}
