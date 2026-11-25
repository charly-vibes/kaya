---
tags: [pipeline-run:tdd-ro5u-2026-11-25-kaya-9w6-explain-refuses-wrong-typed-attributes, pipeline-step:refactor]
---

REFACTOR kaya-9w6: extracted kind_refusals(graph, kinds) helper replacing the duplicated flat_map in product_explain and both census loops; fixed a doc comment placement slip from green. cargo fmt clean, cargo test 503 passed 0 failed, clippy -D warnings clean.
