---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-8dg-record-lag-gate, pipeline-step:red]
---

GREEN kaya-8dg: scripts/record-lag.py reads one `kaya check` envelope (facts, no ledger parser), counts `git rev-list --count <frontier>..HEAD -- src docs openspec .wai`, and compares to --bound (default 25). --mode warn exits 0 above the bound; fail exits 1. justfile recipe `record-lag mode="fail" bound="25"`. Verified: repo reports 53 commits since 645a6b5 (bound 25), fail mode exit 1, warn mode exit 0; py_compile clean; no re-import, no parser.

