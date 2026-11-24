---
tags: [pipeline-run:tdd-ro5u-2026-11-24-kaya-g41-pretender-role-thresholds, pipeline-step:green]
---

GREEN kaya-g41: pretender.toml written (schema from wai sibling config: [thresholds] global + [thresholds.test] + [thresholds.script]; per-role abc unsupported). pretender-baseline.json reset to the 3 documented helper-blind-spot entries. Orchestrator-verified: check --mode gate -> 3 known (baselined), 0 new; second consecutive run -> 0 new WITHOUT --adopt; hook form check --diff-only --mode gate exit 0. Churn eliminated (was 23 new per commit). No src/test changes; cargo suite untouched (449/0 stands).
