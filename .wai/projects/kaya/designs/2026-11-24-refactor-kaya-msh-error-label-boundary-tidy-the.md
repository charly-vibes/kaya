---
tags: [pipeline-run:tdd-ro5u-2026-11-24-kaya-msh-wai-adapter-idempotent-re-import, pipeline-step:refactor]
---

REFACTOR kaya-msh: error-label boundary tidy. The cmd_import id-reuse guard matched the string literal 'kaya.ledger.id_reuse' because LABEL_ID_REUSE was private; promoted the const to pub in src/ledger.rs (doc comment added) and imported it in src/main.rs so guard and refusal site share one symbol — removes the silent-drift hazard if the ledger ever renames the label. Behavior unchanged; cargo test green across all targets after the refactor (34 suites ok, zero failures). This tidy goes in its own commit per the pipeline commit shape.
