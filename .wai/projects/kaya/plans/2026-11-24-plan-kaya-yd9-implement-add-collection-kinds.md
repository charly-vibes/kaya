---
tags: [pipeline-run:tdd-ro5u-2026-11-24-kaya-yd9-implement-add-collection-kinds, pipeline-step:plan]
---

# Plan: kaya-yd9 — implement add-collection-kinds

One pipeline run; red → green → refactor → ro5u → fix → ledger. Contract:
openspec/changes/add-collection-kinds tasks.md §1 (17 scenarios) + specs
delta. No ledger reads; renderers pin their compiled sources.

## Red (failing tests first)

New/changed tests in `tests/publications/build.rs` (fixture: hand-built
`Documents` with synthetic block bodies):

1. `feature_index_declaration_refuses_naming_the_implemented_kinds` —
   narrows the existing `topic-map` pin (build.rs:254-282): refused kind
   `feature-index`; message names it and all five implemented kinds; tree
   byte-identical (`assert_no_writes`).
2. `topic_map_renders_one_row_per_topic_with_entity_and_relationship_types`
   — research-map fixture (3 edges, 1 isolated) ⇒ rows per distinct id with
   type + participated relationship types; block `publication.collection.topic-map`.
3. `topic_map_cross_references_the_page_that_embeds_the_research_map` —
   `assign_owners` maps the kind to a product ⇒ topic-map page references
   `architecture-design.md#orientationresearch-map`, embeds no edge lines.
4. `topic_map_embeds_edges_when_no_page_composes_the_research_map` — no
   owner ⇒ edge statements embedded verbatim on the topic-map page.
5. `knowledge_index_renders_the_per_type_census` — ids across blocks ⇒
   section per closed type with count + listed ids; empty types report
   none; `kaya/NotAnEntity/…` ids excluded.
6. `example_gallery_renders_rows_from_experiment_records` — record block
   naming run + artifact + observation ⇒ one row carrying them; second test
   `gallery_without_record_rows_is_omitted` ⇒ `no-backing-document`, no page.
7. `api_catalogue_renders_one_row_per_spec_id` — technical-reference fixture
   ⇒ rows per spec id with reference; `catalogue_without_spec_rows_is_omitted`.
8. `export_collection_rows_carry_emission_and_reason` (export.rs) — emitted
   glossary row unchanged keys; omitted collection row gains `"reason"`.
9. declarations.rs — default collections are exactly the five implemented
   kinds (task 1.13 pin).
10. cli_pipeline.rs pin flips: pages list += `knowledge-index`, `topic-map`
    (3 sites + 2645 variant); export collections += 5 rows (2 emitted, 2
    omitted-with-reason, glossary); block lists += `publication.collection.topic-map`
    + `publication.collection.knowledge-index`; omitted rows += collection rows.

## Green (src, smallest faithful change)

- `plan.rs` — gate → dispatch: `const IMPLEMENTED_COLLECTIONS: [&str; 5]`;
  membership admits, else `tier_unimplemented` message names declared kind +
  implemented set. Compose phase: `compose_collections(collections, docs,
  ownership)` → per-kind row computation; no rows ⇒ `Omitted { tier:
  Tier::Collection.as_str(), name, reason }` appended to `omitted`;
  renderable collections flow to `render_pages`/`export_json` (which learn
  which collections emitted + the reason map). `plan_removals` unchanged
  (takes `&[Omitted]`; collection rows flow through).
- `render.rs` — `render_collection_page` loses the debug_assert; match on
  name: glossary (verbatim today), topic-map (parse research-map edges +
  isolated; rows per id; embed edges iff `owners` lacks
  `orientation.research-map`, else cross-reference), knowledge-index (census
  over all `DocumentRef.event_ids` grouped by id type segment under
  `EntityType::ALL`), example-gallery (per record-carrying document: runs /
  artifacts / observations the record names via `node_ids` + type segment),
  api-catalogue (per `- <node id> — <spec id>` row of the technical-reference
  block). Managed block `publication.collection.<name>`; no reader question.
- `render.rs` `export_json` — collection rows `{"emitted": bool, "name"}`,
  plus `"reason"` when omitted.
- `source.rs` — read list += `technical-reference.md`; `verification/mod.rs`
  `RESEARCH_READ_ORDER` += same (one change, both doors).
- `decls.rs` — `DEFAULT_COLLECTIONS = ["glossary", "topic-map",
  "knowledge-index", "example-gallery", "api-catalogue"]`.

## Refactor

Tidy parser helpers into one place (single `parse type segment` helper;
section-line scanners shared by gallery/catalogue/topic-map); no behavior
change; rustfmt.

## Acceptance

- `cargo test` green; `cargo clippy --all-targets -- -D warnings` clean.
- Dogfood (task 1.17): `target/debug/kaya build` + `publish --dry-run` /
  publish regenerate `topic-map.md` + `knowledge-index.md`; export rows carry
  the five; `kaya check` clean; cli pins match reality.
