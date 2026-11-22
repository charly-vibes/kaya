---
tags: [pipeline-run:tdd-ro5u-2026-11-22-kaya-d0n-agent-session-capture-proposal, pipeline-step:quality-ledger]
---

# QUALITY LEDGER kaya-d0n — agent-session capture proposal

Run: tdd-ro5u-2026-11-22-kaya-d0n-agent-session-capture-proposal.

## Gates and results (orchestrator-run)
- ORIENT: artifact records the current capture surface with quoted
  file:line facts (turu.rs:41, turu.rs:28/127/136/140, wai.rs:1-8,
  model.rs:25/31, event.rs:108-115).
- PLAN: artifact fixes the decision (distilled), the alternatives, the five
  coverage commitments, the change shape and the risks.
- ANALYSIS (red step for a design ticket): what would falsify the choice,
  the risks it carries, and the four conditions that unlock the raw surface.
- GREEN: the proposal-only change `add-agent-session-capture` with
  `.openspec.yaml` (`schema: spec-driven`, `skip_specs: true`).
- REFACTOR: proposal tidy (section order, single statement of the unlock
  conditions, one commitment block per coverage area, pinned terms, the
  falsification test).
- VERIFY: `openspec validate --changes --strict` → pass; `git diff --stat
  src/` → empty (no production code); `cargo test` → 31/31 suites ok;
  `cargo clippy --all-targets -- -D warnings` → clean; deployed `ah check`
  → 0 findings.

## Remaining known risks
- The distilled surface may be too thin for the questions the external
  evaluation implied; the follow-up ticket must measure it (the falsification
  test in the analysis artifact).
- The redaction pattern list is a starting set, unproven against real
  transcripts.
- The `ActorType::Agent` switch changes shipped event bytes and needs its own
  migration story.

## Next action
Ship-close (pre-approved in the session's batch scope): archive the
proposal-only change, commit it with the beads export closing kaya-d0n, and
file no push (fleet policy requires explicit authorization).

