---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-3f6-reuse-references, pipeline-step:plan]
---

# PLAN kaya-3f6 — reuse references

## Behavior (spec ids)

- kaya.documents/Reuse families reference, never copy: reproduction-package and
  handoff render references to authoritative sources; no ledger entity and no
  payload copy is created; every row names a reference resolvable without kaya.
- kaya.documents/Family-scoped generated blocks: blocks are
  reuse-handoff.reproduction-package and reuse-handoff.handoff, validated by
  the shared vocabulary gate before writing.
- kaya.documents/Empty kinds omitted: a kind with no references emits no file,
  no block, and one {family, kind, reason:"no-records"} entry.
- Bound scenario: reuse_reference_only.

## Out of scope

technical-reference (specodelic) and knowledge-brief (turu) — kaya-lh1,
blocked on deferred adapters 24c/syx; new adapter work; new ledger event types;
spec edits.

## Contract (binding)

src/projection/reuse.rs (new; registered as `pub mod reuse;` in projection/mod.rs)

```rust
pub const REPRODUCTION_BLOCK: &str = "reuse-handoff.reproduction-package";
pub const HANDOFF_BLOCK: &str = "reuse-handoff.handoff";

/// One row: a kaya node id and a source reference resolvable without kaya.
pub struct SourceRef { pub id: String, pub reference: String }

/// Inputs assembled from the modelled graph by the caller.
#[derive(Default)]
pub struct Inputs { pub revisions: Vec<SourceRef>, pub commands: Vec<SourceRef>, pub handoffs: Vec<SourceRef> }

pub struct Built { pub summary: Summary, pub omitted: Vec<Omitted> /* report::Omitted */ }

/// Write the reuse documents under `out`; return what was written/omitted.
pub fn build(inputs: &Inputs, out: &Path) -> Result<Built, ProjectionError>;
```

Row rendering (deterministic, sorted by id then reference):
- reproduction-package block body:
  `### Sources\n- <id> — <path>@<commit>` when revisions exist, then
  `### Commands\n- <run-id> — <command>` when commands exist; a kind with no
  rows is omitted (no file, no block).
- handoff block body: `### Handoffs\n- <artifact-id> — <path>`; omitted when
  empty.
Files: `<out>/reproduction-package.md` and `<out>/handoff.md`; existing
documents are validated with `validate_generated_blocks` before writing;
appending/replacing uses `replace_generated` so authored/foreign bytes survive.
summary.sources[block] = the row ids; recomputed/unchanged use the block name.

Graph assembly (in src/main.rs cmd_build, helper `reuse::inputs(&graph)`):
- revisions: Source nodes with attributes path+commit ->
  SourceRef{id, "path@commit"}.
- commands: Run nodes with `command` (string) or `commands` (array of strings)
  -> one SourceRef per command, reference = the command text, id = run id.
- handoffs: Artifact nodes whose `kind` == "handoffs" -> SourceRef{id, path}.
`cmd_build` merges `Built.summary` and `Built.omitted` into the envelope.

## Tests to write first

tests/reuse.rs (new)
1. `reproduction_package_renders_revision_and_command_references` - Source
   (path+commit) and Run (commands array) inputs; block rows exactly
   `- <id> — <path>@<commit>` and `- <run-id> — <cmd>`; every row contains its
   id and reference.
2. `handoff_renders_wai_artifact_references` - handoff inputs; exact rows.
3. `empty_kinds_are_omitted_and_write_no_file` - no inputs; both kinds
   omitted, no files created.
4. `reuse_block_names_are_vocabulary_names` - constants equal
   Kind::ReproductionPackage/Kind::Handoff block_name().
5. `building_writes_markdown_only` - building over a temp dir leaves an
   existing ledger file byte-identical and appends no events (compare file
   bytes and event count).
tests/cli_pipeline.rs
6. `build_writes_reference_documents_and_leaves_the_ledger_unchanged` - ledger
   with a commit-captured Artifact + wai handoff artifact: build writes
   docs/research/handoff.md with the artifact path, reports
   reuse-handoff.handoff in recomputed, ledger bytes and event count unchanged,
   and repro package omitted when no Source nodes exist.
7. legacy block in an existing reuse document is refused before writing.

Contract: .espectacular/.../kaya-documents/reuse-reference-only.toml bound to
test 1 (`cargo = [{flags = "reproduction_package_renders_revision_and_command_references"}]`).

## Commands

narrow: cargo test --test reuse --test cli_pipeline
full: cargo test; ah check --changes add-document-taxonomy --run-tests; openspec validate --changes --strict

## Files touched

src/projection/reuse.rs, src/projection/mod.rs, src/main.rs, tests/reuse.rs,
tests/cli_pipeline.rs, docs/research/reproduction-package.md (generated),
docs/research/handoff.md (only if the ledger yields handoffs),
.espectacular/.../reuse-reference-only.toml, tasks.md 3.5,
.beads/issues.jsonl.

## Commit shape

1. `feat(projection): render reuse documents as source references`
2. evidence commit for pipeline artifacts.

