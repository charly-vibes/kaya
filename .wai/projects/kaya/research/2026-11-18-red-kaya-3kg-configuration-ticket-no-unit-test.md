---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-3kg-record-hooks, pipeline-step:red]
---

RED kaya-3kg: configuration ticket, no unit test. The executable contract is the command itself: pre-commit `cargo run --quiet -- check --index --fail-on-stale-authored` and pre-push `just record-verify`. Reproduction plan: temp clone, stage a drifted generated block -> pre-commit command exits 1; restore -> exits 0; `just record-verify` on the repo exits 0. No source change yet.

