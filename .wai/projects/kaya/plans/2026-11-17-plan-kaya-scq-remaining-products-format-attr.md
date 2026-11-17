---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-scq-status-release-architecture, pipeline-step:plan]
---

# PLAN kaya-scq — remaining products + format attribution

## Behavior (spec ids)

- kaya.publications/Reader-question binding: the three remaining products emit
  only from their admission rows.
- kaya.publications/One source, many formats: markdown pages and publication.json
  trace every fact to the same event ids; two runs byte-identical.
- Bound scenario: format_attribution.

## Out of scope

collections; adapters (kaya-lh1); tutorial (needs exploration-brief/record
producers — declared only when backed); specs.

## Contract (binding)

src/main.rs cmd_publish adds status-compatibility, release-migration and
architecture-design with includes = publication::admission(name). No publication
module change expected except a shared helper if id extraction needs one
(src/projection/mod.rs may export the node-id tokenizer used by publication).

## Tests to write first

tests/publications_export.rs
1. `remaining_products_emit_from_admission_rows` — documents for
   project-overview, experiment-record, decision-record, research-map; exact
   pages/bodies for the three products; no kind outside the row; omissions for
   unbacked products.
2. `format_attribution_matches_every_page_section` — for each emitted product
   document, the ids parsed from its page section equal the export document's
   event_ids; export and pages come from one build.
3. `repeat_publish_is_byte_identical` — two builds/publishes from unchanged
   inputs produce identical files and export.
tests/cli_pipeline.rs
4. `publish_declares_the_remaining_products_and_export_agrees` — question+source
   fixture: exact pages/omitted, publication.json documents equal page documents
   kind-by-kind, second publish unchanged.
Contract: format_attribution -> test 2.

## Commands

narrow: cargo test --test publications_export --test publications --test publications_developer --test cli_pipeline
full: cargo test; cargo run -- build; cargo run -- publish; ah check --changes add-document-taxonomy --run-tests; just ci

## Files touched

src/main.rs, tests/publications_export.rs, tests/cli_pipeline.rs,
docs/public/*, docs/src/{document-taxonomy,status}.md, tasks.md 3.8,
.beads/issues.jsonl.

## Commit shape

1. `feat(publication): emit the remaining products and pin format attribution`
2. evidence commit.

