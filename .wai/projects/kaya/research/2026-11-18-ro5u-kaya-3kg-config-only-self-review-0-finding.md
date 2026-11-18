---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-3kg-record-hooks, pipeline-step:ro5u-review]
---

RO5U kaya-3kg (config-only self-review): 0 findings. Pre-commit invokes the read-only staged check with stale enforcement; pre-push invokes the same worktree check via just; neither runs build, publish, or capture. Commands verified in a temp record (clean 0, staged drift 1) and on this repo (`just record-verify` 0). Residual risk: a global core.hooksPath shim must remain in place for local hooks; CI parity lands in kaya-tm8.

