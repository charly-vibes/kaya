---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-i03-research-map-edges, pipeline-step:red]
---

RED kaya-i03: cargo test --test research_map -> E0432 unresolved import kaya::projection::index::RESEARCH_MAP_BLOCK (expected: the block constant and map rendering do not exist). tests/research_map.rs has 5 tests (typed edges once with both endpoints/type, isolated listed once with no invented edge, legacy block refused before writing with bytes unchanged, block name equals Kind::ResearchMap.block_name(), deterministic+idempotent). Migrated tests/projection_index.rs, tests/acceptance.rs, tests/cli_pipeline.rs to orientation.research-map; contract .espectacular/changes/add-document-taxonomy/kaya-documents/edges-rendered.toml binds the edge test. No fixture/setup errors.
