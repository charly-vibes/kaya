---
tags: [pipeline-run:tdd-ro5u-2026-11-22-kaya-d0n-agent-session-capture-proposal, pipeline-step:ship-close]
---

# SHIP kaya-d0n — agent-session capture proposal

Run: tdd-ro5u-2026-11-22-kaya-d0n-agent-session-capture-proposal — ship-close
evidence.

## Approval
Human checkpoint covered by the session's batch approval (q5e ship plus
pre-approved q6s and d0n), 2026-11-22.

## What ships
`openspec/changes/add-agent-session-capture/` — a proposal-only openspec
change:
- `proposal.md`: the distilled-vs-raw decision (distilled), with the
  reasoning, the four conditions that would unlock the raw surface, and
  concrete commitments for secret redaction, size bounds, dedup by content
  hash, event kind and actor typing, and MVP-2 relationship linking hooks.
- `tasks.md`: the research steps done here plus the follow-up tickets to
  file.
- `.openspec.yaml`: `schema: spec-driven`, `skip_specs: true` — the change
  declares no spec-level behavior change.

No production code: `git diff --stat src/` is empty.

## Final verification (this session)
- `openspec validate --changes --strict` → pass.
- `cargo test` → 31/31 suites ok; `cargo clippy --all-targets -- -D warnings`
  → clean.
- Deployed `ah check` → 0 findings.
- ro5u review of the proposal: CRITICAL 0 (findings and fixes recorded in
  the run's review artifacts).

## Commit/ticket state
- Atomic commit follows: the proposal-only change, the .wai run state, and
  the beads export with kaya-d0n closed.
- No push (fleet policy: push requires explicit authorization).

