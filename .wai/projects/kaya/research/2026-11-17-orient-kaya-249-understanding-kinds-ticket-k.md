---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-249-understanding-kinds, pipeline-step:orient]
---

# ORIENT kaya-249 — understanding kinds

Ticket: kaya-249, P2, lane:afk, complexity:m, base_commit df81656,
files: src/projection/report.rs.
Spec: kaya.documents "Family-scoped generated blocks" (scenario Block content
limited to its kind) and the taxonomy table rows understanding.findings-and-
analysis (Finding, Claim, Observation, Run) and understanding.decision-record
(Decision, Source, Finding).

## Acceptance as testable statements

1. understanding.findings-and-analysis renders only admissible records:
   Finding, Observation and Claim rows. It never contains run rows (ticket
   anti-goal), artifact rows, or rows from another family.
2. understanding.decision-record renders Decision rows and their evidence
   references: Source nodes as path@commit and Findings linked to the decision
   (influences edges). No other families.
3. Both blocks are project-level (one document, docs/research/understanding.md),
   named understanding.findings-and-analysis and understanding.decision-record,
   validated before writing, written only on change, and omitted when empty
   (no block, no placeholder, one no-records entry each).
4. Authored and foreign bytes survive; a second build is unchanged; a legacy
   generated block is refused before writing.
5. Build summary coverage lists both kinds when present and reports them
   omitted when absent.

## Current code seams

- src/projection/report.rs: Family/Kind vocabulary, Omitted/Built, build for
  per-experiment documents, render_specification/render_record, push_section,
  linked(), validate_generated_blocks, shared plumbing.
- src/main.rs cmd_build: index -> per-experiment reports -> reuse -> envelope
  merge (recomputed/unchanged/stale/sources/omitted).
- Graph: EntityType {Finding, Claim, Observation, Decision, Source, Run, ...};
  Relationship::Influences (Decision -> Finding fixed pair), Derives.
- docs/research/ holds index.md, reproduction-package.md, experiments/*.

## Validation

narrow: cargo test --test understanding --test cli_pipeline
full: cargo test; ah check --changes add-document-taxonomy --run-tests;
openspec validate --changes --strict
baseline: HEAD d6df368, 225 tests green, fmt/clippy clean, build/publish idempotent.

