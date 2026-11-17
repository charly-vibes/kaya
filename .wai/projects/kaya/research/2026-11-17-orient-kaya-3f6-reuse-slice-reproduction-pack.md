---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-3f6-reuse-references, pipeline-step:orient]
---

# ORIENT kaya-3f6 — reuse slice: reproduction-package and handoff

Ticket: kaya-3f6, P2, lane:afk, complexity:m, base_commit df81656,
files: src/projection/mod.rs, src/projection/reuse.rs.
Spec: kaya.documents requirement "Reuse families reference, never copy"
(scenarios "Brief adds no knowledge", "Reproduction package points at a
revision"); layer property reuse_by_reference; delta coverage scenario
reuse_reference_only.

## Acceptance as testable statements

1. Both kinds build from references the current adapters already capture:
   - reproduction-package: git revision references (Source nodes carry
     path@commit) plus any commands declared on Run nodes;
   - handoff: wai handoff artifact references (path).
2. The ledger's entity count is unchanged by a build: the projection writes
   markdown only and appends no events (graph built from the same events
   before and after; ledger bytes identical).
3. Every rendered item carries a source reference resolvable without kaya:
   a node id plus path@commit, a command with its run id, or a wai artifact
   path.
4. A kind with no references is omitted, no file is written, and the build
   reports it once; generated block names are reuse-handoff.reproduction-package
   and reuse-handoff.handoff and pass the shared vocabulary gate.
5. No new entity events, no embedded copies of referenced content, no spec
   edits; meter: cargo test --test reuse && openspec validate --changes --strict.

## Current code seams

- src/projection/mod.rs: scan/replace_generated/remove_generated,
  validate_generated_blocks, Summary.
- src/projection/report.rs: closed Family/Kind vocabulary, Omitted/Built,
  classify_block; the report owns exploration kinds only.
- src/main.rs cmd_build: read ledger -> model::build -> PromotionState ->
  index::build -> per-experiment report::build -> merge summary, omitted,
  stale; envelope keys out/recomputed/unchanged/stale/sources/omitted/
  pending_proposals/ledger_digest.
- Graph: nodes carry attributes from capture events. wai artifacts: path+kind
  (kind "handoffs" for handoff artifacts); commits: sha+message; runs:
  passed/failed/ignored/total; model Sources: path+commit.
- Adapters capture no command text today; commands are optional on Run nodes
  (attribute `command` string or `commands` array) and the projection renders
  them when present.

## Blast radius

New module src/projection/reuse.rs + registration in projection/mod.rs;
cmd_build wiring in src/main.rs; new tests/reuse.rs; tests/cli_pipeline.rs
build-envelope assertions; new documents docs/research/reproduction-package.md
and (fixture-driven only) handoff; contract
.espectacular/changes/add-document-taxonomy/kaya-documents/reuse-reference-only.toml;
tasks.md 3.5.

## Validation

narrow: cargo test --test reuse --test cli_pipeline
full: cargo test; ah check --changes add-document-taxonomy --run-tests; openspec validate --changes --strict
baseline: HEAD 41fc61a, 184 tests green, fmt/clippy clean, build idempotent.

