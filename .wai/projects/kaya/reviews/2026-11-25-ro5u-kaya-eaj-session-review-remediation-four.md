---
reviews: ro5u-kaya-eaj.md
verdict: pass
tags: [pipeline-run:tdd-ro5u-2026-11-25-kaya-eaj-session-review-remediation-four-findings, pipeline-step:ro5u-review]
---

# RO5U kaya-eaj — session-review remediation, four findings (diff of working tree, 2026-11-25)

Scope: src/commands/impact.rs, src/impact.rs, src/commands/publication.rs,
.github/workflows/record.yml, tests/acceptance.rs, docs/src/verification-walk.md,
docs/src/SUMMARY.md, openspec/specs/kaya-impact/spec.md,
openspec/specs/kaya-projection/spec.md, .espectacular/{kaya-impact,kaya-projection}/
(five new scenario tomls).

Five-stage review of the in-run remediation for the four ro5u findings
(CLAR-001, EDGE-001, EDGE-002, EXCL-001). Stages: Draft EXCELLENT ·
Correctness EXCELLENT · Clarity GOOD · Edge EXCELLENT.

## Critical
- none. CRITICAL_COUNT: 0 — gate passes.

## High
- none.

## Medium
- none.

## Low
1. **record_trail allocation per record** — `ledger_block_names` returns a
   `Vec<String>` per authored record id, one small allocation per record
   rather than the old `&[&str]`. Per-block, not per-byte; correctness
   outweighs it until the naming seam is unified. File: `src/commands/publication.rs`
   (trail filter).
2. **Unresolved-provenance reachability** — the walk's `unresolved` branch is
   only reachable via the resolver seam (build-time cite validation refuses
   unknown cites, so CLI flows cannot produce one). Pinned by
   `resolve_provenance_flags_unknown_ids` unit test at the seam rather than
   through an unreachable CLI scenario. File: `src/commands/publication.rs`.

## INFO
- Autostash kept in the retry loop: inert on the clean scheduled CI checkout,
  useful on manual trigger. File: `.github/workflows/record.yml`.
- Legacy dedup deliberately does not swallow rationale-bearing candidates: a
  rationale review is a different decision and still appends. File:
  `src/commands/impact.rs`.

## Verdict
READY. Criticals 0. Suite 516/516, acceptance 7/7, clippy+fmt clean,
`ah check` 0 issues, `just validate` ingredient set green.

