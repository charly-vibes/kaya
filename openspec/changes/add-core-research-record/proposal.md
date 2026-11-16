# Proposal: add-core-research-record

## Why

kaya's design exists as prose (`docs/src/index.md`) and an origin-conversation
research note, but nothing behavioral is specified: the event schema, the
ledger's append-only semantics, the research vocabulary, the projection
managed-block grammar, the epistemic promotion rules, and the adapter
contracts are all undefined as testable behavior. The MVP-1 tickets
(kaya-lb2 … kaya-5cg) have no test shapes, which violates the fleet's own
TDD policy. This change is kaya's first experiment applied to itself: the
spec corpus becomes the first artifact kaya's method structures.

## What Changes

- Add six deployed capability specs, written dual-format (plain openspec
  delta + specodelic layer sidecar, per the spec-integration convention):
  - `kaya.event` — the `kaya.event/v0` ledger record schema
  - `kaya.ledger` — append-only JSONL persistence, corrections, idempotent replay
  - `kaya.model` — the closed thirteen-entity research vocabulary and typed relationship graph
  - `kaya.projection` — the documentation compiler and managed-block grammar
  - `kaya.promotion` — the event → assertion → conclusion epistemic rules
  - `kaya.adapters` — per-source capture adapter contracts (wai, dont, turu, specodelic, git, test runners)
- No existing specs are modified or removed (there are none).
- No implementation code changes; requirements are the red-test shapes for MVP 1.

## Decisions baked in (from the origin conversation + spec review)

1. **Events are not genesis envelopes.** `kaya.event/v0` is the ledger
   record schema; CLI command I/O keeps genesis envelope conventions
   (`ok`/`envelope_version`/`envelope_kind`/`data`). The alignment seam is a
   declared extension point, not an accident (review finding F2).
2. **Corrections are new events** (superseding), never edits — the ledger is
   append-only at the byte level.
3. **Adapters translate, never interpret**: only `kaya.promotion` may change
   a record's epistemic category, and only with human approval.
4. **Capabilities are namespaced `kaya.*`** — avoids the near-name-collision
   class found in the spec review (F4).

## Impact

- Specs: +6 capabilities (new). Nothing removed.
- Code: none yet; scenarios define the contract tests MVP 1 must pass.
- Tickets: kaya-10k, kaya-aw3, kaya-dnk, kaya-e9s, kaya-5cg, kaya-lb2 map to
  these scenarios (see tasks.md).
