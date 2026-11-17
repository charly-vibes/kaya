---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-3f6-reuse-references, pipeline-step:refactor]
---

REFACTOR kaya-3f6: no behavior change. The two reuse kinds share one build loop; the only lint-driven tidy was collapsing a nested if into a let-chain in the omission path plus rustfmt. Structure checked: block rendering, validation and write-on-change paths are single-purpose; no duplication worth extracting. cargo test 192 green; clippy -D warnings clean; build idempotent.
