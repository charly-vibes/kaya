---
sut_path: /var/home/sasha/para/areas/dev/gh/charly/kaya
commit: cb8be97696b59f99909ba7f39a10daa2819294bc
updated: 2026-11-19
external_references:
  - path: /var/home/sasha/para/areas/dev/gh/charly/reports/hegel-eval-2026-11-16/README.md
    why: Suite-level Hegel adoption verdict and hands-on engine facts (build times, ci profile, shrink quality).
  - path: /var/home/sasha/para/areas/dev/gh/charly/antithesis/scratchbook/property-catalog.md
    why: House per-property dossier format and the screening-depth kaya entries this run re-derives at a pinned commit.
  - path: https://github.com/antithesishq/antithesis-skills
    why: antithesis-research method and scratchbook conventions.
  - path: https://hegel.dev/
    why: Property-based testing engine under recommendation (hegeltest 0.52.7, static-engine).
---

# SUT analysis — kaya research record

## Scope

Single-crate deep pass, replacing the suite-level screening entry for `kaya`
from the 2026-11-16 evaluation (that pass noted the tree moved during scanning;
this run pins `cb8be97696b59f99909ba7f39a10daa2819294bc`, rustc 1.98.1).
Scope per kaya-152: the JSONL ledger, correction resolution, event
serialization, and the status/build envelope. Projections, publications, and
adapters are out of scope except where they read the ledger.

## Architecture

- **Form:** one Rust crate (`kaya-vibes`), lib + thin CLI (`kaya`), edition 2024.
  Layers: `event` (closed 9-field `kaya.event/v0` record + JSONL codec),
  `ledger` (append-only JSONL store, digest, correction resolution),
  `model` (graph over 13 entity types built from effective events),
  `projection` (index/report/reuse documents), `promotion`, `publication`,
  `verification` (read-only `kaya check`), `adapters` (wai/git/test-runner/turu
  observation capture).
- **State:** `.kaya/ledger.jsonl` is the only authority; `docs/research/` and
  `docs/public/` are derived views. One event is one JSONL line. Corrections
  are events whose `data.supersedes` names another event; the model reads the
  effective set, not the raw set.
- **Envelope:** every subcommand prints a JSON envelope; `status` reports count,
  size, digest, pending proposals; `build` compiles projections under an
  optional `--expect-digest` pin.

## Concurrency and state model

Single-user CLI; no daemon, no socket. Two real concurrency surfaces:
(1) concurrent `kaya import` invocations appending to one ledger file, and
(2) read paths (`status`, `check`) racing a writer. Neither is designed for
today; the ledger documents append-only semantics for sequential use.

## Existing test infrastructure

No property-based framework today (no proptest, no Hegel). The suite is
contract-style: `tests/*.rs` mirror deployed spec scenarios one-to-one and are
bound from `.espectacular/<capability>/<scenario>.toml` by test-function name;
`ah check --run-tests --all-tests` executes the bindings (149 at the pinned
commit). Integration tests drive the real binary via `assert_cmd` with
tempfile-isolated ledgers; adapters and projections are tested against temp
fixtures.

## Failure-prone areas (candidates this run tests)

1. **Correction chains** — resolution must be total over supersedes chains
   (A→B→C) and refuse or surface cycles/dangling targets rather than dropping
   observations silently.
2. **Serialization round-trip** — the closed field set plus JSON canonical
   ordering must survive parse(to_line(e)) for unicode, empty containers, and
   boundary timestamps.
3. **Crash tails** — a partial final line (crash mid-write) meets the reader
   before the writer returns; recovery-on-read behavior decides observation
   loss.
4. **Envelope drift** — `status` and `build` must digest the same ledger bytes;
   any read-path mutation (stale-authored flags, projection writes) makes the
   two disagree.
