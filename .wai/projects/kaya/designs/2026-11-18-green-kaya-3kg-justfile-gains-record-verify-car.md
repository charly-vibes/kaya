---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-3kg-record-hooks, pipeline-step:green]
---

GREEN kaya-3kg: justfile gains `record-verify: cargo run --quiet -- check`; lefthook pre-commit gains `record-verify-staged: cargo run --quiet -- check --index --fail-on-stale-authored` outside the ddl-managed block; pre-push gains `record-verify: just record-verify`. Verified in a temp record: clean staged tree exit 0, staged drifted research-map block exit 1 naming the block; `just record-verify` on the repo exit 0. Hooks never run build/publish/capture.

