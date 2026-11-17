---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-scq-status-release-architecture, pipeline-step:ro5u-review]
---

# RO5U kaya-scq — five passes (spec conformance, invalid states, error
# boundaries, cost/determinism, test quality)

## Critical
none.

## High
none.

## Medium
M1 (spec) the export's event_ids truncated every Source reference at '@'
(`kaya/Source/path.md` instead of `kaya/Source/path.md@43b606f`), so those ids
matched no ledger node and the markdown/export attribution disagreed for the
research map and reproduction package. Fixed: the id tokenizer keeps '@'; the
export is regenerated.

M2 (tests) the bound format_attribution test derived each product's expected
kinds from a global map and skipped id-less sections before consulting that
product's export row, so a product row could lose a composed kind unnoticed.
Fixed: per-product comparison against the product's own row, with rendered
sections comparing ids and cross-references required to carry none.

## Low
L1 the CLI attribution test derived its section universe from the value under
test and contained an unreachable guard; now admission-based with an explicit
empty-id assertion.
L2 newly omitted products are asserted file-free after their backing document
disappears.
L3 cmd_publish's doc comment still named the old product set. Fixed.

## Checked with no findings
- All eleven declared products emit only from their admission rows, carry
  their reader questions, omit themselves without backing (four explicit
  omissions), and tutorial cannot leak.
- Ownership is derived from emission, so a body owner always has a page and a
  cross-reference cannot dangle; repeated publishes are byte-identical; the
  committed tree is a fixed point.
- Refusals (tier, duplicate, fields, membership, admission, document kind)
  all precede the first write; authored pages at omitted paths survive;
  block-only pages are removed.

