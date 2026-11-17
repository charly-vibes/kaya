---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-1b9-project-overview, pipeline-step:plan]
---

# PLAN kaya-1b9 — project-overview

## Behavior (spec ids)

- kaya.documents/orientation.project-overview: Project, Question and current
  Findings rows only; no commit/artifact capture rows, runs, or other
  families. Empty sections omitted; empty kind omitted and reported; block
  named orientation.project-overview; validation before writing.
- Publication integration: the compiled document is readable by
  publication::read_documents so `overview` gains backing.

## Out of scope

research-map (kaya-i03, done); other products; editorial cutover (kaya-n50);
spec edits.

## Contract (binding)

src/projection/report.rs:

```rust
pub const OVERVIEW_BLOCK: &str = "orientation.project-overview";
pub fn build_overview(graph: &Graph, path: &Path) -> Result<Built, ProjectionError>;
```

- Body: `### Project` rows `- <id> — <title-or-key>` for Project nodes,
  `### Questions` the same for Question nodes, `### Findings` for Finding
  nodes; only non-empty sections, id-sorted; no other entity types.
- Empty when all three groups are empty: no file, remove an existing block,
  one Omitted{orientation, project-overview, no-records}; sources[block] and
  summary keys use the block name; whitespace-only document deleted;
  validate_generated_blocks before writes; shared helpers reused.
- main.rs: call after the understanding build, merge, and flag stale authored
  blocks for docs/research/overview.md.
- src/publication/mod.rs::read_documents: add overview.md to the fixed list.

## Tests to write first

tests/overview.rs
1. `overview_renders_project_questions_and_findings_only` — Project, Question,
   Finding, Run, Artifact(commit sha/path) nodes; exact rows, no run/artifact
   ids, no sha, no capture headings; sources map exact.
2. `empty_overview_is_omitted_and_removes_the_document` — graph with none:
   no file, Omitted entry, stale block removed, whitespace document deleted.
3. `authored_bytes_survive_and_second_build_is_unchanged`.
4. `legacy_block_is_refused_before_writing`.
5. `overview_block_name_is_the_vocabulary_name`.
tests/cli_pipeline.rs
6. `build_writes_overview_and_publish_uses_it` — ledger with a Question;
   build writes docs/research/overview.md; publish emits overview (not
   omitted), project-site links it, and publication.json documents include
   orientation.project-overview; no artifact/run rows.
tests/publications.rs (if needed): overview document backs the overview product.

Contract: bind `.espectacular/.../kaya-documents/known-family-and-kind-admitted.toml`
to test 1? The scenario "Known family and kind admitted" expects a projection
declaring family/kind and a build entry naming both; the overview test asserts
the block name and family/kind naming. Bind it to test 5
(`overview_block_name_is_the_vocabulary_name`) only if the scenario's assertions
can be expressed there; otherwise no new contract (cc9 owns remaining prose
scenarios).

## Commands

narrow: cargo test --test overview --test cli_pipeline --test publications
full: cargo test; cargo run -- build; cargo run -- publish; ah check --changes add-document-taxonomy --run-tests; openspec validate --changes --strict

## Files touched

src/projection/report.rs, src/projection/mod.rs (only if a helper is needed),
src/main.rs, src/publication/mod.rs, tests/overview.rs,
tests/cli_pipeline.rs, tests/publications.rs, docs/research/overview.md,
docs/public/*, docs/src/document-taxonomy.md (status), tasks.md 3.3,
.beads/issues.jsonl.

## Commit shape

1. `feat(projection): render the project overview`
2. evidence commit.

