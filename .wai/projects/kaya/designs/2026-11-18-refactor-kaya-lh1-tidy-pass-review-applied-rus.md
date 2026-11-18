---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-lh1-reuse-families-technical-reference-knowledge-brief, pipeline-step:refactor]
---

REFACTOR kaya-lh1 — tidy pass review

Applied: rustfmt only (line wrapping in src/projection/reuse.rs and tests/reuse_adapters.rs). No behavior change; targeted suites re-run green (reuse 13, reuse_adapters 8).

Candidates examined and rejected as churn:
- Folding `documents()` into a `(Kind, PathBuf)` API so build derives order instead of destructuring: would change the public shape used by src/main.rs and the tests while the explicit four-call build keeps validation-before-write obvious. No duplication beyond four parallel calls, and each kind has a distinct renderer, so a table would add indirection only.
- Collapsing the three `SourceRef` pushes in `inputs()` behind a helper: each selection reads clearly and the helper would obscure the independent-attribute rule.
- Sharing test helpers across reuse.rs and reuse_adapters.rs: integration tests cannot share modules without a common harness; duplicating four tiny helpers is cheaper than a new fixture crate.

Clippy is clean on all targets with -D warnings; no dead code, no unused public items.
