---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-qp8-publications-tracer, pipeline-step:orient]
---

# ORIENT kaya-qp8 — publications tracer bullet

Ticket: kaya-qp8, P2, lane:afk, complexity:l, base_commit df81656,
files: src/publication/mod.rs, src/main.rs.
Spec: openspec/changes/add-document-taxonomy/specs/kaya-publications/spec.md
(Publication tiers, Composition without duplication, Justified emission,
Reader-question binding, One source many formats) and layer.md
(properties tier_resolved, fact_once, stub_free, inadmissible_refused,
format_attribution).

## Acceptance as testable statements

1. Three closed vocabularies (12 products, 9 collections, 8 packages) resolve
   a name to exactly one tier; an unknown tier or name is refused with a label
   naming the value and listing the vocabularies.
2. A package lists products (links), never document families; listing
   exploration.experiment-record is refused naming the reference and the
   package tier, and its generated block contains member names/links only.
3. A product composes document kinds from its admission row; an inadmissible
   kind is refused with a label naming product, kind and row.
4. A product with no admissible backing document emits no page and is reported
   omitted with reason no-backing-document; a package names it unavailable.
5. Each emitted product names its reader question; each fact body appears once
   in the tree, later products cross-reference it.
6. Markdown pages and the structured export derive from one source (same event
   ids per fact); two builds from unchanged inputs are byte-identical.
7. Demoable outcome: project-site over overview and reference. With the
   committed ledger, reference backs on orientation.research-map (and later
   reuse-handoff.technical-reference); overview has no backing document yet, so
   it is omitted and marked unavailable.

## Current code seams

- src/projection/*: Family/Kind vocabulary, Summary, validate_generated_blocks,
  scan/replace_generated/remove_generated, shared plumbing
  (generated_body/read_existing/write/io_error).
- Compiled documents: docs/research/index.md (orientation.research-map),
  docs/research/reproduction-package.md (reuse-handoff.reproduction-package),
  docs/research/experiments/*.md (exploration.experiment-specification/record).
  A reader must extract generated block bodies keyed by family.kind.
- src/main.rs: clap command enum (init/status/import/build/promote/challenge);
  a new `publish` command fits the existing envelope pattern.
- docs/src/document-taxonomy.md documents the publication layer (tier and
  product tables) — the status paragraph will need a one-line update.

## Validation

narrow: cargo test --test publications --test cli_pipeline
full: cargo test; ah check --changes add-document-taxonomy --run-tests; openspec validate --changes --strict
baseline: HEAD 5eb26f0, 199 tests green, fmt/clippy clean, build idempotent.

