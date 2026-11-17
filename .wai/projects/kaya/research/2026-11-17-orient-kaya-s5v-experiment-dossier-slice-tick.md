---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-s5v-experiment-dossier-slice, pipeline-step:orient]
---

# ORIENT kaya-s5v — experiment dossier slice

Ticket: kaya-s5v, P1, lane:afk, complexity:l, base_commit df81656.
files: src/projection/mod.rs, src/projection/report.rs, src/main.rs.
Delta: openspec/changes/add-document-taxonomy/specs/kaya-documents/spec.md.
Layer: .../kaya-documents/layer.md (properties unknown_kind_refused,
intent_record_split, legacy_block_refused, empty_kind_silent).

## Acceptance restated as testable statements

1. Kind/family classification is closed: parse("experiment-report") and any
   value outside the 4 families / 11 kinds is refused with label
   kaya.documents.unknown_kind_refused, the message names the value and lists
   the closed family.kind vocabulary; no document is written.
2. A document that carries a generated block outside the vocabulary (e.g.
   generated:experiment-report) is refused before writing with label
   kaya.documents.legacy_block_refused naming the block and the block
   vocabulary; the file is byte-identical afterwards.
3. An experiment with a hypothesis AND a run renders two generated blocks:
   exploration.experiment-specification (only intent rows: Question,
   Hypothesis) and exploration.experiment-record (only observed rows: Runs,
   Observations, Findings, Interpretations); neither block contains the
   other's ids, and no row is rendered twice.
4. A kind with no source records emits no block and no placeholder row; the
   build result reports it once as {family, kind, reason:"no-records"};
   a previously written block for a now-empty kind is removed, not stubbed.
5. Human-authored prose and foreign managed blocks survive a rebuild byte for
   byte; rebuild with unchanged inputs writes nothing; the source map names
   every rendered row.
6. `cargo run -- build` over the committed ledger emits the new blocks for the
   dogfooded dossier, reports omitted kinds, and leaves `git diff` limited to
   the intended documents.

## Current code seams

- src/projection/mod.rs: scan/replace_generated/Summary; needs remove_generated
  for the omit-empty transition (no block, no markers, no separator newline).
- src/projection/report.rs: REPORT_BLOCK="experiment-report", single body
  renderer with `- _(none)_` placeholders, build() -> Summary.
- src/main.rs: cmd_build calls index::build then report::build per experiment,
  merges Summary, emits envelope keys out/recomputed/unchanged/stale/sources.
- docs/research/experiments/kaya-Experiment-add-core-research-record.md holds
  TWO stale `generated:experiment-report` blocks (lines 57-81, 83-107) — the
  known latent defect; the migration must converge to one spec + one record
  block and preserve authored prose.

## Blast radius of the rename (verified by grep)

tests/report.rs (8 hits), tests/acceptance.rs (8), tests/cli_pipeline.rs (5),
src/projection/report.rs (8), src/main.rs (1), committed dossier (5).

## Validation

narrow: cargo test --test report --test documents --test acceptance --test cli_pipeline
full: cargo test; ah check --changes add-document-taxonomy; openspec validate --changes --strict
baseline: cargo test 15 suites ok, 0 failed (this run, before changes).

