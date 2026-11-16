# Status

**kaya** is **experimental**: exploratory; may be renamed or sunset.

The behavioral specification is the openspec change
[`add-core-research-record`](../../openspec/changes/add-core-research-record/proposal.md)
— six capabilities (event, ledger, model, projection, promotion, adapters),
each with lint-clean specodelic layers and scenario-level red-test shapes.

| Area | Status | Notes |
|---|---|---|
| Architecture | specified | Four subsystems: capture adapters, research ledger, research model, documentation projections |
| Event format | [specified (draft)](../../openspec/changes/add-core-research-record/specs/kaya-event/spec.md) | `kaya.event/v0` — closed 9-field schema, append-only JSONL, provenance-preserving, observation-only |
| Research vocabulary | [specified (draft)](../../openspec/changes/add-core-research-record/specs/kaya-model/spec.md) | 13 entities, 9 typed direction-fixed relationships; relationships over entity count |
| CLI | not implemented | Planned: `kaya init/status/note/question/decision/finding/build/check` plus promotion commands (`approve`, `challenge`); the list grows with the specs |
| Integrations | [specified](../../openspec/changes/add-core-research-record/specs/kaya-adapters/spec.md), not implemented | wai, dont, whisper (turu bundle), specodelic (parse IR), git, test runners |
| Docs | buildable | `book.toml` + `theme/charly.css` at root; `docs.yml` deploy workflow lands with [kaya-vt9](https://github.com/charly-vibes/kaya) (CI setup) |
| CI | gates only | lefthook pre-commit/pre-push (`ah check`, `pretender`, `spk lint`); no build yet |

## Implemented

Nothing yet — see the beads tracker for the MVP 1 breakdown
(`bd ready` in a clone).

## In progress

- MVP 1: static research compilation (dossier from existing wai/git/test artifacts).

## Dogfooding

- Nothing consumes kaya yet — it is a consumer of the ecosystem.
- Reverse dogfooding: kaya's own specs are written in specodelic format and
  gate-checked by `spk lint` on every commit, and the change validates under
  `openspec validate --strict` — kaya is specified with the tools it will read.

## Notes

- No README install section: there is no binary yet; the standard install
  block lands with the first release.
