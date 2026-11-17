---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-duv-developer-products, pipeline-step:plan]
---

# PLAN kaya-duv — developer-guide products

## Behavior (spec ids)

- kaya.publications/Justified emission + Reader-question binding: the six
  developer products emit only from their admission rows, name their reader
  question, refuse inadmissible kinds, and omit themselves without backing.
- kaya.publications/Composition without duplication: shared facts render once,
  cross-referenced.
- Bound scenarios: inadmissible_refused, stub_free (already bound); no new
  contracts required (the scenario->test mapping exists).

## Out of scope

remaining products (kaya-scq); collections; adapters (kaya-lh1); specs.

## Contract (binding)

src/main.rs cmd_publish declares every product in PRODUCTS with
`includes = admission(product)`, plus package project-site over
[overview, reference] and package developer-guide over the six developer
products. No src/publication/mod.rs behavior change is expected beyond tests;
if a missing seam appears, keep the change minimal and inside that module.

Product -> admission row (verbatim):
getting-started: orientation.project-overview, reuse-handoff.reproduction-package
how-to: reuse-handoff.technical-reference, reuse-handoff.reproduction-package
concept: understanding.decision-record, understanding.findings-and-analysis
example: exploration.experiment-record
troubleshooting: understanding.findings-and-analysis
faq: understanding.findings-and-analysis, understanding.decision-record

## Tests to write first (tests/publications_developer.rs)

1. `developer_products_emit_only_from_admission_rows` — fixture Documents with
   one body per kind; each product page lists exactly its present admitted
   kinds; a page never contains a kind outside its row.
2. `inadmissible_developer_include_is_refused` — example including
   understanding.findings-and-analysis refused naming product/kind/row.
3. `developer_products_without_backing_are_omitted` — only
   understanding.findings-and-analysis present: concept/faq/troubleshooting
   emit; getting-started/how-to/example omitted no-backing-document.
4. `shared_developer_fact_renders_once` — concept and faq both admit
   findings-and-analysis: body once (concept owns it), faq cross-references.
5. `developer_guide_package_links_emitted_products` — package lists six members,
   unavailable markers for omitted ones, no bodies.
tests/cli_pipeline.rs
6. `publish_declares_every_product_and_reports_justified_omissions` — publish on
   a workspace with a question only: pages overview + project-site +
   developer-guide; omitted includes reference/example/tutorial/how-to/concept/
   troubleshooting/faq (exact sorted array); no stub files.

## Commands

narrow: cargo test --test publications_developer --test publications --test cli_pipeline
full: cargo test; cargo run -- build; cargo run -- publish; ah check --changes add-document-taxonomy --run-tests; just ci

## Files touched

src/main.rs (declarations), tests/publications_developer.rs,
tests/cli_pipeline.rs, docs/public/* (regenerated), docs/src/document-taxonomy.md
status, tasks.md 3.7, .beads/issues.jsonl.

## Commit shape

1. `feat(publication): emit the developer-guide products`
2. evidence commit.

