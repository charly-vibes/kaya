# Tasks: add-decision-evidence-link

Red→green per the kaya-q6s ticket; each scenario below is a contract-test
shape bound in `.espectacular`.

## 1. Red — failing tests first (kaya-q6s)

- [x] 1.1 `cites_relationship_joins_the_closed_set` — a
      `model.relationship.asserted` "cites" Decision→Source event builds an
      edge.
- [x] 1.2 `unknown_relationship_refusal_names_cites_in_the_closed_set` —
      the relationship-set refusal names `cites` alongside the existing
      nine.
- [x] 1.3 `cites_direction_is_decision_to_source_exact_pair` — reversed
      and wrong-entity pairs refused, label `kaya.model.direction_fixed`,
      rule text `Decision -> Source`.
- [x] 1.4 `cites_to_an_unknown_node_is_refused_as_dangling` — a `cites`
      edge to an absent node refuses with label
      `kaya.model.references_resolve` naming the endpoint.
- [x] 1.5 `decision_evidence_lists_only_sources_the_decision_cites` —
      scoped regime: only cited sources render, attributed `— see
      {decision}`, one row per citing decision; uncited and incomplete
      sources absent from rows and source map; no marker.
- [x] 1.6 `pre_relationship_ledger_falls_back_to_global_attribution_with_staleness_marker`
      — zero-cites ledger keeps global rows and ends with the marker.
- [x] 1.7 Updated expectations in `decision_record_renders_decisions_and_evidence_references`
      and `incomplete_sources_never_render_or_enter_the_source_map` (their
      fixtures are pre-relationship ledgers) to pin the marker line.

## 2. Green — make the tests pass

- [x] 2.1 `src/model.rs`: `Relationship::Cites` in the enum, `ALL` 9→10,
      `as_str` "cites", `fixed_direction` Decision→Source,
      `requires_exact_pair`, `RELATIONSHIP_SET` gains `, cites`; refusal
      message wording made cardinality-free; module doc header amended.
- [x] 2.2 `src/projection/report.rs`: per-decision attribution. Citations
      are collected per decision; a decision with citations renders only
      those Sources (`- {source} — {reference} — see {decision}`, keyed
      `(source id, "{reference} — see {decision}")` so the existing
      `sort`/`dedup` orders multi-cited sources by id then decision); a
      decision without citations keeps today's global rows and gains one
      marker naming it; a record with no complete Source renders no
      Evidence section and no marker; `pub fn attribution_stale_row`;
      doc comment rewritten.
- [x] 2.3 Spec deltas (kaya-model MODIFIED + two ADDED) + layer sidecar +
      `.espectacular` bindings (+ deployed copies under
      `.espectacular/kaya-model/`).

## 3. Review fixes (Rule-of-5, CRITICAL 0)

- [x] 3.1 Attribution regime decided per decision: a cited decision scopes
      its evidence, an uncited one falls back with its own marker; mixed
      ledgers covered.
- [x] 3.2 The marker never creates an Evidence section — a record with no
      complete Source renders no section and no marker.
- [x] 3.3 Property coverage mirror MODIFIED: one dedicated scenario with a
      `**VERIFIES**` line per new layer property row.
- [x] 3.4 Closed-set rows name the set including `cites`; stale cardinalities
      removed from doc comments.
- [x] 3.5 New contract tests: mixed ledger, decisions-only ledger,
      duplicate and incomplete citations, shared source with its own test
      fn, closed-set refusal naming `cites`.
- [x] 3.6 Bindings regenerated: 20 tomls in the overlay and the deployed
      capability path; marker imported from the crate in tests.

## 4. Verify (orchestrator)

- [x] 4.1 `cargo test --test understanding --test model` green; full
      `cargo test` — 30/30 suites ok.
- [x] 4.2 `cargo clippy --all-targets -- -D warnings` — clean.
- [x] 4.3 `openspec validate --changes --strict` — pass.
- [x] 4.4 `ah check --changes add-decision-evidence-link` — 0 findings;
      `--run-tests` — 177 passed, 0 findings.
