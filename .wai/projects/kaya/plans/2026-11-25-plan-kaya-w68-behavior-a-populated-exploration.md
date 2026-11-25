---
tags: [pipeline-run:tdd-ro5u-2026-11-25-kaya-w68-align-read-lists-so-the-exploration-brief-can-back-tutorial, pipeline-step:plan]
---

PLAN kaya-w68 — Behavior: a populated exploration.exploration-brief block in docs/research/brief.md is admissible for tutorial (admission row tutorial <- experiment-record + exploration-brief); publish emits tutorial.md. Out of scope: kaya-34g record seeding, renderer changes, collections. RED test first: tests/explain.rs::brief_backed_tutorial_publishes — fresh ledger, question, build, publish, assert docs/public/tutorial.md exists; fails at HEAD. Narrow: cargo test --test explain brief_backed_tutorial_publishes. Full: cargo test + just record-check + census==export. Files: src/publication/source.rs, src/verification/mod.rs, src/explain.rs, tests/explain.rs, docs/public regenerated. Commit shape: single feature commit (code+tests+docs atomic); separate tidy commit only if extraction warrants.
