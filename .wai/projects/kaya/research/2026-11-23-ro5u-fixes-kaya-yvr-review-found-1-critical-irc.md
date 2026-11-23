---
tags: [pipeline-run:tdd-ro5u-2026-11-23-kaya-yvr-document-ir-materialization, pipeline-step:fix-review]
---

RO5U-FIXES kaya-yvr: review found 1 critical — IrCollector::collect discarded its own labeled grammar-refusal (unwrap_or_default), so a corrupted document source yielded a silently-empty rendered list instead of a refusal. Fix committed ccc41d76e5: collect -> Result<(), Failure>, five call sites propagate '?', doc pins that the IR never silently drops a document's records; duplicated doc-comment lines removed. Behavior-impacting fix covered by existing tests: cache_outside_sources_leaves_ir_identical drives the full cmd_ir path (kaya ir) and fails loudly on the labeled refusal path; full cargo suite rerun green (0 FAILED), clippy -D warnings 0, just ci passed, ah check clean (exit 0).
