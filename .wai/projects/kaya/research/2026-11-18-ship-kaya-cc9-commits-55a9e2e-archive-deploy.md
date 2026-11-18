---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-cc9-archive-and-deploy-document-taxonomy, pipeline-step:ship-close]
---

SHIP kaya-cc9 — commits 55a9e2e (archive/deploy) + ed9adef (evidence)

Ticket kaya-cc9 closed in bd; .beads/issues.jsonl exported in the feature commit. The change is archived as 2026-11-17-add-document-taxonomy; both layers are deployed in openspec/specs with contracts in .espectacular/; the book renders them through scripts/sync-specs.py. No push performed.

Verification at ship: ah check --run-tests 36 passed / 0 findings; ah report 35/35 covered; spk lint openspec/specs 2 files / 0 issues; openspec validate --specs --strict 2 passed; openspec validate --changes --strict 1 passed; cargo test 24 targets; just ci passed; build/publish idempotent.

Known advisory: pre-commit ddl-pretender-gate still reports file-length/function-length thresholds on pre-existing modules and tests (50 open violations, 0 new); scripts/sync-specs.py is clean.

Next: kaya-n50 overview cutover (user-approved scope: overview half only).
