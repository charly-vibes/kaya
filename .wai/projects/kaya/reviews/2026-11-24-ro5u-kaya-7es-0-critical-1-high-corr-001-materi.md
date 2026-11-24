---
reviews: 2026-11-24-rule-of-5-review-kaya-7es-doctest-execution-co.md
verdict: needs-work
tags: [pipeline-run:tdd-ro5u-2026-11-24-kaya-7es-doctest-execution-contracts, pipeline-step:ro5u-review]
---

RO5U kaya-7es: 0 critical, 1 HIGH (CORR-001 materialize panics on malformed doctest fence; TypeSafe-verified @0.95, FP 0/1), 1 MEDIUM (EDGE-003 non-recursive walk misses experiments/*.md), 4 LOW. Fixes land in fix-review step: extend validate_generated_blocks with doctest::extract gate, recursive md walk, rename report shadow, grammar doc note, spawn-failure comment, layer table realign.
