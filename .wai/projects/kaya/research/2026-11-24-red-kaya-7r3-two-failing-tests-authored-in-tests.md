---
tags: [pipeline-run:tdd-ro5u-2026-11-24-kaya-7r3-incremental-regeneration, pipeline-step:red]
---

RED kaya-7r3: two failing tests authored in tests/cli_pipeline.rs and observed failing (0 passed / 2 FAILED; both exit at unknown clap flag --instrument): second_identical_build_performs_zero_render_invocations (second instrumented build must report render_invocations==0, recomputed==[], byte-identical docs/research tree) and new_event_recomputes_affected_projections (instrumented rebuild after identical build reports 0; after one appended event reports >0). Helper snapshot_docs_research (sorted path+sha256 walk). Green target: .kaya/build-state.json sidecar skip in cmd_build + RENDER_CALLS counter + --instrument envelope field.
