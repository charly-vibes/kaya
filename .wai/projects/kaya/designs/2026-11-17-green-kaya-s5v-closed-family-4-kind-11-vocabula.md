---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-s5v-experiment-dossier-slice, pipeline-step:green]
---

GREEN kaya-s5v: closed Family(4)/Kind(11) vocabulary in src/projection/report.rs (block_vocabulary, refuse_kind -> kaya.documents.unknown_kind_refused, classify_block -> kaya.documents.legacy_block_refused); build() now validates existing generated block names before writing, renders exploration.experiment-specification (Question/Hypothesis) and exploration.experiment-record (Runs/Observations/Findings/Interpretations), skips empty groups, omits empty kinds and removes obsolete blocks via new projection::remove_generated; summary keys are <block>:<experiment>; main.rs merges omitted and adds envelope key omitted. Dossier migrated: both legacy experiment-report blocks removed byte-preservingly, spec block regenerated, record reported omitted (no-records). Verified: 164 tests pass; cargo clippy -D warnings clean; cargo fmt --check reports formatting only.
