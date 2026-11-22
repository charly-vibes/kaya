---
tags: [pipeline-run:tdd-ro5u-2026-11-21-kaya-q6s-decision-evidence-link, pipeline-step:green]
---

# GREEN kaya-q6s — Decision-to-Source evidence link

Run: tdd-ro5u-2026-11-21-kaya-q6s-decision-evidence-link (step 4/9 green).

## Shipped
- `src/model.rs`: `Relationship::Cites` joins the closed set (9→10), wire name
  `cites`, `fixed_direction = (Decision, Source)`, exact pair required; refusal
  texts name `cites` (message made cardinality-free, still byte-pinned by
  tests); direction refusals reuse `kaya.model.direction_fixed`; dangling
  endpoints reuse `kaya.model.references_resolve`.
- `src/projection/report.rs`: two attribution regimes switched by
  `graph.edges().any(|e| e.relationship == Cites)`:
  - scoped (≥1 cites edge): per citing decision, `- {source} — {reference} —
    see {decision}` rows; uncited Sources render no row and enter no source
    map; sort/dedup on `(source id, dedup key)`.
  - fallback (zero cites edges, pre-relationship ledgers): today's exact global
    rows plus trailing marker `ATTRIBUTION_STALE_ROW`
    ("- attribution-stale — evidence is ledger-global: this ledger records no
    decision-source links") appended after sort/dedup so it ends the Evidence
    section byte-stably.
- `openspec/changes/add-decision-evidence-link/`: proposal, tasks, kaya-model
  delta (MODIFIED closure + direction rows; ADDED "Decision evidence scoping"
  429 chars and "Pre-relationship ledger attribution" 482 chars), layer.md
  sidecar with updated exprs + two new constraint rows.
- `.espectacular/changes/add-decision-evidence-link/kaya-model/`: six overlay
  tomls (id == slug == filename, [tests] naming red test fns) + deployed copies
  in `.espectacular/kaya-model/`.

## Verification (orchestrator)
`cargo test` full — 30/30 suites ok, 0 failures (includes the 8 new red tests
now green and all pre-existing byte pins).

## Deviations
None substantive. Publications byte-change (decision-record Evidence rows) is
expected and flagged for the 0.2.0 release notes.

