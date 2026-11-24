---
tags: [pipeline-run:tdd-ro5u-2026-11-24-kaya-w66-cap-proposals-production-caller, pipeline-step:green]
---

GREEN kaya-w66 decision: budget is a named shared constant — pub const PROPOSAL_BUDGET: usize = 16 in src/promotion.rs (doc: one session-close's approval budget per pending_proposals_bounded; constant so the CLI wire value and tests share one symbol; 16 gives multi-adapter sessions headroom while keeping the surfaced queue small). pending_proposals() in src/main.rs swaps state.pending_count() for cap_proposals(&state.assertions, promotion::PROPOSAL_BUDGET).len() — the ranking+capping semantics ship through the status point of use only (proposal_surfacing_pulled: counts only, no prompting; dossier paths deliberately untouched per ticket scope). Test imports the canonical const instead of its local stand-in (local const 5 was red-step scaffolding), so the seeded count and expected cap stay parametric.
