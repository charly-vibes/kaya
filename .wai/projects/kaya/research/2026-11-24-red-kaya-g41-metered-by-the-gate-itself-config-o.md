---
tags: [pipeline-run:tdd-ro5u-2026-11-24-kaya-g41-pretender-role-thresholds, pipeline-step:red]
---

RED kaya-g41 (metered by the gate itself, config-only ticket — no cargo test applies): failing state observed pre-change — defaults flagged ~900 units, baseline churn 23 new/commit (observed live at 94c8740 and 0291837 ship attempts), 126 stale baseline entries. Red evidence: pretender check --human pre-config output retained in orient artifact (hotspots: 1416 function_lines_max in src, 1316 in tests).
