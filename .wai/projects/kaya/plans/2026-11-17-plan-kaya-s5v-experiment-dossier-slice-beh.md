---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-s5v-experiment-dossier-slice, pipeline-step:plan]
---

# PLAN kaya-s5v — experiment dossier slice

## Behavior (spec ids)

- kaya.documents/Family vocabulary: closed 4 families x 11 kinds; unlisted
  value -> labeled refusal naming value + vocabulary; no document written.
- kaya.documents/Family-scoped generated blocks: block names are
  `<family>.<kind>`; a document holding a block outside the vocabulary is
  refused before writing, naming the block.
- kaya.documents/Intent is not record: `exploration.experiment-specification`
  renders only intent rows (Question via Investigates, Hypothesis via
  Predicts); `exploration.experiment-record` renders only observed rows
  (Run via Executes, Observation via Observes, Finding via Derives,
  Interpretations from promotion assertions).
- kaya.documents/Empty kinds omitted: no block, no placeholder row, no
  empty section; reported once as {family, kind, reason:"no-records"};
  a previously written block for a now-empty kind is removed.
- Scenarios bound this ticket: unknown_kind_refused, intent_record_split,
  legacy_block_refused, empty_kind_silent.

## Out of scope

orientation/understanding/reuse-handoff sources (kaya-1b9, 249, i03, 3f6);
publication layer (kaya-qp8); spec edits; ledger format change.

## Module contract (red tests compile against exactly this)

src/projection/report.rs (public):

```rust
pub const SPECIFICATION_BLOCK: &str = "exploration.experiment-specification";
pub const RECORD_BLOCK: &str = "exploration.experiment-record";
pub const REASON_NO_RECORDS: &str = "no-records";
pub const LABEL_UNKNOWN_KIND: &str = "kaya.documents.unknown_kind_refused";
pub const LABEL_LEGACY_BLOCK: &str = "kaya.documents.legacy_block_refused";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Family { Orientation, Exploration, Understanding, ReuseHandoff }
impl Family {
    pub const ALL: [Family; 4];
    pub fn as_str(self) -> &'static str;
    pub fn kinds(self) -> &'static [Kind];
    pub fn parse(value: &str) -> Option<Family>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    ProjectOverview, ResearchMap, ExplorationBrief, ExperimentSpecification,
    ExperimentRecord, FindingsAndAnalysis, DecisionRecord, TechnicalReference,
    ReproductionPackage, KnowledgeBrief, Handoff,
}
impl Kind {
    pub const ALL: [Kind; 11];
    pub fn family(self) -> Family;
    pub fn as_str(self) -> &'static str;        // "experiment-record"
    pub fn block_name(self) -> String;          // "exploration.experiment-record"
    pub fn parse(value: &str) -> Option<Kind>;  // bare kind, family-checked by caller
}

/// Closed block vocabulary, comma-separated family.kind names, stable order.
pub fn block_vocabulary() -> String;

/// Refuse a value outside the vocabulary: label LABEL_UNKNOWN_KIND, message
/// quotes the value and lists block_vocabulary().
pub fn refuse_kind(value: &str) -> ProjectionError;

/// `<family>.<kind>` -> Kind; unknown or family-mismatched names refuse with
/// LABEL_LEGACY_BLOCK naming the block and the vocabulary.
pub fn classify_block(name: &str) -> Result<Kind, ProjectionError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Omitted { pub family: &'static str, pub kind: &'static str, pub reason: &'static str }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Built { pub summary: Summary, pub omitted: Vec<Omitted> }

pub fn build(graph: &Graph, experiment_id: &str, path: &Path, promotion: &PromotionState)
    -> Result<Built, ProjectionError>;
pub fn stale_authored(...) -> Vec<StaleBlock>;   // unchanged
```

Rendering rules: sections are omitted when their group is empty (no
`- _(none)_`); a kind whose groups are all empty emits no block at all; blocks
are written in Kind order (specification then record); each block body starts
with its sections and rows are sorted ids `- <id>`; Interpretations rows keep
the current `- `<status>` <text> — machine-proposed: ...; cites: ...; author: ...`
shape. Summary keys: `"<block>:<experiment_id>"` in recomputed/unchanged, and
sources maps each key to the ids rendered in that block. Removing an obsolete
block counts as recomputed.

src/projection/mod.rs (public): `pub fn remove_generated(doc: &str, name: &str)
-> Result<String, ProjectionError>` — removes the block and its markers plus
the one blank separator line before it; absent block returns the document
unchanged; broken grammar still refuses. Every byte outside that span is
preserved.

src/main.rs: cmd_build merges `Built.omitted` across experiments, sorts and
dedupes, and adds envelope key `omitted`:
`[{"family":"exploration","kind":"experiment-record","reason":"no-records"}]`.

## Tests to write first (red)

tests/documents.rs (new)
1. `unknown_kind_is_refused_with_a_label_and_vocabulary` — Family/Kind parse
   of `experiment-report`, `archive`, `exploration.experiment-report` refused;
   label and full 11-name vocabulary in the message.
2. `legacy_block_name_is_refused_before_writing` — doc seeded with
   `generated:experiment-report` + authored prose; build errs with
   LABEL_LEGACY_BLOCK, message names the block, file bytes unchanged.
3. `intent_and_record_blocks_do_not_mix` — hypothesis + run + observation +
   finding; spec block has q1/h1 and no run/observation/finding ids; record
   block has r1/o1/f1 and no hypothesis id; both blocks present once.
4. `empty_kind_leaves_no_section_and_reports_omission` — no runs; no record
   markers anywhere; `omitted == [Omitted{exploration, experiment-record,
   no-records}]`; a pre-existing record block is removed and authored bytes
   survive; second build reports unchanged.
5. `specification_without_runs_renders_and_record_is_omitted` — inverse.
6. `every_kind_names_its_block_family_dot_kind` — 11 distinct block names;
   `Kind::parse(kind.as_str())` round-trips; `family()` agrees with
   `Family::kinds()`; `classify_block("orientation.finding")` refused.

tests/report.rs (migrate): block constants; deterministic bytes; second build
unchanged; changed graph recomputes; source map names every rendered fact for
both blocks; interpretations shape; authored/foreign bytes survive; stale
authored tests unchanged.

tests/acceptance.rs (migrate): dossier block constants become the two new
blocks; strip_generated_blocks handles both; section expectations match the
split; dossier authored prose still byte-identical.

tests/cli_pipeline.rs (migrate): summary keys `<block>:<experiment>`, block
markers, plus `omitted_kinds_reported_once_in_build_envelope` (two experiments,
one empty -> `omitted` names experiment-record once).

Contracts (subagent A only): 4 files under
`.espectacular/changes/add-document-taxonomy/kaya-documents/` created with
`ah scenario new add-document-taxonomy kaya-documents --requirement "Property
coverage mirror" <heading>`, then `[tests] cargo = [{flags = "<fn>"}]` bound to
tests 1-4.

## Commands

narrow: cargo test --test documents --test report --test acceptance --test cli_pipeline
full: cargo test && ah check --changes add-document-taxonomy --run-tests && openspec validate --changes --strict

## Files touched

src/projection/report.rs, src/projection/mod.rs, src/main.rs, tests/documents.rs,
tests/report.rs, tests/acceptance.rs, tests/cli_pipeline.rs,
docs/research/experiments/kaya-Experiment-add-core-research-record.md,
openspec/changes/add-document-taxonomy/tasks.md (tick 3.1 at ship),
.beads/issues.jsonl (bd export at ship).

## Commit shape

1. `feat(projection): document taxonomy blocks for the experiment dossier`
   (code + tests + contracts + regenerated dossier + tasks tick).
2. optional `refactor(projection): tidy report split` — only if the review
   finds structural cleanup worth separating.

