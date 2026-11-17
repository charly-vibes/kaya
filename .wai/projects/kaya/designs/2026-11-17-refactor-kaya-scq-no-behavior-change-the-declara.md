---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-scq-status-release-architecture, pipeline-step:refactor]
---

REFACTOR kaya-scq: no behavior change. The declaration list stays a single named constant with includes read from admission(); tests/publications_export.rs reuses the existing fixture helpers; rustfmt and two clippy auto-deref fixes applied. cargo test 256 green; clippy clean.
