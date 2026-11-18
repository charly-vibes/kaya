---
reviews: 2026-11-18-ro5u-kaya-ntv-0-critical-0-high-1-medium-3-low.md
verdict: fail
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-ntv-kaya-check-assertions-stale, pipeline-step:ro5u-review]
---

RO5U kaya-ntv: 0 Critical, 0 High, 1 Medium, 3 Low. Verdict incorrect pending one shape fix.
MEDIUM 1: Stale findings omit document identity and are neither sorted nor deduped; authored-block ordinals repeat across documents, so data.stale is ambiguous and can double-count. Fix: add root-relative `path` to Stale, sort/dedup by (path, block, evidence).
LOW 2: no test for a satisfied requirement or a repeated flag.
LOW 3: the `absent` reason branch is untested.
LOW 4: index-mode parity for assertions/staleness is untested.

