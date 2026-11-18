---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-lh1-reuse-families-technical-reference-knowledge-brief, pipeline-step:plan]
---

PLAN kaya-lh1 — projection-layer technical-reference and knowledge-brief

Desired behavior (kaya.documents "Reuse families reference, never copy", scenario kaya-documents/brief-adds-no-knowledge, plus the taxonomy vocabulary):
- A `Source` with a non-blank `spec` attribute contributes a technical-reference row naming the specodelic spec id.
- A `Source` with a non-blank `entry` attribute contributes a knowledge-brief row naming the turu/whisper entry id.
- Both kinds write markdown only and create no ledger entity; every row carries its reference; empty kinds are omitted once with no file/placeholder; a present non-string attribute is refused with kaya.projection.reuse_attribute.

Out of scope: turu/specodelic ledger ingestion (kaya-syx, kaya-24c, deferred), payload copying, spec deltas in add-core-research-record, publication admission changes (already admits both kinds), taxonomy vocabulary changes (already present).

Tests written first in tests/reuse_adapters.rs (expected: compile succeeds, all fail because the kinds are not rendered; the refusal test may pass early only if inputs rejects the attribute):
T1 inputs_select_spec_and_entry_references — spec/entry sources selected, blank attributes skipped, non-string `spec` refused naming node+attribute.
T2 technical_reference_renders_spec_ids — build writes technical-reference.md; block body is exactly the sorted `- <node id> — <spec id>` rows.
T3 knowledge_brief_names_every_entry_id — build writes knowledge-brief.md; every row names its entry id; duplicates collapse.
T4 ledger_gains_no_entity_and_only_markdown_is_written — a seeded ledger file is byte-identical after build and the output dir holds only .md files.
T5 empty_kinds_are_omitted_once_and_write_no_file — empty Inputs reports each of the four reuse kinds exactly once with reason no-records and writes nothing.
T6 stale_technical_reference_block_removed — a pre-existing document loses the block and is deleted when only whitespace remains.
T7 block_names_are_vocabulary_names — documents() pairs the canonical order technical-reference, reproduction-package, knowledge-brief, handoff with the vocabulary block names.
T8 authored_bytes_survive_around_new_blocks — authored and foreign-tool bytes around a new block are preserved byte-for-byte.

Narrow command: cargo test --test reuse_adapters
Full verification: cargo test; openspec validate --changes --strict; ah check (deployed specs unchanged); just ci at ship time.

Files: src/projection/reuse.rs (Inputs, documents, build, inputs, render fns, module header), tests/reuse_adapters.rs (new). No production callers need changes: src/main.rs iterates reuse::documents and merges summaries generically.

Commit shape: one feature commit for red+green of both kinds; a separate refactor commit only if review identifies a real tidy; then the evidence commit (ship record, ticket close, beads export).
