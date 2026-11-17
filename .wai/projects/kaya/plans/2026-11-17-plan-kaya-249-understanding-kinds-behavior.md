---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-249-understanding-kinds, pipeline-step:plan]
---

# PLAN kaya-249 — understanding kinds

## Behavior (spec ids)

- kaya.documents/Family-scoped generated blocks: understanding.findings-and-
  analysis renders only Finding, Observation, Claim rows; understanding.
  decision-record renders Decision rows plus evidence references (Source
  path@commit, Findings linked by `influences`). No run rows in either kind
  (ticket anti-goal).
- kaya.documents/Empty kinds omitted: each kind omitted with no-records when
  empty, reported once.
- Bound scenario: block-content-limited-to-its-kind.

## Out of scope

other families; per-experiment changes; publication changes; spec edits.

## Contract (binding)

src/projection/report.rs additions:

```rust
pub const FINDINGS_BLOCK: &str = "understanding.findings-and-analysis";
pub const DECISION_BLOCK: &str = "understanding.decision-record";
pub fn build_understanding(graph: &Graph, path: &Path) -> Result<Built, ProjectionError>;
```

- One project-level document at `path` (docs/research/understanding.md).
- findings block body: `### Findings` rows `- <id> — <title-or-key>`,
  `### Observations` rows `- <id> — <title-or-key>`, `### Claims` rows
  `- <id> — <title-or-key>`; only non-empty sections; sorted by id. No Runs,
  no Artifact rows, no experiment content.
- decision block body: `### Decisions` rows `- <id> — <title-or-key>`, then
  `### Evidence` rows: for each Source node `- <id> — <path>@<commit>`, and for
  each Finding reachable from a Decision through `influences`
  `- <finding-id> — see <decision-id>`; sorted by id then reference.
- Empty body -> kind omitted (no block; remove an existing block), one
  Omitted{understanding, <kind>, no-records}. Summary keys are the block names;
  sources map block -> rendered ids. Validation (validate_generated_blocks)
  before any write; write only on change; shared plumbing reused.
- main.rs: call build_understanding after the report loop, merge summary and
  omitted.

## Tests to write first (tests/understanding.rs)

1. `findings_and_analysis_renders_admissible_records_only` — Finding,
   Observation, Claim, Run, Artifact nodes; block lists f/o/c rows, contains no
   run id, no `### Runs`, no artifact id.
2. `decision_record_renders_decisions_and_evidence_references` — Decision,
   Source(path,commit), Finding with influences; exact rows and evidence refs;
   no findings rows from the other block.
3. `empty_kinds_are_omitted_and_reported` — findings-only graph -> decision
   omitted, no decision marker, exact Omitted entry.
4. `authored_bytes_survive_and_second_build_is_unchanged` — seeded authored and
   foreign bytes; recomputed then unchanged; bytes identical.
5. `legacy_block_is_refused_before_writing` — seeded generated:explanation ->
   LABEL_LEGACY_BLOCK, bytes unchanged.
Contract: .espectacular/.../kaya-documents/block-content-limited-to-its-kind.toml
bound to test 1.
tests/cli_pipeline.rs: `build_writes_understanding_from_recorded_records` —
ledger with a Finding entity and a Decision + Source + influences edge; build
writes docs/research/understanding.md with both blocks, envelope recomputed
names them, no run/artifact rows.

## Commands

narrow: cargo test --test understanding --test cli_pipeline
full: cargo test; ah check --changes add-document-taxonomy --run-tests; openspec validate --changes --strict

## Files touched

src/projection/report.rs, src/main.rs, tests/understanding.rs,
tests/cli_pipeline.rs, docs/research/understanding.md (generated),
.espectacular/.../kaya-documents/block-content-limited-to-its-kind.toml,
tasks.md 3.2, .beads/issues.jsonl.

## Commit shape

1. `feat(projection): render the understanding document`
2. evidence commit.

