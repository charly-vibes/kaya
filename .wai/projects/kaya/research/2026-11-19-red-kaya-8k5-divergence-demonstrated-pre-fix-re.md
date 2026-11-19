---
tags: [pipeline-run:tdd-ro5u-2026-11-19-kaya-8k5-resolve-alignment, pipeline-step:red]
---

RED kaya-8k5: divergence demonstrated pre-fix — resolve kept self-superseding events while resolve_effective dropped them; resolve had zero production callers (tests/ledger.rs only). Expected failure = deletion of the divergent surface; red confirmed by the two repointed tests previously asserting resolve() semantics (correction-kept vs effective-patched identity).
