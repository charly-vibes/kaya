---
tags: [pipeline-run:tdd-ro5u-2026-11-25-kaya-w68-align-read-lists-so-the-exploration-brief-can-back-tutorial, pipeline-step:red]
---

RED kaya-w68: tests/explain.rs::brief_backed_tutorial_publishes — init, question, build, publish on a fresh ledger; asserts docs/public/tutorial.md exists. Fails at HEAD (0 passed, 1 failed) because brief.md is in neither read list, so read_documents never admits exploration.exploration-brief and tutorial omits.
