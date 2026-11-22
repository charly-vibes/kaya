# Proposal: add-decision-evidence-link

## Why

A decision record's Evidence section today renders every Source node in the
ledger as a ledger-global repo pin — `- {source} — {path}@{commit}` —
naming no decision. When two decisions exist, a reader cannot tell which
sources back which decision, and the global set silently overstates the
link between a decision and its evidence. Finding rows already name their
decision (`— see {decision}`); Source rows are the asymmetry.

## What Changes

- `src/model.rs`: the closed relationship set grows by `cites`, a
  direction-fixed Decision → Source edge and an exact pair (joining
  executes/observes/derives/realizes): reversed and wrong-entity pairs are
  labeled `kaya.model.direction_fixed` refusals, a `cites` edge naming an
  absent node is a labeled `kaya.model.references_resolve` refusal. No
  acyclicity change (a Source cannot cite).
- `src/projection/report.rs`: `understanding.decision-record` attributes
  evidence per decision. A decision citing at least one Source renders
  only the Sources it cites (`- {source} — {reference} — see {decision}`,
  one row per citing decision); Sources no decision cites render no row
  and enter no source map for that decision. A decision citing no Source
  predates the relationship for its own rows — its evidence stays
  ledger-global, exactly as before, and one explicit staleness marker
  (`attribution_stale_row`) names that decision. A record with no complete
  Source renders no Evidence section and no marker. Exported as
  `attribution_stale_row`.
- Spec deltas: `kaya-model` MODIFIED (relationship_set_closed and
  direction_fixed exprs gain `cites`; the property coverage mirror gains
  one dedicated scenario per new layer property row), ADDED (Decision
  evidence scoping; Pre-relationship ledger attribution).
- `.espectacular` bindings for the new scenarios, in the change overlay and
  under the deployed capability path.

## Impact

- Specs: kaya-model MODIFIED + two ADDED requirements.
- Code: `src/model.rs`, `src/projection/report.rs` only (test expectations
  updated in `tests/understanding.rs`, documented in tasks.md).
- Tickets: kaya-q6s. BLOCKS kaya-3tm (0.2.0).
- Publications reading decision-record blocks see byte changes: scoped
  ledgers change evidence rows, pre-relationship ledgers change by exactly
  the marker line. Flag for 0.2.0 release notes.
