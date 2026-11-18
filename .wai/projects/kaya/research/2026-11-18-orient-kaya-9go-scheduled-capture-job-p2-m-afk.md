---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-9go-scheduled-capture, pipeline-step:orient]
---

ORIENT kaya-9go: scheduled capture job (P2, m, afk). Files: .github/workflows/record.yml, justfile.
Acceptance: one scheduled run appends capture events and commits products; a second run adds no duplicates. No capture in pre-commit or PR jobs; no thresholds.
Approach: record-capture recipe (import wai, import git, build, publish); record.yml on schedule/workflow_dispatch on main with contents:write, fetch-depth 0, commits and pushes the snapshot; idempotence comes from adapter event ids and ledger dedupe.

