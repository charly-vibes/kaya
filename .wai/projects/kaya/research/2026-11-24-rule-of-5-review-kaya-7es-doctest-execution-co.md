---
tags: [pipeline-run:tdd-ro5u-2026-11-24-kaya-7es-doctest-execution-contracts, pipeline-step:ro5u-review]
---

# Rule of 5 Review — kaya-7es doctest execution contracts

**Work reviewed:** code — kaya-7es diff vs c0c767d (src/doctest.rs new; src/adapters/testrunner.rs; src/projection/ir.rs; src/main.rs; openspec/specs/kaya-documents/*; .espectacular bindings)
**Convergence:** Stage 4 (TypeSafe-verified pass applied; measured FP 0/1)

## Summary

Total Issues by Severity:
- CRITICAL: 0
- HIGH: 1 — CORR-001 (TypeSafe verified @ 0.95)
- MEDIUM: 1 — EDGE-003 (UNVERIFIED, below severity cap)
- LOW: 4 — CORR-002, CLAR-001, EDGE-001, EDGE-002, EXCL-001 (UNVERIFIED)

## STAGE 1: DRAFT

Shape GOOD: the doctest surface is isolated in `src/doctest.rs`; the IR gains an
additive `RecordKind::Doctest` variant; `capture_with_kind` parameterizes the
existing adapter instead of duplicating it; the CLI mirrors the `acknowledge`
human-door pattern. Scenario bindings and layer property rows follow the
deployed house format.

## STAGE 2: CORRECTNESS

[CORR-001] HIGH — src/projection/ir.rs:187 (materialize) with
validate_generated_blocks call sites (report.rs:365, report.rs:510, index.rs:83,
reuse.rs:193, reuse.rs:241)
Description: a malformed ```doctest declaration in authored prose (unknown
runner, unterminated fence, no steps, duplicate label) passes every planner's
`validate_generated_blocks` gate — it only checks generated block names — and
reaches `materialize`, which panics
(`materialize requires grammar-valid doctest declarations; ...`). Both `kaya
build` and `kaya ir` are user-reachable panic paths for hand-edited documents.
Evidence: `unwrap_or_else(|error| panic!(...))` at ir.rs:187; no call site runs
`doctest::extract` before planning. TypeSafe verification: **verified @ 0.95**.
Recommendation: extend `validate_generated_blocks` to also run
`doctest::extract(doc).map(|_| ())` and update its doc comment; every planner
then refuses with a labeled `ProjectionError` before rendering. The materialize
panic stays as an invariant guard.

[CORR-002] LOW — src/doctest.rs extract()
Description: extraction scans the whole document, including generated blocks; a
generated block containing a ```doctest fence would create a phantom record.
Evidence: kaya's generated vocabulary never emits doctest fences, so the path is
unreachable today.
Recommendation: no action; revisit if a renderer ever emits fenced examples.

Correctness Quality: GOOD

## STAGE 3: CLARITY

[CLAR-001] LOW — src/main.rs cmd_doctest
Description: the local `report: String` shadows the `report` module import.
Impact: mild reader confusion inside the function body.
Recommendation: rename to `report_lines`.

Clarity Quality: GOOD

## STAGE 4: EDGE CASES

[EDGE-003] MEDIUM — src/main.rs cmd_doctest
Description: the document walk is `read_dir` non-recursive, so
`docs/research/experiments/*.md` (experiment reports — a natural home for
declared run commands) never run.
Scenario: a user declares a contract in an experiment report; `kaya doctest`
silently reports declared 0.
Impact: the feature silently under-covers its own compiled document set.
Recommendation: recurse over `*.md` under `--out`.

[EDGE-001] LOW — src/doctest.rs build_contract
Description: an expected-output line beginning with `$ ` cannot be expressed
(parsed as the next step).
Recommendation: document the grammar limitation in the module docs.

[EDGE-002] LOW — src/doctest.rs run_step
Description: spawn failure conflates to `Failure::Exit(127)`.
Recommendation: one comment stating the command-not-found convention.

Edge Case Coverage: FAIR → GOOD after EDGE-003

## STAGE 5: EXCELLENCE

[EXCL-001] LOW — openspec/specs/kaya-documents/layer.md
Description: new constraint/property rows break the table pipe alignment.
Recommendation: realign to the existing columns.

Excellence Assessment:
- Structure: GOOD
- Correctness: GOOD
- Clarity: GOOD
- Edge Cases: FAIR
- Overall: WITH_NOTES

Production Ready: WITH_NOTES — fix CORR-001 and EDGE-003 before ship; LOWs
opportunistic.

## Verdict

NEEDS_REVISION — one HIGH (verified) and one MEDIUM; no criticals. After the
fix pass and a green re-run, READY.

