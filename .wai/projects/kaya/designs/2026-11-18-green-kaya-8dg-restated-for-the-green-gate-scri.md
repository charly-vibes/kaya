---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-8dg-record-lag-gate, pipeline-step:green]
---

GREEN kaya-8dg (restated for the green gate): scripts/record-lag.py reads check facts and counts commits since the capture frontier over src docs openspec .wai; bound 25, --mode warn|fail; just recipe record-lag. Verified fail exit 1 at 53 commits, warn exit 0, py_compile clean.

