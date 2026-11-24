---
tags: [pipeline-run:tdd-ro5u-2026-11-24-kaya-msh-wai-adapter-idempotent-re-import, pipeline-step:refactor]
---

REFACTOR kaya-msh: single tidy — LABEL_ID_REUSE promoted to pub const in src/ledger.rs (with doc) and the cmd_import collision guard now imports it instead of duplicating the 'kaya.ledger.id_reuse' literal, removing the silent-drift hazard if the ledger ever renames the label. No duplication left; names and boundaries already minimal (helper ~12 lines, one guarded branch). cargo test green after refactor (all targets ok, zero failures).
