---
tags: [pipeline-run:tdd-ro5u-2026-11-25-kaya-dar-pretender-target-exclude-glob-fix, pipeline-step:plan]
---

PLAN kaya-dar: single-line config change in pretender.toml exclude list - add './target/**' beside existing 'target/**' (keep both: direct-invocation 'pretender check target' produces un-prefixed paths that match the old glob; default '.'-rooted walks produce './'-prefixed paths matching only the new one). RED = current full scan emits ./target/** findings (verified: last full check lists 12+ target/package + target/doc yellows, 60 still_open delta). GREEN = full 'pretender check' after edit shows zero ./target findings, delta flips to fixed>=55 still_open<=5 new 0, exit code governed only by the 5 documented live yellows. No code motion, no threshold changes - caps stay at g41/g9u measured maxima. Verification: pretender check full + diff-only, cargo test -q (504), just record-verify.
