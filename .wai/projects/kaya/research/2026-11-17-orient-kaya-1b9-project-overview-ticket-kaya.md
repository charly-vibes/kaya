---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-1b9-project-overview, pipeline-step:orient]
---

# ORIENT kaya-1b9 — project overview

Ticket: kaya-1b9, P2, lane:afk, complexity:m, base_commit df81656,
files: src/projection/report.rs, src/projection/mod.rs.
Spec: kaya.documents orientation.project-overview (Project, Question, current
Findings) and "Family-scoped generated blocks"; research-map is owned by
kaya-i03 and already done.

## Acceptance as testable statements

1. orientation.project-overview renders only Project, Question and Finding
   rows; it never renders commit or artifact capture rows, runs, or records
   from another family.
2. Sections with no records are omitted; the kind is omitted entirely (no file,
   no block, one no-records entry) when all three are empty.
3. The block is named orientation.project-overview, validated before writing,
   written only on change, and a whitespace-only document is deleted.
4. Authored and foreign bytes survive; a second build is unchanged; legacy
   blocks are refused; authored citations get the stale-evidence audit.
5. The compiled document backs publication products: after build, `kaya
   publish` emits overview from it (so project-site links overview) and the
   committed docs/public reflects that.

## Current code seams

- src/projection/report.rs: build_understanding (project-level template),
  build for experiment kinds, shared helpers in projection::mod,
  validate_generated_blocks, Omitted/Built.
- src/main.rs cmd_build: index -> experiment reports -> understanding -> reuse
  -> envelope; stale flagging per document.
- src/publication/mod.rs::read_documents: fixed file list (index, understanding,
  reproduction, handoff); add overview.md.
- Committed ledger: two Question nodes, no Project/Finding -> overview renders
  Questions only and creates docs/research/overview.md.

## Validation

narrow: cargo test --test overview --test cli_pipeline --test publications
full: cargo test; cargo run -- build/publish; ah check --changes add-document-taxonomy --run-tests; openspec validate --changes --strict
baseline: HEAD 8f5ffb4, all suites green, fmt/clippy clean.

