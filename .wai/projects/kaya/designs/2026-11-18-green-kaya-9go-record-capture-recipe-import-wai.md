---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-9go-scheduled-capture, pipeline-step:green]
---

GREEN kaya-9go: record-capture recipe (import wai, import git, build, publish) and .github/workflows/record.yml on cron/dispatch for main with contents: write, full checkout, commit-if-changed and push. Verified idempotence in a temp repo: repeat capture adds zero events, ids unique.

