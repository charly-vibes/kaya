---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-i03-research-map-edges, pipeline-step:plan]
---

# PLAN kaya-i03 — research map renders typed edges

## Behavior (spec ids)

- kaya.documents/Research map renders relationships: typed edge per declared
  relationship naming both endpoints; admitted entities with no edges listed.
- kaya.documents/Family-scoped generated blocks: the map block is
  orientation.research-map; out-of-vocabulary generated blocks in index.md are
  refused before writing (closes s5v review M1).
- Bound scenario: edges_rendered (Property coverage mirror).

## Out of scope

project-overview (kaya-1b9); publications; new entity kinds; spec edits;
changing edge direction rules in src/model.rs (the model already validates).

## Contract (binding)

src/projection/index.rs
```rust
pub const RESEARCH_MAP_BLOCK: &str = "orientation.research-map";
pub fn build(graph: &Graph, path: &Path, event_count: usize) -> Result<Summary, ProjectionError>;
```
Rendered body, deterministic:
1. `### Relationships` (omitted when the graph has no edges):
   `- <from> -[<relationship>]-> <to>` for every edge, sorted by
   (from, relationship, to); each edge exactly once, both ids literal as they
   appear in the graph, relationship lowercase (Relationship::as_str).
2. `### Isolated` (omitted when empty): admitted entities with no incident edge
   (neither from nor to), sorted by id:
   `- <id> — <EntityType> — <title-or-key>` (existing title_or_key rule, single
   line, same sanitisation as attribute()).
3. Existing attribute sections unchanged and still only when non-empty:
   `### Artifacts` (`- <id> — <label>`), `### Runs`
   (`- <id> — passed N, failed N, ignored N`), `### Sources`
   (`- <id> — <path>@<commit>`).
4. Blank line, then `_{n} nodes from {event_count} events._`, where n = number
   of distinct node ids appearing in any rendered row (edges endpoints,
   isolated rows, artifact/run/source rows).
5. `summary.sources[RESEARCH_MAP_BLOCK]` = those ids, sorted and deduped.
   recomputed/unchanged entries use the block name `orientation.research-map`.

src/projection/mod.rs: move the vocabulary/duplicate gate out of report.rs as
`pub fn validate_generated_blocks(doc: &str) -> Result<(), ProjectionError>`
(LABEL_LEGACY_BLOCK for names outside the vocabulary, LABEL_MANAGED_GRAMMAR for
duplicates); report::build and index::build both call it before any write.

Migration: docs/research/index.md - remove the legacy `research-index` block
(authored/foreign bytes untouched), update the two authored references to the
block name, then regenerate with `cargo run -- build`. Block markers become
`orientation.research-map`; the committed doc carries relationships from the
committed ledger and isolated rows for nodes without edges.

## Tests to write first

tests/research_map.rs (new)
1. `every_declared_edge_renders_once_with_both_endpoints_and_type` - graph with
   supports/challenges/derives/executes edges; each edge line appears exactly
   once; line count equals edge count; both ids and the type in each line.
2. `isolated_entity_is_listed_once_and_no_edge_is_invented` - one isolated node
   plus one edge; isolated row appears once in `### Isolated`, no `-[` near it.
3. `legacy_block_in_the_index_is_refused_before_writing` - seed index.md with
   `generated:experiment-report` and authored bytes; build errs with
   LABEL_LEGACY_BLOCK naming the block; file bytes unchanged.
4. `map_block_name_is_the_vocabulary_name` - RESEARCH_MAP_BLOCK ==
   Kind::ResearchMap.block_name().
5. `map_is_deterministic_and_idempotent` - two builds byte-identical; second
   build unchanged, no write.

Migrated: tests/projection_index.rs (block constants/rows), tests/acceptance.rs
(INDEX block constants, recomputed/unchanged names, generated content),
tests/cli_pipeline.rs (block marker, summary name, node-count line).

Contract: .espectacular/changes/add-document-taxonomy/kaya-documents/
edges-rendered.toml with `[tests] cargo = [{flags =
"every_declared_edge_renders_once_with_both_endpoints_and_type"}]`.

## Commands

narrow: cargo test --test research_map --test projection_index --test acceptance --test cli_pipeline
full: cargo test; ah check --changes add-document-taxonomy --run-tests; openspec validate --changes --strict

## Files touched

src/projection/index.rs, src/projection/mod.rs, src/projection/report.rs,
tests/research_map.rs, tests/projection_index.rs, tests/acceptance.rs,
tests/cli_pipeline.rs, docs/research/index.md, .espectacular/changes/
add-document-taxonomy/kaya-documents/edges-rendered.toml,
openspec/changes/add-document-taxonomy/tasks.md (tick 3.4),
.beads/issues.jsonl.

## Commit shape

1. `feat(projection): render the research map as typed edges`
2. evidence commit for the pipeline run artifacts.

