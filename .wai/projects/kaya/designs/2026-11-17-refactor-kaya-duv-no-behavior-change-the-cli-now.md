---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-duv-developer-products, pipeline-step:refactor]
---

REFACTOR kaya-duv: no behavior change. The CLI now reads admission rows from publication::admission instead of restating them, and the two declaration lists are named constants; rustfmt applied. cargo test 251 green; clippy clean; publish idempotent.
