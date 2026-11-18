---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-585-kaya-check, pipeline-step:refactor]
---

REFACTOR kaya-585: no additional tidy beyond the planning split. The pure plan/write separation already removes duplication, the comparison core is one function shared by worktree modes, and clippy/fmt are clean. Corrective green change: publication comparison now always plans (missing publish tree = drift), compares omitted-product removals and the export through the same Source, and the test fixture publishes before commit. cargo test all suites green.

