---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-s5v-experiment-dossier-slice, pipeline-step:ro5u-review]
---

# RO5U kaya-s5v — findings (5 passes: spec conformance, invalid states, error
# boundaries, cost/determinism, test quality). Review only; fixes follow.

## Critical
none.

## High
H1 (cost, invalid-states) `kaya build` aborts when a report document is
legitimately absent: report::build writes nothing when both kinds are omitted,
then main.rs::flag_stale_authored unconditionally reads the path and maps
NotFound to a build-io failure whenever the ledger has any correction. Trigger:
an experiment with no intent/record edges plus one superseding event.
Fix: NotFound is "nothing to check" -> return Ok(()).

H2 (spec) record emptiness counts ledger-global promotion assertions: any
assertion in the ledger makes a run-less experiment emit an
exploration.experiment-record block containing only ## Interpretations, i.e.
rows that may belong to another experiment, and no omitted entry is reported,
defeating empty_kind_silent. Fix: decide record emptiness from runs,
observations and findings only.

H3 (tests) the no-empty-section/no-placeholder rule is never exercised inside a
rendered block: every assertion targets documents whose only rendered block has
all groups populated, so restoring `- _(none)_` in push_section passes the suite.
Fix: runs-only fixture; assert absent headings and no placeholder inside the
record block.

H4 (tests) specification omission is never asserted: no record-only fixture, no
omitted/sources assertion for experiment-specification. Fix: record-only build.

## Medium
M1 (spec, invalid-states) closed-vocabulary validation is wired only into
report::build; index::build still writes generated:research-index and would
accept any legacy block. Deferred to kaya-i03, which owns the index block
rename to orientation.research-map and the shared validator wiring.

M2 (invalid-states, spec) blocks are appended, not kind-ordered: a later
hypothesis appends experiment-specification after an existing record block.
Fix: normalize to kind order when the specification block is absent and the
record block exists.

M3 (tests) omitted dedupe/sort cannot fail: the only fixture yields one
omission. Fix: several runless experiments -> dedupe and ordering load-bearing.

M4 (tests) classify_block has no positive coverage for the 9 unused kinds and
block_vocabulary() is only containment-checked; a wrong pairing for a
non-orientation family would pass. Fix: positive round-trip for all 11,
exact vocabulary string, Family::parse round-trip.

M5 (tests) duplicate generated block names are reported unchanged when bodies
match and never refused; the committed dossier only escaped because its
duplicates were removed by hand during migration. Fix: refuse duplicate names
in validation; test valid+legacy mix and duplicate same-name.

## Low
L1 (spec) refuse_kind reports a bad *family* as an unknown "kind" and never
lists the closed families. Fix: message names families and block vocabulary.

L2 (spec) unknown_kind_refused is exercised only through the parse helper; no
build input can declare a family/kind, so the enum closes the state instead.
Accepted as API-level; a declaration surface would be new scope.

L3 (spec) per-document classification vs per-block classification: the delta
says a document carries one family+kind, the implementation classifies blocks
and one dossier carries two kinds. Reading adopted: a document projection is
its generated block(s); risk noted for the taxonomy doc.

L4 (docs) authored prose still names the removed experiment-report block in the
dossier and in docs/src/document-taxonomy.md. Fix both.

L5 (tests) remove_generated preservation is asserted via contains() with the
block at EOF; a mid-document removal that truncated trailing bytes would pass.
Fix: mid-document fixture with exact expected string.

## Error-boundary pass (5th reviewer)

H1 confirmed identically (missing report + correction -> build-io abort).
L6 (new) remove_generated leaves a stray newline when the begin marker is the
document's second line (begin_start == 1): the separator guard requires two
preceding bytes. Reachable when a fresh report loses both kinds in one rebuild.
Fix: guard on begin_start >= 1 and treat the first byte as a separator.
L2 confirmed (kind order depends on build history).
Duplicate rendered block tolerated when bodies match (same as M5).

