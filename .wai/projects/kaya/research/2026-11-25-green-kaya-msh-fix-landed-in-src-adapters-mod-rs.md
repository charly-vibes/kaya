---
tags: [pipeline-run:tdd-ro5u-2026-11-25-kaya-msh-wai-adapter-idempotent-re-import, pipeline-step:green]
---

GREEN kaya-msh: fix landed in src/adapters/mod.rs (stable_identity/same_observation/latest_stored_same_identity/root_observation helpers, wai-shaped only via data.path string gate) + src/commands/import.rs (capture.*-gated dedup block before ledger.append: same identity + same data-minus-supersedes -> duplicates+=1 skip; same identity + changed content -> root-superseding generation-salted reobservation_correction with salt inputs identical to the id_reuse arm). Verified: cli_pipeline 41/0 including the two new red->green tests (reimport_of_drifted_dateless_artifact_appends_nothing; edited_dateless_artifact_supersedes_previous_observation) and all pinned dated re-import behaviors byte-identical; FULL suite 506 passed, 0 failed (504 + 2 new).
