---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-s5v-experiment-dossier-slice, pipeline-step:quality-ledger]
---

QUALITY LEDGER kaya-s5v

Changed:
- src/projection/report.rs: closed Family(4)/Kind(11) vocabulary, block_vocabulary/family_vocabulary, refuse_kind, classify_block, Omitted/Built, split build (exploration.experiment-specification from Question/Hypothesis; exploration.experiment-record from Runs/Observations/Findings/Interpretations), empty-group and empty-kind omission, obsolete-block removal, duplicate-name refusal, kind-order normalization.
- src/projection/mod.rs: remove_generated (marker+body+one separator, byte-preserving) and public LABEL_MANAGED_GRAMMAR.
- src/main.rs: consume Built, merge/sort/dedupe omitted into the build envelope, tolerate a legitimately absent report when flagging stale authored blocks.
- tests/documents.rs (new, 12 tests) and migrated tests/report.rs (14), tests/acceptance.rs (2), tests/cli_pipeline.rs (19).
- .espectacular/changes/add-document-taxonomy/kaya-documents/: 4 contracts bound to the ticket's four scenarios.
- docs/research/experiments/kaya-Experiment-add-core-research-record.md: legacy duplicate blocks replaced by the specification block; record reported omitted; authored prose updated to the split. docs/src/document-taxonomy.md status paragraph updated.

Verified:
- cargo test: 174 passed, 0 failed (16 targets).
- cargo fmt --all --check: clean.
- cargo clippy --all-targets --all-features -- -D warnings: clean.
- cargo run -- build: idempotent; recomputed [], unchanged [exploration.experiment-specification:..., research-index], omitted [{exploration, experiment-record, no-records}].
- ah check --changes add-document-taxonomy --run-tests: 4 passed (this ticket's scenarios), 29 structural (other tickets' unbound scenarios).
- openspec validate: unchanged specs; change validity checked by ah.

Review:
- rule-of-5-universal, 5 parallel passes (spec conformance, invalid states, error boundaries, cost/determinism, test quality): 0 critical, 4 high, 5 medium, 6 low. All high and medium fixed except M1 (index writer), which is kaya-i03's subject; lows fixed except the two recorded design notes.

Risks:
- The index projection still writes research-index and does not run the vocabulary gate: the closed-block invariant is unenforced for docs/research/index.md until kaya-i03 (next run) renames it to orientation.research-map and shares the validator.
- Per-document vs per-block classification reading: the delta says a document carries one family/kind; this implementation classifies each generated block and one dossier carries two kinds. Recorded for a spec clarification; no code behaviour depends on the disputed reading beyond block naming.
- Promotion assertions remain ledger-global rows in the record block; they no longer create records for run-less experiments, but scoping interpretations to an experiment is still open (pre-existing).

Next:
- Ship: tick tasks.md 3.1, close kaya-s5v, export beads, commit, then start kaya-i03 (research map) in the next tdd-ro5u run.
