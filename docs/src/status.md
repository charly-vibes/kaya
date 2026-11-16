# Status

**kaya** is **experimental**: exploratory; may be renamed or sunset.

| Area | Status | Notes |
|---|---|---|
| Architecture | specified | Four subsystems: capture adapters, research ledger, research model, documentation projections |
| Event format | specified (draft) | `kaya.event/v0` — append-only JSONL, provenance-preserving |
| Research vocabulary | specified (draft) | 13 entities; relationships over entity count |
| CLI | not implemented | Planned: `kaya init/status/note/question/decision/finding/build/check` |
| Integrations | designed, not implemented | wai, dont, whisper, specodelic as information sources; git + test runners as passive capture |
| CI | gates only | lefthook pre-commit/pre-push (`ah check`, `pretender`, `spk lint`); no build yet |

## Implemented

Nothing yet — see the beads tracker for the MVP 1 breakdown
(`bd ready` in a clone).

## In progress

- MVP 1: static research compilation (dossier from existing wai/git/test artifacts).

## Dogfooding

None yet — kaya is a consumer of the ecosystem, not yet consumed by it.
