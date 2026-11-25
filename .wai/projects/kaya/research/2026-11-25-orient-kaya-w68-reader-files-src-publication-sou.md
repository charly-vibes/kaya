---
tags: [pipeline-run:tdd-ro5u-2026-11-25-kaya-w68-align-read-lists-so-the-exploration-brief-can-back-tutorial, pipeline-step:orient]
---

ORIENT kaya-w68: READER_FILES (src/publication/source.rs:58-65) and RESEARCH_READ_ORDER (src/verification/mod.rs:64-71) are identical 6-entry lists, both omitting brief.md; docs/research/brief.md carries a populated exploration.exploration-brief block, so tutorial can never compose it and kaya-dsv special-cased product_emitted to filter exploration-brief (src/explain.rs) plus a structural note that becomes false after this fix. Change: add brief.md to both lists, drop the filter + note, flip the agreement test, regenerate docs. Full artifact: .wai/projects/kaya/research/2026-11-25-orient-kaya-w68-brief-md-is-in-neither.md
