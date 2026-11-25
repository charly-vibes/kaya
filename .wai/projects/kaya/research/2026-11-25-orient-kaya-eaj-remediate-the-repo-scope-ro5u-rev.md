---
tags: [pipeline-run:tdd-ro5u-2026-11-25-kaya-eaj-session-review-remediation-four-findings, pipeline-step:orient]
---

ORIENT kaya-eaj: remediate the repo-scope ro5u review's four findings (READY CRITICALS=0). CLAR-001: reader page in docs/src (reference.md is generated, untouchable) + spec deltas in kaya-impact (reviewed binds digest) and kaya-projection (walk surface, generators = existing l46 tests); just validate must stay green. EDGE-001: record.yml retry loop - branch on .git/rebase-merge after failed pull, name the runbook, drop final sleep. EDGE-002: cmd_disposition dedup ignores candidate digest when stored legacy reviewed event lacks it; red = clone-strip-append fixture test. EXCL-001: pub helper ledger_block_names in kaya::impact used by ir_block_name + record_trail; comment made true. EDGE-003 (dogfood ack) explicitly out of scope.
