---
tags: [pipeline-run:tdd-ro5u-2026-11-19-kaya-cl6-supersedes-validation, pipeline-step:green]
---

GREEN kaya-cl6: append validates supersedes via validate_supersedes before write — shape (nonempty string or nonempty array of nonempty strings), dangling targets, self-reference, cycles (iterative DFS over stored corrections) all refused with new label kaya.ledger.supersedes_invalid naming event id + target; byte-duplicate replay still returns Duplicate before validation. Green: ledger 31 passed, ledger_properties 2 passed, full suite 30 suites exit 0, clippy clean, fmt applied. resolve_effective untouched; silent-drop documented as hand-edited-files-only.
