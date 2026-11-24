---
tags: [pipeline-run:tdd-ro5u-2026-11-24-kaya-w66-cap-proposals-production-caller, pipeline-step:orient]
---

ORIENT kaya-w66: cap_proposals (src/promotion.rs:472) implements pending_proposals_bounded (openspec/specs/kaya-promotion/spec.md:28, scenario pending_proposals_bounded_checked :198-203) with contract tests (tests/promotion.rs:328 cap_proposals_ranks_then_caps_at_budget) but no production caller. Production surface: main.rs cmd_status reports "pending_proposals": pending_proposals()? at :590; pending_proposals() (:650-665) reads the ledger, replays PromotionState::from_events, returns state.pending_count() — uncapped. Spec invariant: "pending proposals fit one session-close; over-cap proposal generation is ranked/capped, never pushed onto the human; proposals never expire and never masquerade as settled" and proposal_surfacing_pulled: counts at point of use only. No budget constant exists anywhere. Ranking: more cited events first, then assertion id ascending; only proposed status candidates; established/challenged excluded.
Acceptance restated: (W1) with pending proposals over budget, cap_proposals executes on the status production path; (W2) kaya status reports pending_proposals <= budget; (W3) existing contract tests stay green; (W4) under-budget counts unchanged (status_counts_pending_proposals stays 1:1).
Out of scope: dossier build/read paths (:949 state.pending_count()) stay uncapped — ticket wires the FIRST proposal-generating path (status); no budget configuration flag; no ranking changes.

