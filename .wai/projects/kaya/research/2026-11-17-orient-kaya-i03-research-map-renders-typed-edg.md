---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-i03-research-map-edges, pipeline-step:orient]
---

# ORIENT kaya-i03 — research map renders typed edges

Ticket: kaya-i03, P1, lane:afk, complexity:m, base_commit df81656,
files: src/projection/index.rs, src/model.rs.
Spec: kaya.documents requirement "Research map renders relationships"
(scenario edges_rendered); layer property relationship_rendered; delta
scenario "Typed edge rendered" and "Isolated entity still listed".

## Acceptance as testable statements

1. Every declared edge among admitted entities appears exactly once with both
   endpoints and its relationship type; the map is a typed edge list, not a
   type-grouped entity list.
2. An admitted entity with no incident edge is listed exactly once and no edge
   is invented for it.
3. The map's generated block is named orientation.research-map (the closed
   vocabulary name for Kind::ResearchMap); the legacy research-index block is
   refused before writing, like any out-of-vocabulary block.
4. Rendering is deterministic and idempotent: same graph -> same bytes; a
   second build reports the block unchanged and writes nothing; the source map
   names every rendered node.
5. Authored prose and foreign managed blocks in docs/research/index.md survive
   byte for byte; the committed index is migrated to the new block name.

## Current code seams

- src/projection/index.rs: INDEX_BLOCK="research-index", entity list grouped by
  type (Question/Hypothesis/Experiment/Finding/Decision), then Artifacts/Runs/
  Sources attribute sections, then `_{n} nodes from {m} events._`; sources map
  keyed by block name.
- src/projection/mod.rs: scan/replace_generated/remove_generated; label
  LABEL_MANAGED_GRAMMAR; no vocabulary gate.
- src/projection/report.rs: private validate_blocks (vocabulary + duplicate
  refusal) - s5v review M1 defers the shared validator and the index rename to
  this ticket.
- src/model.rs: Graph::nodes/edges/node, Relationship Display/as_str (nine
  typed relationships, validated direction).
- docs/research/index.md: authored prose names the `research-index` block twice;
  generated block lists 76 nodes from 80 events.

## Blast radius (grep-verified)

tests/acceptance.rs (INDEX_BLOCK_BEGIN/END, `lists(recomputed, "research-index")`,
generated_index content), tests/cli_pipeline.rs (block marker, summary name,
node count line), tests/projection_index.rs (END_BLOCK const, block assertions),
src/projection/index.rs, src/main.rs (no change expected), docs/research/index.md.

## Validation

narrow: cargo test --test research_map --test projection_index --test acceptance --test cli_pipeline
full: cargo test; ah check --changes add-document-taxonomy --run-tests; openspec validate --changes --strict
baseline: HEAD 6b41ea5 tree green, 174 tests, fmt/clippy clean.

