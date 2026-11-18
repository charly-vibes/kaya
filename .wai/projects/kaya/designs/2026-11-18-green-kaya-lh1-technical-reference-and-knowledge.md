---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-lh1-reuse-families-technical-reference-knowledge-brief, pipeline-step:green]
---

GREEN kaya-lh1 — technical-reference and knowledge-brief at the projection layer

Change (src/projection/reuse.rs):
- New public constants TECHNICAL_REFERENCE_BLOCK / KNOWLEDGE_BRIEF_BLOCK plus files technical-reference.md / knowledge-brief.md; `documents()` now returns four pairs in canonical kind order (technical-reference, reproduction-package, knowledge-brief, handoff), which keeps main.rs stale-authored iteration and summary ordering deterministic.
- `Inputs` gains `specs` and `entries`; `inputs()` selects them from `Source` nodes by the non-blank `spec` and `entry` attributes, independently of the existing path+commit revision selection, so one source may contribute several references.
- `build()` now destructures four targets and validates every existing document before writing any (`read_validated`), preserving the "legacy block leaves all bytes untouched" contract across the whole family.
- Three single-section kinds share `render_flat` (sorted, single-line, deduplicated rows plus distinct backing ids); reproduction keeps its two-section renderer.

Decisions: rows name ids only (no payload copy, no ledger write); absent/blank attributes select nothing; a present non-string attribute is refused with kaya.projection.reuse_attribute naming node and attribute; only `Source` nodes feed the adapter-backed kinds (an `Artifact` carrying `entry` is ignored and tested).

Verification:
- `cargo test --test reuse_adapters`: 8 passed.
- `cargo test`: 264 passed, 0 failed across 24 targets (up from 256/23).
- `ah check --changes add-document-taxonomy --run-tests`: 15 passed, 19 structural; the new brief-adds-no-knowledge contract executes both selectors.
- `openspec validate --changes --strict`: add-core-research-record passes; add-document-taxonomy fails on two pre-existing long-requirement WARNINGs in kaya-publications/spec.md. No spec file is modified by this ticket (git status shows only src/projection/reuse.rs and three test files), so the failure is drift inherited by cc9, not caused here.

Expected-behavior updates in existing tests (new kinds are always reported when empty): tests/reuse.rs omission sets, tests/cli_pipeline.rs five omitted lists, tests/acceptance.rs dossier omission list.
