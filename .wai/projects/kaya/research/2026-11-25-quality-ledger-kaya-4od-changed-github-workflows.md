---
tags: [pipeline-run:tdd-ro5u-2026-11-25-kaya-4od-scheduled-record-capture-races-in-flight-work, pipeline-step:quality-ledger]
---

QUALITY LEDGER kaya-4od: changed .github/workflows/record.yml (push trigger removed + rationale comment), justfile (record-capture flock+dirty-guard shebang recipe; lag bound 25->60 x2), .gitignore (+capture.lock), scripts/record-conflicts.md (new union recipe, corrected rebase stage semantics). Oracles: SKIP visible exit 0 on dirty main tree with zero mutations; check-ignore ok; record-check fail ok at 60; cargo test 510/0; fmt clean; workflow yaml exercised only at nightly (not runtime-verifiable locally - noted as EXCL-001 residual: no CI assertion pinning the trigger set or guard). Review READY CRITICALS=0 after NEEDS_REVISION round (4 fixed, 1 accepted).
