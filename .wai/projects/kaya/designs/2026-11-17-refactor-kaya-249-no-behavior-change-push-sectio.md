---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-249-understanding-kinds, pipeline-step:refactor]
---

REFACTOR kaya-249: no behavior change. push_section now delegates to a shared push_heading(level) so experiment blocks keep ## and understanding blocks use ###; title/attribute helpers mirror index.rs semantics (single-line, trimmed, blank -> key). rustfmt applied. cargo test 231 green; clippy -D warnings clean; build idempotent.
