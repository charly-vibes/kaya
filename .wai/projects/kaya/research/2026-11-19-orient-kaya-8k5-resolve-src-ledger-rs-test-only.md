---
tags: [pipeline-run:tdd-ro5u-2026-11-19-kaya-8k5-resolve-alignment, pipeline-step:orient]
---

ORIENT kaya-8k5: resolve (src/ledger.rs, test-only) diverges from resolve_effective on self-superseding events (resolve keeps, resolve_effective drops) and has no production callers (tests/ledger.rs only). cl6 now refuses self-reference at append, making stored self-refs hand-edited-files-only. Fix: DELETE resolve and its divergent tests; resolve_effective is the one resolver. No new behavior.
