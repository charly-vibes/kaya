---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-ntv-kaya-check-assertions-stale, pipeline-step:quality-ledger]
---

QUALITY LEDGER kaya-ntv
Changed: src/verification/assertions.rs (new: request validation, Required, Stale with path, required(), stale()); src/verification/mod.rs (CheckOptions, required/stale Outcome, enforcement-aware exit); src/main.rs (repeatable --require-kind/--require-source, --fail-on-stale-authored, envelope required/stale); tests/verification.rs (9 assertion/stale tests); 6 TOMLs.
Verified: verification suite 32/32; full cargo test; clippy -D warnings; fmt; ah check --changes add-record-verification --run-tests → 60 passed, 0 findings (all change scenarios now bound and green); just ci green.
Review: RO5U 0 Critical, 0 High, 1 Medium, 3 Low; all fixed.
Risks: acknowledgments remain empty until kaya-7hm; hooks/CI not yet wired.
Next: kaya-7hm cites validation, supersede-only trigger, acknowledge command.

