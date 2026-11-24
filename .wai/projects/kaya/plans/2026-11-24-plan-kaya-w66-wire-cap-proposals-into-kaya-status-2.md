---
tags: [pipeline-run:tdd-ro5u-2026-11-24-kaya-w66-cap-proposals-production-caller, pipeline-step:plan]
---

PLAN kaya-w66: wire cap_proposals into kaya status.
- src/promotion.rs: add `pub const PROPOSAL_BUDGET: usize = 16;` (doc: one session-close's approval budget, per pending_proposals_bounded; named constant so tests and CLI share it).
- src/main.rs pending_proposals(): return cap_proposals(&state.assertions, promotion::PROPOSAL_BUDGET).len() instead of state.pending_count(); import cap_proposals + the const.
- tests/cli_pipeline.rs RED first: status_caps_pending_proposals_at_the_budget — seeds PROPOSAL_BUDGET+1 proposed assertions (record_assertion helper), asserts status pending_proposals == PROPOSAL_BUDGET (fails today: 17 reported).
- Espectacular: bind into existing kaya-promotion scenario toml for pending_proposals_bounded if present (append cargo entry), else report absence.
- Narrow: cargo test --test cli_pipeline status_caps_pending; full: cargo test. Commit shape: one feature commit fix(status) — no tidy expected (two-line change).

