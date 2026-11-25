---
tags: [pipeline-run:tdd-ro5u-2026-11-25-kaya-w68-align-read-lists-so-the-exploration-brief-can-back-tutorial, pipeline-step:green]
---

GREEN kaya-w68: added brief.md to READER_FILES (src/publication/source.rs) and RESEARCH_READ_ORDER (src/verification/mod.rs), keeping the pinned parity between the two lists; removed product_emitted's exploration-brief exclusion and the contract note (src/explain.rs). Tests: new brief_backed_tutorial_publishes end-to-end (init/question/build/publish -> tutorial.md) with git-seeded ledger for the uncommitted-history gate; agreement test flipped to emitted with note removed; eight cli_pipeline expectations updated to the new truth (tutorial emits on question-only ledgers, brief joins the compiled-document set). Full suite 502 passed, 0 failed; fmt + clippy -D warnings clean.
