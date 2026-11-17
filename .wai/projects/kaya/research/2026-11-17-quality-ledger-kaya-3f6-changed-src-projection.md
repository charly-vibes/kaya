---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-3f6-reuse-references, pipeline-step:quality-ledger]
---

QUALITY LEDGER kaya-3f6

Changed:
- src/projection/reuse.rs (new): reuse-handoff.reproduction-package (### Sources path@commit from Source nodes, ### Commands from Run command/commands) and reuse-handoff.handoff (### Handoffs from wai handoff artifacts); inputs() with skip-on-absent and refuse-on-invalid semantics; single-line rows, identical-row dedupe, delete-on-empty, validation before any write, write only on change, omission reporting per kind.
- src/projection/mod.rs: shared pub(crate) document plumbing (generated_body/read_existing/write/io_error/LABEL_IO).
- src/projection/report.rs: uses the shared plumbing; private copies removed.
- src/main.rs: reuse always runs after the reports; summary and omitted merged; stale-authored flagging for both reuse documents.
- tests/reuse.rs (13), tests/acceptance.rs, tests/cli_pipeline.rs; contract reuse-reference-only bound to inputs_selects_references_from_the_graph (includes the ledger-unchanged clause).
- docs/research/reproduction-package.md (generated: six committed Sources at 43b606f; no commands because no adapter captures them yet).

Verified:
- cargo test: 199 passed, 0 failed (16 targets).
- cargo fmt --all --check clean; cargo clippy --all-targets --all-features -- -D warnings clean.
- cargo run -- build twice: recomputed [], unchanged [experiment-specification, orientation.research-map, reuse-handoff.reproduction-package], omitted [experiment-record, reuse-handoff.handoff]; ledger bytes untouched.
- ah check --changes add-document-taxonomy --run-tests: 6 passed, 27 structural (remaining tickets).

Review:
- rule-of-5-universal, five passes: 0 critical, 1 high, 5 medium, 6 low; all fixed except the deferred redundant rescan. The most serious finding: raw references could inject a kaya marker into the generated block and lock every later build out; rows are now single-line. The graph-assembly contract had no executing test; it now does, including the ledger clause.

Risks:
- No adapter records commands yet, so ### Commands is only reachable from hand-authored Run attributes (documented; adapter work outside this ticket).
- A valid block for the sibling reuse kind left inside the wrong document is not refused or cleaned (name-only vocabulary gate); tracked as future hardening with kaya-scq/cc9.

Next:
- Ship: tick tasks.md 3.5, close kaya-3f6, export beads, commit, then start kaya-qp8 (publications tracer bullet).
