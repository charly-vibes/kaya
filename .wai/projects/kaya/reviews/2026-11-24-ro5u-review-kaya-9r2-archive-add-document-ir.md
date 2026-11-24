---
reviews: ro5u-kaya-9r2.md
verdict: fail
tags: [pipeline-run:tdd-ro5u-2026-11-24-kaya-9r2-archive-add-document-ir-commit-gate, pipeline-step:fix-review]
---

# RO5U Review — kaya-9r2 (archive add-document-ir-commit-gate)

Reviewed the full ticket diff (42 tracked files, +127/−860, plus new archive/deploy trees).

## Stage 1 — DRAFT
Shape good: archive bookkeeping follows the kaya-8f4 precedent (openspec archive + ah archive + docs regen); no production code. Quality: GOOD.

## Stage 2 — CORRECTNESS
- [CORR-001][HIGH] `openspec/specs/kaya-impact/layer.md` missing. openspec archive applies only spec.md; the 79-line layer sidecar from the delta was never copied (kaya-8f4 copied `openspec/specs/kaya-publications/layer.md` manually; SUMMARY.md now asserts kaya-impact keeps a layer.md). Fix: copy the archived delta layer.md to openspec/specs/kaya-impact/layer.md; rerun `just docs`.
- [CORR-002][HIGH] `openspec/specs/kaya-impact/spec.md` Purpose is the TBD archive placeholder. kaya-publications precedent has a written purpose (its delta was ADDED-only; purpose written post-archive, as the placeholder itself instructs). Fix: write the Purpose from the proposal's Why.

TypeSafe verification (key set): CORR-001 verified 0.88, CORR-002 verified 0.97; both ≥ 0.8 gate; measured FP rate 0/2.

Correctness quality: FAIR (two HIGH, both fixable now).

## Stage 3 — CLARITY
- [CLAR-001][LOW] SUMMARY groups kaya.impact after kaya.projection — matches IR-producer→consumer reading; no change needed.
Clarity quality: GOOD.

## Stage 4 — EDGE CASES
- [EDGE-001][MEDIUM, pre-existing, not introduced by this diff] docs/src/status.md "In progress" still says the turu adapter is "specified but not written" though kaya-syx closed. status.md belongs to kaya-3tm's release pass — deferred, tracked there.
- [EDGE-002][LOW, pre-existing] .espectacular tool_version inconsistency (kaya-documents tomls carry a 0.12.3 header, kaya-projection none, config 0.12.2 vs binary 0.12.3); `ah upgrade` exists; out of scope.
Edge case coverage: GOOD.

## Stage 5 — EXCELLENCE
Verification battery green: ah check 0 findings; ah check --run-tests 227 passed, 0 findings; cargo test all targets ok; just links 37 pages 0 unresolved; openspec list has no active changes.

Convergence: reached at stage 4 — 0 new critical, FP 0%.

## Verdict
NEEDS_REVISION — fix CORR-001 and CORR-002 before ship-close.

