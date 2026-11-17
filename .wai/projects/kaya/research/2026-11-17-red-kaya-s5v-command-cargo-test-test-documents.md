---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-s5v-experiment-dossier-slice, pipeline-step:red]
---

RED kaya-s5v: command=cargo test --test documents --test report. Failure is exactly the missing contract: E0432 unresolved imports (Block/Block families, Family, Kind, Omitted, block_vocabulary, classify_block, refuse_kind, LABEL_*, REASON_NO_RECORDS) and E0609 no field summary/omitted on Summary. No setup or fixture error; tests/documents.rs (6 tests) and migrated tests/report.rs, tests/acceptance.rs, tests/cli_pipeline.rs are in place. 4 espectacular contracts created under .espectacular/changes/add-document-taxonomy/kaya-documents/ (ah check --changes now reports 25 no-toml, down from 29; our four are recognized).
