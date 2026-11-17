---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-3f6-reuse-references, pipeline-step:ro5u-review]
---

# RO5U kaya-3f6 — five passes (spec conformance, invalid states, error
# boundaries, cost/determinism, test quality)

## Critical
none.

## High
H1 (tests) the graph-assembly contract had no executing test anywhere:
reuse::inputs was only exercised indirectly. Fixed with direct tests for
Source path+commit, missing commit, Run command forms, handoff artifacts,
non-handoff artifacts, and path-less handoffs.

## Medium
M1 (spec) cmd_build gated the whole reuse projection on "references exist or a
reuse document exists", so empty reuse kinds were reported only when a sibling
record happened to exist - a filesystem-dependent envelope and a deviation
from empty_kind_omitted. Fixed: always run the projection; the envelope is a
function of the ledger.

M2 (invalid states, boundaries) references were interpolated raw into the
generated body: a newline or a marker-shaped line in a command or path could
write a document that every later build refuses (unterminated/mismatched
block) with no self-repair path. Fixed: rows collapse CR/LF to spaces.

M3 (spec, invalid states) blank attribute values rendered unresolvable rows
(`- id — `, `path@`) and malformed attribute types were dropped silently.
Fixed: absent/blank attributes skip the node (not a reference), present-but-
invalid values are refused with label kaya.projection.reuse_attribute naming
the node and attribute.

M4 (tests) the legacy-block refusal test seeded the first document only, so it
could not distinguish validation-before-any-write from per-kind validation.
Fixed: legacy block seeded in the second document, asserting the first was
never written.

M5 (tests) the bound contract test could not fail for the "ledger gains no
entity" half of reuse_reference_only. Fixed: the contract binds both the
reference-rendering test and the ledger-unchanged test.

## Low
L1 removing the last block from a block-only document left a 0-byte file
instead of no file. Fixed: delete the file when nothing but whitespace
remains.
L2 the reuse documents never received the stale-authored pass every other
compiled document gets. Fixed: reuse::documents() exposes both targets and
cmd_build flags them.
L3 identical (id, reference) rows could render twice while the source map
counted one id. Fixed: dedupe identical pairs after sorting.
L4 reuse.rs duplicated report.rs document plumbing (generated_body,
read_existing, write, io_error, LABEL_IO). Fixed: moved to projection::mod and
shared.
L5 test gaps: stale-block removal, authored/foreign byte preservation,
second-build unchanged + source map, raw block bytes, CLI exactness. Fixed.
L6 Cost: each build rescans an already-validated document (validate, body,
replace). Deferred: one extra linear scan of a ~1 KB document, consistent with
report.rs and the index; no complexity argument.

## Checked with no findings
- No path creates an entity event or copies payload into the ledger; the
  committed reproduction package is the six committed Sources at 43b606f and
  every reference resolves in git.
- Block names are the closed vocabulary names; both existing documents are
  validated before the first write; row and section order are total orders
  over (id, reference) with no hash iteration.

