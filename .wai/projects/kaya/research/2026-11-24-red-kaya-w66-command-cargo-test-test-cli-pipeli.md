---
tags: [pipeline-run:tdd-ro5u-2026-11-24-kaya-w66-cap-proposals-production-caller, pipeline-step:red]
---

RED kaya-w66: command=cargo test --test cli_pipeline status_caps_pending_proposals_at_the_budget. Test written by subagent with local const PROPOSAL_BUDGET=5 (canonical const cannot exist pre-green since src/ is frozen in red): seeds budget+1 proposed assertions via record_assertion, asserts status pending_proposals == budget. Expected failure: status reports the full uncapped count (budget+1) today because cap_proposals has no production caller. Espectacular binding appended to .espectacular/kaya-promotion/pending-proposals-bounded-checked.toml.
