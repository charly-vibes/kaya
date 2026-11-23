# Proposal: add-turu-source-row

## Why

The verification facts census counts events under four canonical sources —
`wai`, `git`, `test-runner`, `model` — but the turu adapter has shipped
(`capture.entry.observed`) and its events are invisible: `source_of` has no
arm for the turu event type, so every turu event falls through to "no
source". A repository that requires turu coverage cannot state it:
`--require-source turu` is refused as an unknown source (usage exit 2) before
any comparison runs, and a record holding turu events reports no turu count
at all.

## What Changes

- `src/verification/facts.rs`: the canonical source list grows from four
  entries to five, with `turu`; `source_of` classifies a
  `capture.entry.observed` event as the `turu` source.
- Spec: `kaya-verification` MODIFIED — the canonical source enumeration names
  `turu`, and the `--require-source` vocabulary names `turu`.
- Deployed constraint row: `facts_not_policy` in
  `openspec/specs/kaya-verification/spec.md` names `turu`. An openspec
  archive merges only requirement deltas — it does not touch a capability's
  constraint table — so this row is a direct deployed edit, recorded here so
  the archive explains the deployed state.
- Docs: `README.md` names `turu` in the `--require-source` vocabulary, and
  `CHANGELOG.md` records the new canonical source.
- Tests: the census counts turu events under `turu`, and `--require-source
  turu` passes on a record holding one and fails on a record without one.

## Impact

- Specs: kaya-verification MODIFIED (canonical source enumeration, assertion
  source vocabulary) plus the deployed `facts_not_policy` constraint row.
- Code: `src/verification/facts.rs` only.
- Docs: `README.md`, `CHANGELOG.md`, and the regenerated book spec page.
- Tickets: kaya-xjp.

## Out of Scope

- The turu adapter (`src/adapters/turu.rs`) already emits
  `capture.entry.observed` with `provenance.tool = turu` and a `Source`
  subject; it is unchanged.
- The model vocabulary, the projection, the ledger, and the CLI import
  envelope are unchanged — `src/main.rs` already names the import source
  `turu`.
- No new event type and no `SOURCE_VERSION` bump.
