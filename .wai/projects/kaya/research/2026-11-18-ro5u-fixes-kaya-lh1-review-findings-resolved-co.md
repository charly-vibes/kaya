---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-lh1-reuse-families-technical-reference-knowledge-brief, pipeline-step:fix-review]
---

RO5U-FIXES kaya-lh1 — review findings resolved

CORR-001 (MEDIUM) fixed: inputs_select_spec_and_entry_references now proves independence with one Source carrying spec+entry+path+commit; it contributes one row to specs, entries, and revisions.
CORR-002 (LOW) fixed cheaply: render_flat filters blank references out of the block source map, so an attributed id always renders a row.
CLAR-001 (LOW) fixed: block_names_are_vocabulary_names now asserts a zip against an explicit (block, file, kind) table — order, path, vocabulary name, and Kind::parse round-trip are visible in one place.
EDGE-001 (LOW) fixed: authored-byte preservation and stale-block removal are parameterized over technical-reference and knowledge-brief.
EDGE-002 (LOW) tracked: the `spec`/`entry` attribute contract is recorded as comments on kaya-24c and kaya-syx, the deferred adapters that must emit those keys.
EXCL-001 (LOW) not fixed here: `openspec validate --changes --strict` still reports two pre-existing long-requirement warnings inside the add-document-taxonomy kaya-publications delta; no spec file is touched by lh1 and kaya-cc9 owns that delta.

Verification: cargo test 264 passed / 0 failed across 24 targets; cargo clippy --all-targets -D warnings clean; cargo fmt --check clean; ah check --changes add-document-taxonomy --run-tests 15 passed / 19 structural.
