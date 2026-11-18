---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-3kg-record-hooks, pipeline-step:orient]
---

ORIENT kaya-3kg: record hooks (P1, complexity m, lane afk). Files: lefthook.yml, justfile. Depends kaya-585/ntv (closed).
Acceptance: pre-commit runs `cargo run --quiet -- check --index --fail-on-stale-authored`; a deliberately drifted staged block fails it; a clean staged tree passes; pre-push runs `just record-verify` (same check command); hooks never run build, publish, or capture.
Verification: reproduce the exact commands in a temp fixture (stage drift -> exit 1; restore -> exit 0); inspect `git config core.hooksPath` note (global shim; never lefthook install locally).

