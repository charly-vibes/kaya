---
tags: [pipeline-run:tdd-ro5u-2026-11-24-kaya-w66-cap-proposals-production-caller, pipeline-step:green]
---

GREEN kaya-w66: cargo test full suite 435 passed / 0 failed; status_caps_pending_proposals_at_the_budget green — status reports exactly PROPOSAL_BUDGET (16) with 17 seeded proposals. Implementation: pub const PROPOSAL_BUDGET in src/promotion.rs; pending_proposals() caps via cap_proposals(&state.assertions, PROPOSAL_BUDGET).len(); test seeds parametrically against the canonical const. Red-loop correction: record_assertion overwrites the ledger per call, so the test now batches all assertion lines through one write_ledger call.
