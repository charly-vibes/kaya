---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-249-understanding-kinds, pipeline-step:ro5u-review]
---

# RO5U kaya-249 — five passes (spec conformance, invalid states, error
# boundaries, cost/determinism, test quality)

## Critical
none.

## High
none.

## Medium
M1 (spec, invalid states, boundaries) authored blocks in docs/research/
understanding.md were never checked for stale cited evidence: every other
compiled document gets flag_stale_authored, so a corrected event cited in
understanding prose produced no stale entry. Fixed: main.rs flags it like the
others.

M2 (cost, invalid states) understanding block order depended on build history:
a decision-only document gained the findings block appended after the decision
block, so identical ledgers produced different bytes. Fixed: canonicalization
guard mirroring the experiment kinds.

M3 (spec, boundaries, tests) a Source without path or commit rendered an
unresolvable evidence row (`@`, `path@`) and entered the source map. Fixed:
incomplete Sources are skipped entirely; shared source_reference returns
Option.

M4 (boundaries) removing the last blocks left a zero-byte understanding.md.
Fixed: delete a whitespace-only document.

M5 (boundaries, integration) publication::read_documents never opened
understanding.md, so the new kinds could never back the products that admit
them. Fixed: the file is in the reader's fixed list.

## Low
L1 block-name constants were literals not derived from Kind; the build did not
classify them. Fixed by deriving inside the loop (consts kept, pinned by test).
L2 duplicate node-attribute helpers across report/index/reuse with divergent
blank handling. Fixed: shared pub(crate) helpers in projection::mod.
L3 docs stale: findings-and-analysis still listed Run as a source (the ticket
deliberately excludes run rows) and the taxonomy/status pages said the kinds
were not generated. Fixed.
L4 test gaps: incomplete Sources, block order, both-kinds removal, const/kind
equality, stale-authored wiring, exact decision source map, publication backing,
removal summary. Added.

## Checked with no findings
- Run/artifact rows never enter the findings block or its source map.
- Decision evidence is limited to Source path@commit and influences-linked
  Findings; a Sources-only ledger never conjures a decision block.
- Block names are vocabulary names; validation precedes writes; authored and
  foreign bytes survive; repeated builds are byte-identical; cost is negligible
  (three linear scans over a small document, id-ordered BTreeMap iteration, no
  hash iteration).
- The committed no-file outcome is correct and pinned by acceptance.

