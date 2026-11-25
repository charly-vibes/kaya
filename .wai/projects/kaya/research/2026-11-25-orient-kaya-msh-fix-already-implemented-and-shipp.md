---
tags: [pipeline-run:tdd-ro5u-2026-11-24-kaya-msh-wai-adapter-idempotent-re-import]
---

ORIENT kaya-msh: fix already implemented and shipped - commit 2cb05bb (stable-identity dedup at the import seam: latest_stored_same_identity/same_observation skip + reobservation_correction supersede, src/commands/import.rs:44-125), regression tests land in tests/cli_pipeline.rs (reimport_of_unchanged_tree_appends_nothing :327, edited_record_yields_correction_never_id_reuse :342, reimport_of_drifted_dateless_artifact_appends_nothing :479), prior tdd-ro5u run completed full red/green/refactor/ro5u cycle with verdict READY criticals-0 (.wai/projects/kaya/reviews/ro5u-kaya-msh-wai-idempotent-reimport-criticals-0.md, 2026-11-25 artifact set). Ticket was stale: fix landed after ticket creation; no code changes needed from this run - verification only. Meter check: rerun the three regression tests + full suite now.
