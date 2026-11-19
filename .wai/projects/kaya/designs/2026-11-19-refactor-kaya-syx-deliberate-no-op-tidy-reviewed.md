---
tags: [pipeline-run:tdd-ro5u-2026-11-19-kaya-syx-turu-adapter, pipeline-step:refactor]
---

REFACTOR kaya-syx: deliberate no-op tidy. Reviewed src/adapters/turu.rs against the adapter family: it already mirrors wai.rs structure (single capture entry point, string_field/bundle_failure/entry_failure helpers, event_id mirroring the family formula, atomic parse-before-emit); fmt/clippy clean, no duplication inside the module. Cross-adapter extraction of the shared event_id/failure helpers was considered and deferred: it would edit three stable adapters (wai/git/testrunner) and their contract tests for zero behavior gain — recorded as an advisory, not ticket scope. Rationale kept: boring structure wins; the module stays reviewable next to its siblings.
