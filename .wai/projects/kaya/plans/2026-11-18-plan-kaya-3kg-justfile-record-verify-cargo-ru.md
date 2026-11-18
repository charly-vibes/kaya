---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-3kg-record-hooks, pipeline-step:plan]
---

PLAN kaya-3kg
- justfile: `record-verify: cargo run --quiet -- check` (read-only verifier).
- lefthook.yml: pre-commit command `record-verify-staged: cargo run --quiet -- check --index --fail-on-stale-authored` outside the ddl-managed block; pre-push `record-verify: just record-verify` outside the managed block. No build/publish/capture.
- Verification: in a temp copy of the repo, run the exact pre-commit command over (a) a clean staged tree (exit 0) and (b) a staged drifted generated block (exit 1); run `just record-verify` on the real repo (exit 0). No permanent config-text tests.
Commit: chore(gates): run read-only record verification in hooks. Closes kaya-3kg.

