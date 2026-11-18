# Record gates — automation plan

Source: dogfooding evaluation of this repository (2026-11-18). The evaluation
found the committed record frozen 52 commits behind HEAD, no gate on the
generated products, a dead publication link in the deployed book, and a status
page contradicting the shipped release. This note is the durable plan the gate
tickets cite; it is not a spec. Capability behavior lives in
`openspec/changes/add-record-verification` and
`openspec/changes/clarify-authored-staleness`.

## What went wrong

| Failure | Evidence |
|---|---|
| record frozen | 80 events, newest 2026-11-16T18:53Z; 52 commits since the ledger commit |
| no gate on products | `just ci` never runs `kaya build`/`publish` or diffs their output |
| dead publication link | `book/index.html` → `../public/overview.html`; Pages uploads only `book/` |
| stale status page | `docs/src/status.md:21` says no release tagged; v0.1.0 is tagged and published |
| unexercised sources | no test-runner events; stable cargo cannot emit libtest JSON |

## Gates

| # | Gate | Tier | Catches | Determinism |
|---|---|---|---|---|
| G1 | `kaya check` (worktree) | pre-push / CI | drifted or hand-edited generated blocks | deterministic |
| G2 | `kaya check --index` | pre-commit | drift in the staged tree, partial staging | deterministic |
| G3 | `kaya check --fail-on-stale-authored` | pre-commit | authored blocks resting on corrected evidence | deterministic |
| G4 | `kaya check --require-kind/--require-source` | pre-push / CI | silent product or source loss | deterministic |
| G5 | lag gate (`scripts/record-lag.py`) | PR warn, main/release fail | record currency | deterministic |
| G6 | link check (`scripts/check-links.py`) | pre-push / CI | unresolved relative links in the built book | deterministic |
| G7 | release-claim check | CI / release | status page contradicting `Cargo.toml`/crates.io | deterministic |
| G8 | capture job (`kaya import wai/git`, build, publish) | scheduled / main only | record going stale | not deterministic — commits deliberately |
| G9 | publication deployment | `docs.yml` | publish tree unreachable | n/a |

## Vocabulary

- Sources: `wai`, `git`, `test-runner`, `model` — derived from capture family
  and provenance, never from the actor name (seed events carry a human actor).
- Kinds: dotted `family.kind` (`orientation.project-overview`), identical for
  emitted and omitted kinds.
- Exit contract: 0 clean, 1 violation, 2 usage/I/O.

## Boundaries

- Hooks are advisory; CI runs the same command as the authority.
- Capture never runs in a hook: it appends timestamped events.
- kaya evaluates no thresholds; the repository owns bounds, required lists, and
  allowlists.