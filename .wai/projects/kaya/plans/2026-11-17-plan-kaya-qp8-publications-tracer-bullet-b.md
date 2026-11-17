---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-qp8-publications-tracer, pipeline-step:plan]
---

# PLAN kaya-qp8 — publications tracer bullet

## Behavior (spec ids)

- kaya.publications/Publication tiers: closed product/collection/package
  vocabularies; name -> one tier; unknown -> labeled refusal with vocabularies.
- kaya.publications/Composition without duplication: products compose document
  kinds; packages reference products; each fact body appears once, later
  products cross-reference.
- kaya.publications/Justified emission: product emitted only with an admissible
  backing document; otherwise omitted (no-backing-document) and named
  unavailable in its package.
- kaya.publications/Reader-question binding: fixed question per product and the
  admission table; inadmissible inclusion refused naming product, kind, row.
- kaya.publications/One source many formats: markdown pages and a JSON export
  from one source map with the same event ids; two builds byte-identical.
- Bound scenarios: tier_resolved, fact_once, stub_free, inadmissible_refused.

## Out of scope

the remaining ten products and eight collections (kaya-duv, kaya-scq);
technical-reference knowledge-brief adapters (kaya-lh1); editorial cutover
(kaya-n50); spec edits.

## Contract (binding)

src/publication/mod.rs (new; `pub mod publication;` in lib.rs)

```rust
pub const PRODUCTS: [&str; 12]; pub const COLLECTIONS: [&str; 9]; pub const PACKAGES: [&str; 8];
pub enum Tier { Product, Collection, Package }
pub struct PublicationError { pub label: String, pub message: String }
pub fn resolve_name(name: &str) -> Result<(Tier, &'static str), PublicationError>;
pub fn reader_question(product: &str) -> Option<&'static str>;
pub fn admission(product: &str) -> &'static [&'static str];      // document block names
pub struct DocumentRef { pub kind: String, pub body: String, pub event_ids: Vec<String> }
pub struct Documents(pub BTreeMap<String, DocumentRef>);
pub struct Decl { pub tier: Tier, pub name: String, pub includes: Vec<String>, pub members: Vec<String> }
pub struct Omitted { pub tier: &'static str, pub name: String, pub reason: &'static str }
pub struct Page { pub name: String, pub path: PathBuf, pub question: Option<&'static str>, pub documents: Vec<String> }
pub struct Built { pub pages: Vec<Page>, pub omitted: Vec<Omitted>, pub summary: Summary }
pub fn build(decls: &[Decl], docs: &Documents, out: &Path) -> Result<Built, PublicationError>;
pub fn read_documents(research_dir: &Path) -> Result<Documents, PublicationError>;
```

Rules:
- Labels: kaya.publications.tier_resolved (unknown tier/name; message quotes the
  value and lists products, collections, packages), kaya.publications.
  inadmissible_refused (product + kind + the admission row),
  kaya.publications.package_members (member is not a product: quotes the
  member and the package tier), kaya.publications.io for document IO.
- Emission: a product is emitted iff at least one of its declared includes is
  present in `docs` and admissible; otherwise omitted with reason
  "no-backing-document" and no page file.
- Admission: every declared include must be in `admission(product)`, else
  inadmissible_refused before any write.
- Rendering (deterministic; pages sorted by name):
  product page block `publication.product.<name>`:
    `## <Name>` / blank / `Reader question: <question>` / blank / per included
    document in declaration order: `### <kind>` / blank / body (first product
    rendering that kind) OR `See [<first product>](<first>.md#<kind>).` /
    blank.
  package page block `publication.package.<name>`:
    `## <Name>` / blank / one row per member in declaration order:
    `- [<member>](<member>.md)` when emitted, `- <member> — unavailable` when
    omitted. No product body text.
- Export: `<out>/publication.json` with products (name, question, documents
  [{kind, event_ids}]) and packages (name, members [{name, emitted}]) from the
  same source map; written only when changed.
- read_documents: parse markdown under `<research_dir>` (index.md,
  reproduction-package.md, handoff.md, experiments/*.md), extract every
  generated block whose name is in the document vocabulary, map kind -> body +
  event ids from the block body's node ids (no ledger access).
- CLI `kaya publish [--out docs/public]`: reads docs/research, declares
  project-site over [overview, reference]; overview includes
  [orientation.project-overview]; reference includes
  [reuse-handoff.technical-reference, orientation.research-map]; envelope keys
  out/pages/omitted/documents.

## Tests to write first

tests/publications.rs
1. tier_resolved: every product/collection/package name resolves; unknown
   "experiment-report" and an unknown tier name refused with vocabularies;
   question table complete for 12 products.
2. inadmissible_refused: faq including exploration.experiment-specification
   refused naming product, kind and row; no page written.
3. package_cannot_list_a_family: package declaring exploration.experiment-record
   refused naming it and the package tier.
4. fact_once: two products including the same kind -> body once, second page
   cross-references the first; package block contains links only.
5. stub_free: product with no backing document -> no file, omitted reason
   no-backing-document; package names it unavailable.
6. formats_agree: build twice byte-identical; publication.json event ids equal
   the page source-map attribution.
7. project_site_overview_reference: overview omitted + unavailable, reference
   page carries the map body, package links reference.
tests/cli_pipeline.rs
8. `publish_writes_project_site_and_reports_the_missing_overview`: after build,
   `kaya publish` writes docs/public/{reference.md,project-site.md}, no
   overview.md, envelope omitted names overview/no-backing-document, and the
   ledger is untouched.

Contracts: 4 tomls under .espectacular/changes/add-document-taxonomy/
kaya-publications/ bound one-to-one to tests 1,4,5,2.

## Commands

narrow: cargo test --test publications --test cli_pipeline
full: cargo test; ah check --changes add-document-taxonomy --run-tests; openspec validate --changes --strict

## Files touched

src/publication/mod.rs, src/lib.rs, src/main.rs, tests/publications.rs,
tests/cli_pipeline.rs, docs/public/* (generated), docs/src/document-taxonomy.md
(status line), .espectacular/.../kaya-publications/*.toml, tasks.md 3.6,
.beads/issues.jsonl.

## Commit shape

1. `feat(publication): compose project-site from overview and reference`
2. evidence commit.

