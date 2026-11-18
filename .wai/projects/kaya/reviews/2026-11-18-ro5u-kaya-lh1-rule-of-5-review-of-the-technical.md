---
reviews: 2026-11-18-green-kaya-lh1-technical-reference-and-knowledge.md
verdict: pass
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-lh1-reuse-families-technical-reference-knowledge-brief, pipeline-step:fix-review]
---

RO5U kaya-lh1 — Rule-of-5 review of the technical-reference/knowledge-brief projection slice

Artifact under review: src/projection/reuse.rs (diff vs f2709ac), tests/reuse_adapters.rs (new), expectation updates in tests/{reuse,cli_pipeline,acceptance}.rs.
TYPESAFE_API_KEY is not set: all findings are UNVERIFIED (self-reported).

STAGE 1 — DRAFT
Shape matches the module it extends: block/file constants, Inputs fields, documents(), build(), inputs(), one renderer per kind with render_flat shared by the three single-section kinds. No new abstraction layer, no second convention.
Shape Quality: GOOD.

STAGE 2 — CORRECTNESS
CORR-001 MEDIUM — tests/reuse_adapters.rs
The doc comment and plan claim spec/entry/revision selections are independent ("one source may carry several"), but no test gives one Source all three attributes; the fixture spreads them across three nodes, so a regression making the selections exclusive would pass.
Fix: add a node carrying spec+entry+path+commit and assert one row in each of specs, entries, revisions.

CORR-002 LOW — src/projection/reuse.rs render_flat
`sources` comes from unfiltered references while the body comes from filtered rows, so a caller-supplied blank reference attributes the block to an id that renders nothing. Pre-existing shape for handoff, now reused twice; inputs() already filters blanks, so no production path exposes it.

Correctness Quality: GOOD (no defect against kaya.documents "Reuse families reference, never copy"; ledger untouched; rows carry references only).

STAGE 3 — CLARITY
CLAR-001 LOW — tests/reuse_adapters.rs block_names_are_vocabulary_names
The file-name assertion is hidden inside an O(n) find with format! and unwrap_or_default; the invariant (canonical order, file named for kind) is not visible. Fix: assert a zip against an explicit (block, file, kind) table.

Clarity Quality: GOOD.

STAGE 4 — EDGE CASES
EDGE-001 LOW — coverage
Authored-byte preservation and stale-block removal are tested for technical-reference only; knowledge-brief is symmetric but unproven. Fix: parameterize over both adapter kinds.

EDGE-002 LOW — contract ownership
`spec` and `entry` are now a projection input contract documented only in the module header; kaya-syx and kaya-24c must emit exactly these keys. Fix: carry the contract into those tickets (spec edits are an lh1 anti-goal).

Edge Case Coverage: GOOD.

STAGE 5 — EXCELLENCE
EXCL-001 LOW — pre-existing meter drift
`openspec validate --changes --strict` fails on two long-requirement WARNINGs in the add-document-taxonomy kaya-publications delta; no spec file is modified here. cc9 owns that delta.

Convergence (after stage 4): new CRITICAL 0; new issues 2 (LOW), rate falling from stage 2; false positives 0. Status CONVERGED.

Final: CRITICAL 0, HIGH 0, MEDIUM 1, LOW 4.
Verdict: READY WITH NOTES — fix CORR-001 and CLAR-001 in fix-review; EDGE-001 cheap to fix; EDGE-002 tracked; CORR-002 deferred with reason.

