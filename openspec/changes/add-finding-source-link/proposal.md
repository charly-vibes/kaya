# Proposal: add-finding-source-link

## Why

The MVP-2 session-capture proposal filed its relationship hook verbatim
(archive `2026-11-22-add-agent-session-capture`, What Changes §5; line
references as filed — the arms now live in `fixed_direction`,
`src/model.rs:195-212`):

> Today the only link from a session capture into the graph is `cites`
> (Decision → Source, exact pair) pointing at the Source node
> `<project>/Source/entry-<entry id>`. … Linking a Finding to a session is
> **not** expressible today: `influences` is Decision → Finding and
> `derives` is Finding → Observation, and there is no Finding → Source
> edge. That gap needs a relationship-set delta filed with the MVP-2
> ticket.

This is that delta (ticket kaya-4gv). A Finding grounded in a captured
session Source is the other half of capture linking: today a finding can
point at nothing it rests on, and the AFK session-linking work has no edge
to emit.

## What Changes

- Name: `grounds`, a direction-fixed Finding → Source edge and an exact
  pair (joining executes/observes/derives/realizes/cites). `grounds` keeps
  the attribution family's shape — an evidence edge into a Source — while
  carrying its own semantics: `cites` is a Decision attesting the revision
  backing it; `grounds` is a Finding anchored in the recorded source it
  rests on. Not `cites`: the verb is taken, and one wire name cannot carry
  two direction rules — the `kaya.model.direction_fixed` refusal renders
  `{verb} is directed {from} -> {to}`, so `cites` cannot name both
  `Decision -> Source` and `Finding -> Source` (and `FromStr` maps one
  wire name to one variant). Not `observes`: taken (Observation → Run),
  and a Finding does not observe. Not `derives`: taken (Finding →
  Observation) and it participates in acyclicity; grounding is support,
  not derivation. `grounds` is **not** acyclic: the acyclic subgraph stays
  exactly influences|derives. `grounds` is exact-pair Finding → Source, so
  no grounds edge leaves a Source and a grounds-only cycle is
  unconstructible; multi-edge cycles touching grounds sit outside the
  acyclic subgraph, as for `cites`.
- `src/model.rs`: `Relationship::Grounds` joins the enum; `ALL` 10→11;
  `as_str` "grounds"; `RELATIONSHIP_SET` gains `, grounds` so the
  `kaya.model.relationship_set_closed` refusal names it; `fixed_direction`
  arm `(Finding, Source)`; `requires_exact_pair` (six structural
  relationships); `is_acyclic_edge` untouched. Module doc header and the
  `requires_exact_pair` doc comment ("five structural relationships")
  amended.
- No projection change: the research map renders every declared edge
  generically (`src/projection/index.rs` `relationship_section`), so a
  grounds edge renders with no code; decision-evidence attribution still
  walks `cites` only (`src/projection/report.rs`).
- No adapter emits the edge yet: capture commands and the AFK
  session-linking ticket consume this hook.
- Spec deltas: `kaya-model` MODIFIED (Typed relationship closure — the
  closed-set list gains `grounds` plus admitted/dangling scenarios;
  Direction discipline — the grammar gains "a Finding grounds a Source"
  with a reversed-grounds refusal; Property coverage mirror — one
  dedicated scenario per new layer property row).
- `.espectacular` bindings for the new scenarios, in the change overlay
  and under the deployed capability path.

## Impact

- Specs: kaya-model MODIFIED (three requirements). No ADDED requirements —
  grounds has no render behavior to scope.
- Code: `src/model.rs` only (four new contract tests in `tests/model.rs`,
  documented in tasks.md).
- Tickets: kaya-4gv. Consumed by the AFK session-linking follow-up, which
  emits the first `grounds` edges.
