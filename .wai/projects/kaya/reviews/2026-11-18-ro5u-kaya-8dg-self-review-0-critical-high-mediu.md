---
reviews: 2026-11-18-ro5u-kaya-8dg-self-review-0-critical-high-mediu.md
verdict: pass
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-8dg-record-lag-gate, pipeline-step:green]
---

RO5U kaya-8dg (self-review): 0 critical/high/medium. The gate reads only `kaya check` facts and owns the bound and mode; a missing frontier or unreachable sha fails loudly in fail mode and warns in warn mode; no re-import, no threshold inside kaya. Verified 53>25 exit 1, warn exit 0, py_compile. Residual risk: the recipe shells `cargo run`, so CI pays a build; acceptable for the PR/main gate.

