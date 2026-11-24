---
tags: [pipeline-run:tdd-ro5u-2026-11-24-kaya-y5v-exploration-brief-generator, pipeline-step:red]
---

RED kaya-y5v: three failing tests authored in tests/cli_pipeline.rs and observed failing (cargo test --test cli_pipeline: 32 passed, 3 FAILED): build_omits_exploration_brief_without_question_records (run-only ledger -> expects omitted {exploration, exploration-brief, no-records}), build_renders_exploration_brief_from_question_idea_hypothesis (Q+I+H seed -> expects docs/research/brief.md with block marker + ## Question/## Idea/## Hypothesis id sections, no omission), stale_brief_without_records_is_removed (brief removed after records leave). First panic: 'the first build renders the exploration brief'. Green plan wired into the dispatch (render_brief entity_rows over Question/Idea/Hypothesis, plan/build via plan_project_document family, cmd_build + collect_document_ir wiring, contract-test updates).
