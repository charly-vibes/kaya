---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-1b9-project-overview, pipeline-step:ro5u-review]
---

# RO5U kaya-1b9 — five passes (spec conformance, invalid states, error
# boundaries, cost/determinism, test quality)

## Critical
none.

## High
none.

## Medium
M1 (tests) empty sections were never asserted on a partially populated graph:
the only subset case is the committed one-Question fixture, so a regression
emitting a fixed section skeleton would ship green. Fixed: Project-only
fixture pins the exact block.

M2 (tests) authored prose survival was not asserted on the omitted-kind path;
a regression deleting the document when the kind is empty would pass. Fixed
with an authored-plus-stale-block case and a stable third build.

## Low
L1 the publications fixture move retired the only proof that a vocabulary
block is read from a file that is not its canonical home. Fixed: a
technical-reference block inside index.md is still read.
L2 build_overview restated its block name instead of deriving it from Kind.
Fixed.
L3 the overview and understanding builders duplicated the section renderer and
the commit/delete policy, already diverging on whitespace-only commits. Fixed:
one shared row renderer and one shared commit policy.
L4 docs stale: status.md still called project-overview unspecified and omitted
overview.md from the compiled-output list; README likewise; the taxonomy
wording said "three id-sorted sections" where only rows are id-sorted. Fixed.

## Checked with no findings
- No run/artifact/commit row can reach the overview or its source map; block
  name is vocabulary; sections and kind omission are correct; the compiled
  document truthfully backs the overview product (page, package link, export
  attribution, reader question).
- Error boundaries: missing parent, authored/foreign bytes, legacy/duplicate
  blocks, unterminated markers, marker-like and blank titles, write failures,
  whitespace-only deletion, stale-block removal reported exactly once.
- Cost: linear pass plus scans of a 185 B document; BTreeMap ordering and
  exact-body comparison make repeated builds byte-identical; the added
  publication read is <0.2% of publish volume.

