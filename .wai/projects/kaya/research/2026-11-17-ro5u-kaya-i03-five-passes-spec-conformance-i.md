---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-i03-research-map-edges, pipeline-step:ro5u-review]
---

# RO5U kaya-i03 — five passes (spec conformance, invalid states, error
# boundaries, cost/determinism, test quality)

## Critical
none.

## High
none.

## Medium
M1 (spec, invalid states: 2 reviewers) ### Isolated duplicated every edgeless
Artifact/Run/Source row already rendered by the attribute detail sections: 73
of 77 nodes appeared twice in the committed block, contradicting the scenario
"Isolated entity still listed ... lists it once". Fix: isolated_section skips
Artifact/Run/Source; the distinct-node count is unchanged.

M2 (tests) relationship sort and triple-dedup were untested: every fixture
declared distinct, already-ordered edges, so deleting sort_unstable or dedup
left the suite green. Fix: exact ordered-dedup test with a non-adjacent
duplicate triple and out-of-order edges.

## Low
L1 index.rs build doc comment claimed validation happens "before anything is
rendered"; it happens before any write (render is pure). Fixed wording.

L2 docs/research/index.md authored paragraph still described the old
type-grouped entity list. Fixed to describe typed relationships, isolated
entities, and detail sections.

L3 no test rendered one id as both an edge endpoint and an attribute-section
row. Test added (exactly once per section, never isolated).

L4 self edge (legal for free-direction relationships) untested. Test added.

L5 empty graph / empty ### Isolated omission untested. Test added
(body exactly `\n_0 nodes from 0 events._\n`; no headings on an edge-only
graph).

L6 legacy block only tested without a valid map block in the same document.
Test added (valid map + legacy block refused, bytes unchanged).

## Checked with no findings
- Build cost: one extra linear scan over index.md (~22 KB) required by the
  closed-block contract; all orderings derive from BTreeMap/BTreeSet/explicit
  sort; a from-ledger re-derivation reproduced the committed block byte for
  byte (idempotent build).
- Error paths: missing/empty/unreadable document, unterminated/mismatched/
  duplicate/legacy markers, marker-like prose, no trailing newline; the single
  write happens after all refusals, so no partial document.
- Block-name drift is pinned by a test (RESEARCH_MAP_BLOCK ==
  Kind::ResearchMap.block_name()).
- Contract binding edges-rendered -> exactly one executing test.

