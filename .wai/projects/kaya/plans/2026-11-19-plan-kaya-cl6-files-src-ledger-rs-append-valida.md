---
tags: [pipeline-run:tdd-ro5u-2026-11-19-kaya-cl6-supersedes-validation, pipeline-step:plan]
---

PLAN kaya-cl6. Files: src/ledger.rs (append validation helper + label const), tests/ledger.rs (red refusal tests first), no changes to resolve_effective. One agent: tests first (confirm red by narrow cargo test), then implement, then confirm green; orchestrator runs full gates. Then review/fix/ledger/ship per gates.
