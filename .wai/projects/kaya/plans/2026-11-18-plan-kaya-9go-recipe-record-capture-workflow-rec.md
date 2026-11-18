---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-9go-scheduled-capture, pipeline-step:plan]
---

PLAN kaya-9go: recipe record-capture; workflow Record on cron + dispatch, checkout full, cargo build, run recipe, configure identity, commit only if changes, push. Verify in a temp repo: first capture appends events and commits; second capture adds zero ledger lines and no commit. Does not run on pull_request.

