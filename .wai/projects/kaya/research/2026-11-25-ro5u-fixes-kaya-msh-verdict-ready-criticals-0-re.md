---
tags: [pipeline-run:tdd-ro5u-2026-11-25-kaya-msh-wai-adapter-idempotent-re-import, pipeline-step:fix-review]
---

RO5U-FIXES kaya-msh: verdict READY CRITICALS=0 (review attached to green artifact 2026-11-25-green-kaya-msh-fix-landed-in-src-adapters-mod-rs.md). Three P3 findings, dispositions: (1) maintainability stored-invariant comment - FIXED inline, comment added at src/commands/import.rs documenting the one-event-per-identity-per-run adapter invariant; (2) parsed-data-equality skip narrowing the byte-refusal path for hand-edited stored lines - ACCEPTED as-is: consistent with the feature's same-facts model, trigger requires manual ledger editing; (3) identity rescan allocation O(events x ledger lines) - ACCEPTED as-is: dominated by ledger.append's existing full-file reload, JSONL scale fine. Post-fix gates: full suite 506/0, clippy 0 warnings, fmt clean.
