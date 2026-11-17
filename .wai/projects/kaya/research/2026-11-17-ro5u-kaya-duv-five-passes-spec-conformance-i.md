---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-duv-developer-products, pipeline-step:ro5u-review]
---

# RO5U kaya-duv — five passes (spec conformance, invalid states, error
# boundaries, cost/determinism, test quality)

## Critical
none.

## High
none.

## Medium
M1 (boundaries) publish deleted a whole block-less page at an omitted product
path: remove_page stripped nothing, compared equal, and unlinked the file. With
four newly declared but unbacked product paths this could silently destroy an
authored docs/public/<product>.md. Fixed: a document without this page's
managed block is left byte-identical; block-only pages are still removed.

M2 (tests) the developer-guide package body and a multi-kind developer product
were never asserted through the CLI, so a mistyped member list or a narrowed
include list passed. Fixed: exact package body pinned and a Source/commit
fixture makes getting-started compose two kinds and how-to emit.

## Low
L1 (spec, boundaries) cross-reference fragments used `page.md#<kind>` but the
heading anchor strips dots, so every shared-fact link opened the top of the
target page. Fixed: slugged anchors, regenerated pages.
L2 (states, design note) fact-once ownership is the first emitted product in
name order, so getting-started owns the project-overview body and the overview
page carries only a cross-reference. Deliberate, documented and test-pinned;
not changed.

## Checked with no findings
- Every declared developer product emits only from its admission row (includes
  read from publication::admission, never restated), names its reader
  question, omits itself without backing with the exact omission set and no
  stub files, and shared facts render once.
- Declaration validity: all eight names resolve to products, no duplicates, no
  tier-field misuse, every include inside its row, package members declared.
- Cost/determinism: one linear pass over ~50 KB, no ledger access; page and
  export order name-sorted; write-on-change; repeat publish byte-identical.
- Committed docs/public and the export match the implementation and the spec
  tables; docs/src status lines updated.

