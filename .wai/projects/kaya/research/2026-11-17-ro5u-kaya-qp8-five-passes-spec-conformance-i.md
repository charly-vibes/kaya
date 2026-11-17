---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-qp8-publications-tracer, pipeline-step:ro5u-review]
---

# RO5U kaya-qp8 — five passes (spec conformance, invalid states, error
# boundaries, cost/determinism, test quality)

## Critical
none.

## High
none.

## Medium
M1 (cost, invalid states) page writes bypassed the managed block: on any
content change `build` wrote the bare composed page and the CLI re-wrapped it,
so authored bytes outside `publication.<tier>.<name>` would be erased, and the
wrapper's block-found branch was a provable no-op that doubled every page read
and scan. Fixed: build writes `replace_generated(existing, block, body)` and
write-on-change; the wrapper is deleted.

M2 (invalid states) the structured export published a row for the omitted
product with `documents: []` and no emission marker, contradicting the same
file's package member `emitted:false` and the markdown tree. Fixed: product
rows carry `emitted`.

M3 (invalid states) duplicate declarations were accepted (duplicate pages,
duplicate export rows, a name both emitted and omitted), tier-mismatched
includes/members were silently discarded, and Collection declarations were
silently dropped. Fixed: `duplicate_declaration`, `tier_fields`,
`tier_unimplemented` refusals before any write.

M4 (invalid states) a package could list a valid product name that no
declaration declared, rendering it indistinguishable from an omitted product.
Fixed: undeclared members refused with `package_members`.

M5 (invalid states) DocumentRef repeated its kind as both map key and field;
the page/export and the CLI envelope could name different kinds. Fixed:
mismatch refused with `document_kind`.

## Low
L1 test gaps: stale page removal, read_documents refusal/happy path, the full
admission table, declared-tier mismatch, partial and duplicate includes,
publish idempotence, non-product/undeclared members, export shape, rebuild
summary and mtime, exact package bytes, refusal leaving an existing tree,
envelope documents, per-document attribution. All added.
L2 docs/src/document-taxonomy.md status overstated the reference page
(reuse-handoff.technical-reference has no producer yet). Corrected.

## Checked with no findings
- Vocabularies, questions and admission table match the spec verbatim; no
  inadmissible kind can slip through products; package pages embed no product
  body; fact-once holds; the committed project-site is truthful about
  overview being unavailable; export attribution equals the page source map;
  two publishes are byte-identical.

## Boundary pass (5th reviewer)
M6 page/export writes were not atomic: a failure after some pages were written
left the formats disagreeing (page rewritten, export stale). Fixed: two-phase
write (compute all bytes, write .tmp siblings, rename), export in the same
phase, stale-page removal only after success.
M7 an empty compiled block counted as backing and produced a stub page. Fixed:
whitespace-only bodies are absent.
M8 duplicate generated blocks in one compiled file were silently first-wins
while the rest of the codebase refuses duplicates. Fixed:
kaya.publications.duplicate_block naming file and block.
Checked, no finding: missing/unreadable research tree, malformed markers,
marker-like prose inside document bodies, stale-page removal, admission rows
with unknown kinds, repeated build idempotence.

