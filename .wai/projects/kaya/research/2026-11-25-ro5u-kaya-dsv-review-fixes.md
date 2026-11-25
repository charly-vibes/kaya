# kaya-dsv: Rule-of-5-Universal review, fixes applied

Reviewer agent on the full diff (code + tests + Part B content). Verdict:
zero Critical (gate passes); two High, three Medium, five Low.

## Applied

- H1 `orientation.research-map`: block always renders (index.rs appends the
  summary line even for zero nodes), so `reference` / `architecture-design`
  emit while explain said omitted. Fix: `populated(research-map) => true`,
  `collection_emitted` special-cases `topic-map` (still needs >=1 node for
  relationship rows). Runtime proof: committed-but-empty ledger in a git
  repo -> `kaya build` succeeds, `publish` writes reference.md, census no
  longer lists reference/architecture-design. Test updated: reference moved
  to the emitted side.
- H2 `product_explain` dropped the exploration-brief exclusion: named view
  said emitted while census and publish said omitted. Fix: reuse
  `product_emitted`; product views now also carry the admitted kinds'
  structural notes. Proof: `kaya explain tutorial` -> omitted + note.
  New test pins the census agreement.
- M1 experiment-record: renderer's emptiness condition is exactly an
  `executes` edge (Run -> Experiment); observations/findings enter only
  through run-anchored chains. Predicate narrowed to `Relationship::Executes`.
- M2 experiment-specification: renders only Question-investigates or
  Hypothesis-predicts joined into an existing Experiment (the dossier
  anchor set). Predicate rewritten as a join check.
- M3 ingestion rows named impossible paths (capture commands never create
  relationships; test import writes no command attribute). Rows rewritten
  to name the actual unblock: `import model` records with the join edge.
- L1 blank-string commands no longer count as run commands.
- L2 handoff artifacts require a non-blank `path`.
- L4 error labels pass the underlying `err.label` through instead of
  erasing the class.
- L5 vocabulary parity pinned: `DOCUMENT_KINDS` exported from
  `kaya::publication`; new test asserts `kaya explain <kind>` answers ok
  for all 11 kinds.

## Deferred

- L3 wrong-typed feeder attributes: build refuses with
  `kaya.projection.reuse_attribute` while explain calmly says unpopulated.
  A `refused` status naming offending nodes is a design addition -> filed
  as kaya follow-up ticket.
- exploration-brief reader-set gap (brief.md never reaches the publication
  reader set, so it cannot back tutorial): documented in the kind/product
  note; decision ticket filed.

## Gates after fixes

cargo fmt clean, clippy 0 warnings, cargo test 501 passed / 0 failed,
record-check drift [], release claims agree v0.4.0, links 0 unresolved,
ah check clean, census == export (example, example-gallery, tutorial).
