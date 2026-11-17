---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-3f6-reuse-references, pipeline-step:red]
---

RED kaya-3f6: cargo test --test reuse -> E0432 unresolved import kaya::projection::reuse (module absent as expected); tests/cli_pipeline.rs additions compile-fail on the same module. tests/reuse.rs has 6 tests (exact revision+command rows, exact handoff rows, empty-kind omission with no files, vocabulary names, ledger bytes/event count unchanged, legacy block refused before writing). Contract .espectacular/changes/add-document-taxonomy/kaya-documents/reuse-reference-only.toml bound to the reproduction test. No setup errors.
