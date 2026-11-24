---
tags: [pipeline-run:tdd-ro5u-2026-11-24-kaya-y5v-exploration-brief-generator, pipeline-step:green]
---

GREEN kaya-y5v: render_brief (entity_rows over Question/Idea/Hypothesis, None when all empty) at src/projection/report.rs:845-870, plan_brief :488-491 via plan_project_document, build_brief :493-498 (shared empty-doc deletion), BRIEF_BLOCK const :34; cmd_build wiring src/main.rs:893-897 (brief.md beside overview, stale-authored flag), collect_document_ir :1029-1032 (build/check parity). Contract-test updates: two omission fixtures gained the sorted exploration-brief no-records entry (tests/cli_pipeline.rs:1252-1256,:1576-1580); verification suite needed no changes (assertions non-exact). Orchestrator-verified: cargo test full suite 438 passed / 0 failed (3 new + 2 updated fixtures).
