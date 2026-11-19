---
tags: [pipeline-run:tdd-ro5u-2026-11-19-kaya-nx4-hegel-property-tests, pipeline-step:red]
---

RED kaya-nx4: command=cargo test --test event_properties --test ledger_properties --test status_build_properties. Expected per synthesis (ticket kaya-152): all properties green on first execution — they characterize existing well-formed behavior; any failure is either a harness bug or a bug lead. Actual: event_properties 2/2 green, ledger_properties 2/2 green first run; status_build failed on four harness defects (no git init in seed; exit-code oracle hard-coded 2 instead of zero-iff-ok; listing oracle raced git add/commit bookkeeping and git auto-gc; anchor needle searched past the final newline where no bytes exist). All four fixed as harness fixes — zero product behavior implicated, matching the synthesis prediction that the scoped properties are green. Bug leads from research are pre-ticketed (cl6, 8k5, 1y9, 8tn, g11, 8yw, 7ma, sh6) and none fired.
