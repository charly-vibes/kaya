# Tasks: add-collection-kinds

Design change: fix the per-kind composition contracts and the render-or-refuse
decision per collection kind. No production code; the implementation is the
follow-up ticket's.

## 1. Implementation-ticket work items (one per scenario in the kaya-publications delta)

The follow-up ticket implements these; each item names the scenario it must
verify. One dispatch replaces the glossary gate; every renderer reads only
the documents `read_documents` returned.

- [ ] 1.1 `topic_map_renders_from_the_research_map_block` — one row per
      distinct topic id in the research-map block, entity type and
      relationship types per row, block `publication.collection.topic-map`.
- [ ] 1.2 `topic_map_cross_references_embedded_edges` — when reference
      embeds the research-map body, the topic-map page references it for the
      edges and embeds none.
- [ ] 1.3 `topic_map_embeds_edges_when_no_page_does` — no page composes the
      research-map: the topic-map embeds the statements and is their only
      rendering.
- [ ] 1.4 `example_gallery_renders_from_experiment_records` — one row per
      recorded experiment with its run, artifacts, and observations.
- [ ] 1.5 `api_catalogue_renders_from_the_technical_reference` — one row per
      spec id with its resolvable reference; entry ids are knowledge-brief
      rows, not catalogue rows.
- [ ] 1.6 `knowledge_index_renders_the_per_type_census` — a section per
      closed entity type with a count, node ids the tree names listed, none
      reported for empty types.
- [ ] 1.7 `collection_source_bound` — each collection body names only rows
      its pinned source carries (the layer property's dedicated scenario).
- [ ] 1.8 `collection_without_backing_rows_omitted` — a declared implemented
      collection with no source rows omits with `no-backing-document`; no
      page of empty sections.
- [ ] 1.9 `collection_without_a_composition_contract_refused` — declaring a
      refused kind (`capability-catalogue`, `compatibility-matrix`,
      `feature-index`, `related-projects`) refuses with
      `kaya.publications.tier_unimplemented` naming the kind and the
      implemented kinds, tree byte-identical.
- [ ] 1.10 `every_declared_collection_is_implemented_or_refused` — the five
      implemented kinds render; any other collection kind is a labeled
      refusal.
- [ ] 1.11 `collection_declaring_fields_refused` — a collection carrying
      `includes` or `members` refuses naming the collection and the field.
- [ ] 1.12 Dispatch replaces the gate — `partition_declarations` admits the
      five implemented kinds and refuses the four; the refusal message names
      the kind and the implemented kinds (`error.rs`).
- [ ] 1.13 `DEFAULT_COLLECTIONS` grows to exactly `["glossary",
      "topic-map", "knowledge-index", "example-gallery",
      "api-catalogue"]` — never a refused kind.
- [ ] 1.14 Read-list parity — `technical-reference.md` joins publish's fixed
      list (`src/publication/source.rs`) and check's `RESEARCH_READ_ORDER`
      (`src/verification/mod.rs`) in the same change.
- [ ] 1.15 Export rows mirror pages — collection rows carry `emitted` (and
      the omission reason when omitted), same source map as the pages.
- [ ] 1.16 Test pins flip with the behaviour — the
      `non_glossary_collection_still_refused_as_unimplemented` pin narrows
      to a refused kind; the exact pages/export/block lists in
      `tests/cli_pipeline.rs` grow by the new dogfood pages
      (`topic-map.md`, `knowledge-index.md` and their
      `publication.collection.*` blocks); the glossary pins stay.
- [ ] 1.17 Dogfood tree — `kaya build`/`kaya publish` in the dogfood repo
      regenerate with the two new pages; `kaya check` stays clean.

## 2. Follow-up tickets to file

- 2.1 Implementation ticket — section 1, per-kind renderers behind the
  dispatch, read-list parity, export rows, pins. (to be filed)
- 2.2 Dogfood the experiment-record producer: record kaya's next experiment
  run with observations so the dossier gains an
  `exploration.experiment-record` block — un-omits `example-gallery` and,
  with the same rows, the `tutorial` product. (to be filed)
- 2.3 Dogfood reuse rows: import specodelic sources so the projection
  compiles `technical-reference.md` from its spec ids (turu entries render
  the separate `knowledge-brief` block, which is not a catalogue source) —
  un-omits `api-catalogue` and feeds
  `how-to`/`reference` a block their admission rows already admit. (to be
  filed; filed as kaya-quh — amend there)
- 2.4 Read-list gap: `brief.md` carries an `exploration.exploration-brief`
  block but is in neither read list, so `tutorial` cannot compose it — align
  both lists (same seam as 1.14, different product behaviour, own ticket).
  (to be filed)

## 3. Verify (orchestrator)

- [ ] 3.1 `openspec validate --changes --strict` passes.
- [ ] 3.2 `openspec validate add-collection-kinds --strict` passes.
- [ ] 3.3 `git diff --stat src/ tests/` is empty — no production code in
      this change.
- [ ] 3.4 `cargo test` and `cargo clippy --all-targets -- -D warnings` stay
      green.
