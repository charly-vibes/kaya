# Proposal: add-collection-kinds

## Why

The publication tier promises nine collections; eight of them are refused
with "only the glossary collection is implemented". The refusal is honest —
it is the opposite of a stub page — but it is now an accident of a
glossary-only gate rather than a recorded decision, and it understates what
the tree can publish today: four of the eight have admissible, ledger-free
sources available now. The research map already compiles every declared
relationship into `orientation.research-map`; the experiment-record and
technical-reference producers exist (`src/projection/report.rs`,
`src/projection/reuse.rs`) and their files are one read-list entry away; the
node ids a knowledge-index would file are already parsed out of every
compiled block for event attribution. Meanwhile the projection can emit
blocks the publication layer of both doors never reads: `kaya publish` and
`kaya check` today read the same fixed five-file list (check's expected-IR
map is filtered by the identical order), so `technical-reference.md` and
`brief.md` stay invisible to `how-to`, `reference`, and `tutorial` — kinds
their admission rows already admit. This change decides
render-or-refuse per kind, pins the renderers' composition contracts, and
records the refusals as specified behaviour instead of a gate default.

## What Changes

No production code in this change. This proposal fixes the per-kind
composition contracts, the ledger-free data routing, and the commitments the
follow-up implementation must honour.

### Per-kind decision table

| collection | verdict | source — all ledger-free | dogfood today |
|---|---|---|---|
| topic-map | render | relationship statements of `orientation.research-map` (docs/research/index.md, already in the publish read list) | page appears immediately |
| knowledge-index | render | per-type census of the node ids the compiled blocks already carry (parsed today for event attribution); headers from the closed thirteen-entity set | page appears immediately |
| example-gallery | render | `exploration.experiment-record` blocks in experiments/*.md (already read; producer `src/projection/report.rs`) | omitted with `no-backing-document` until a record block exists |
| api-catalogue | render | `reuse-handoff.technical-reference` in technical-reference.md (producer `src/projection/reuse.rs`); requires extending the publish read list | omitted with `no-backing-document` until spec or entry rows exist |
| capability-catalogue | refuse | none — no Capability entity, no admissible rows, no producer; a catalogue would exist because the taxonomy says so | refusal retained, now specified |
| compatibility-matrix | refuse | nothing compiled — per-item status lives in authored docs/src/status.md and ledger verification facts, and the `status-compatibility` product already owns that composition | refusal retained, now specified |
| feature-index | refuse | none — no Feature entity and no feature section in any compiled block | refusal retained, now specified |
| related-projects | refuse | none — every node id is `kaya/…` and the handoff block lists artifacts, not projects | refusal retained, now specified |

Publish stays ledger-free throughout: every renderer composes from the
compiled documents the publication layer reads, exactly as the glossary
composes from compile-time vocabularies and the research-map block proves
per-ledger data can be compiled at build time. Publish-time ledger queries
are rejected (design.md, A1).

### Commitments the implementation must honour

1. **Publish never reads the ledger.** Every collection renderer consumes
   the documents `read_documents` returns — no `read_build_ledger` in the
   publish path, no new block parsed from anything but
   `docs/research/**`. The knowledge-index composes the node ids the
   compiled blocks already carry, not a ledger census.
2. **Refuse, never stub.** The four source-less kinds stay undeclarable:
   declaring one refuses with `kaya.publications.tier_unimplemented`
   naming the kind and the implemented collection kinds, before anything is
   written, leaving the tree byte-identical. `DEFAULT_COLLECTIONS` grows to
   exactly the five implemented kinds; the refused four are never declared.
3. **Omit, never hollow.** A declared implemented collection whose contract
   has no source rows — example-gallery with no experiment-record blocks,
   api-catalogue with no technical-reference block — is omitted and
   reported with reason `no-backing-document`, exactly like the `tutorial`
   product. No page of empty sections is ever written.
4. **Each fact appears once across the whole tree.** The topic-map
   cross-references the page that embeds the research-map body and embeds
   the relationship statements itself only when no page in the tree does.
5. **Publish and check read the same files.** The read-list extension adds
   technical-reference.md to publish's fixed list and to check's
   `RESEARCH_READ_ORDER` in the same change; the two doors never diverge
   again.
6. **The publication block vocabulary is untouched.** Collection pages
   carry `publication.collection.<name>`, validated before any write,
   exactly as today; declarations stay field-free for collections; no
   collection page names a reader question.
7. **The export mirrors the pages.** Collection export rows carry the same
   emitted flag (and omission reason) the summary reports, projected from
   the same source map.

## Impact

- **This change**: an openspec proposal only. No Rust, no behaviour change.
- **Specs**: `kaya-publications` MODIFIED (Justified emission extends from
  products to publications; the collection refusal and omission become
  pinned scenarios; Property coverage mirror gains one row) and ADDED
  (Collection composition contracts; layer sidecar restated with the
  `collection_contracts` constraint and `collection_source_bound`
  property). No delta to `kaya-documents` — no new document kind: the
  census composes node ids the blocks already carry. No delta to
  `kaya-model` — the closed vocabularies are unchanged.
- **Code (follow-up ticket, to be filed)**: per-kind renderers behind a
  per-kind dispatch replacing the glossary gate
  (`src/publication/plan.rs`, `render.rs`); the read-list extension in both
  `src/publication/source.rs` and `src/verification/mod.rs`;
  `DEFAULT_COLLECTIONS` growth (`decls.rs`); the refusal message reword
  (`error.rs`); export collection rows (`render.rs`); test-pin updates per
  the scenario names in the spec delta — the `topic-map`-based
  unimplemented-collection pin narrows to a refused kind, and the exact
  pages/export/block lists in `tests/cli_pipeline.rs` grow by the new
  dogfood pages.
- **Docs**: the implementation ticket updates the `docs/src/status.md` CLI
  row (it still reads "including the `glossary` collection") and the
  dogfood tree gains `topic-map.md` and `knowledge-index.md`.
- **Risks**, each with its trigger and detection signal:
  - the knowledge-index reads as an index of all knowledge when it indexes
    the published corpus — detected in dogfood review; mitigation is the
    census scope sentence on the page, not a ledger read;
  - the read-list extension changes publish output for any tree already
    carrying technical-reference.md — the dogfood tree carries none, and
    the exact pages lists in `tests/cli_pipeline.rs` catch the drift;
  - fact-once ownership between `reference`/`architecture-design` (both
    admit the research-map) and `topic-map` depends on
    allocation order across tiers — detected by the
    `topic_map_cross_references_embedded_edges` scenario test;
  - the refusal-message reword flips byte-pins — detected by the narrowed
    unimplemented-collection test;
  - example-gallery and api-catalogue ship as always-omitted until their
    producers yield rows — visible in the build summary; the follow-up
    dogfooding tickets remove the omission rather than a renderer
    pretending.
