---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-i03-research-map-edges, pipeline-step:green]
---

GREEN kaya-i03: src/projection/index.rs renders orientation.research-map: shared validate_generated_blocks (vocabulary + duplicate refusal) runs before any write; ### Relationships renders every edge once as '- <from> -[<type>]-> <to>' sorted by (from, relationship, to); ### Isolated lists every admitted node without an incident edge; Artifacts/Runs/Sources attribute sections unchanged; trailing '_n nodes from m events._' counts distinct rendered ids; summary and sources keyed by the block name. Validator moved to projection::mod and shared with report::build (closes s5v review M1). docs/research/index.md migrated to the new block, authored prose updated; docs/src/document-taxonomy.md status paragraph updated. Verified: 179 tests, fmt and clippy clean, build idempotent (index unchanged on second run).
