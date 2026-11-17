---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-i03-research-map-edges, pipeline-step:quality-ledger]
---

QUALITY LEDGER kaya-i03

Changed:
- src/projection/index.rs: RESEARCH_MAP_BLOCK=orientation.research-map; build validates generated blocks before writing; renders ### Relationships ('- <from> -[<type>]-> <to>' sorted+deduped), ### Isolated for edge-less nodes outside the detail kinds, the Artifacts/Runs/Sources detail sections, and the distinct-node count line; summary/sources keyed by the block name.
- src/projection/mod.rs: shared validate_generated_blocks (vocabulary + duplicate refusal); src/projection/report.rs uses it (s5v review M1 closed).
- tests/research_map.rs (new, 9 tests), tests/projection_index.rs (12), tests/acceptance.rs (2), tests/cli_pipeline.rs (19) migrated and strengthened.
- .espectacular/changes/add-document-taxonomy/kaya-documents/edges-rendered.toml bound to the edge test.
- docs/research/index.md migrated to the new block (authored prose updated, detail rows no longer duplicated); docs/src/document-taxonomy.md status paragraph updated.

Verified:
- cargo test: 184 passed, 0 failed (16 targets).
- cargo fmt --all --check clean; cargo clippy --all-targets --all-features -- -D warnings clean.
- cargo run -- build twice: recomputed [], unchanged [exploration.experiment-specification:..., orientation.research-map], omitted [{exploration, experiment-record, no-records}].
- ah check --changes add-document-taxonomy --run-tests: 5 passed (all taxonomy scenarios landed so far), 28 structural (remaining tickets).
- openspec validate unchanged specs.

Review:
- rule-of-5-universal, five passes: 0 critical, 0 high, 2 medium, 6 low; all fixed. Notable: two reviewers independently found that Isolated duplicated Artifact/Run/Source detail rows (73 of 77 nodes), contradicting the scenario's lists-it-once rule; a third found sort/dedup unguarded by tests. Both fixed and now pinned.

Risks:
- Commands are not captured by any adapter yet, so a reproduction package can only render revisions until a command source exists (kaya-3f6 scope).
- The map's detail sections still render attribute rows separately from the graph view; the boundary is documented in the module docs and the taxonomy page.

Next:
- Ship: tick tasks.md 3.4, close kaya-i03, export beads, commit, then start kaya-3f6 (reuse references) in the next tdd-ro5u run.
