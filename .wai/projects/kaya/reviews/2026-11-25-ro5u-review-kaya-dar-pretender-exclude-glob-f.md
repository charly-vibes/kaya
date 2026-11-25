---
reviews: 2026-11-25-green-kaya-dar-three-edits-applied-1-pretender.md
verdict: pass
tags: [pipeline-run:tdd-ro5u-2026-11-25-kaya-dar-pretender-target-exclude-glob-fix, pipeline-step:ro5u-review]
---

# RO5U Review - kaya-dar (pretender exclude glob fix + assertion hoists)

Verdict: READY · FINAL: CRITICALS=0 · Converged stage 5 · Reviewer subagent on uncommitted working tree, base HEAD 01618d8.

## Scope reviewed

4-file diff (+31/-2): pretender.toml exclude twins (./-prefixed) + book/** pair; tests/impact.rs + tests/ledger.rs one direct assertion each; tests/status_build_properties.rs run_cell returns (accepts, ledger_digest) with wrapper asserting accepted-builds-carry-digest / refused-carry-none across all 10 CELLS.

## Checklist

| Stage | Result |
|---|---|
| 1 Inventory | 4 files, all hunks read in full context |
| 2 Correctness | all three assertion hoists contract-consistent; repaired ledger.rs edit verified intact (three_event_supersedes_cycle_is_refused complete) |
| 3 Performance/scope | thresholds sections byte-untouched; excludes only |
| 4 Adversarial | glob-gap check vs repo reality: no gap; refused-cell Null-index semantics confirmed |
| 5 Converge | verdict below |

## Findings

| Dimension | Finding | Severity |
|---|---|---|
| Tests | `lines().count() == 1` implied by helper's byte-equality + single-line seed — deliberate direct assertion, no coverage gain | P3 / LOW |
| Maintainability | glob-twin pairs can drift; mitigated by in-array comment pinning 0.7.0 behavior | P3 / LOW |

FINAL: CRITICALS=0

