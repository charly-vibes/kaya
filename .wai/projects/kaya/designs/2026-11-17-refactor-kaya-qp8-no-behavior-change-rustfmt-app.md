---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-qp8-publications-tracer, pipeline-step:refactor]
---

REFACTOR kaya-qp8: no behavior change. rustfmt applied; the module separates vocabulary/tables (pure data), document reading, and build/apply, and the CLI wrapper reuses the same render path. Two test-side clippy lints fixed by removing redundant into_iter calls. cargo test 207 green; clippy -D warnings clean; publish idempotent.
