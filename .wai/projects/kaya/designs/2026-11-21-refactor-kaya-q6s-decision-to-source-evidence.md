---
tags: [pipeline-run:tdd-ro5u-2026-11-21-kaya-q6s-decision-evidence-link, pipeline-step:refactor]
---

# REFACTOR kaya-q6s — Decision-to-Source evidence link

Run: tdd-ro5u-2026-11-21-kaya-q6s-decision-evidence-link (step 5/9 refactor).

## Tidy decision: none required (behavior-preserving review done)
- Both regimes reuse the existing `(id, dedup-key, line)` tuple + sort + dedup
  pattern; row formats differ by design (scoped rows carry `— see {decision}`),
  so merging the loops would obscure the regime boundary, not simplify it.
- The regime predicate (`edges().any(cites)`) necessarily traverses edges before
  branch selection; a single-pass rewrite would need a collected edge list —
  no allocation win at ledger scale, pure churn.
- `ATTRIBUTION_STALE_ROW` is a `pub const` beside the other public row/label
  consts, matching module convention.

## Verification
`cargo test` full — 30/30 suites ok, 0 failures (post-green, unchanged by this
no-op pass).

