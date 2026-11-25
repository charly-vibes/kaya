---
tags: [pipeline-run:tdd-ro5u-2026-11-25-kaya-eaj-session-review-remediation-four-findings, pipeline-step:refactor]
---

REFACTOR kaya-eaj: EXCL-001 is the refactor half - the authored-block name bridge previously lived in two places (impact.rs ir_block_name comment-knowledge + publication.rs inline names block with its own duplicate comment); now one pub helper pair kaya::impact::ledger_block_names/ir_block_name carries both directions, publication.rs record_trail consumes ledger_block_names and the stale inline comment is gone. No behavior change: str-comparison adapted to Vec<String> via iter().any; suite green 516/516, fmt+clippy clean.
