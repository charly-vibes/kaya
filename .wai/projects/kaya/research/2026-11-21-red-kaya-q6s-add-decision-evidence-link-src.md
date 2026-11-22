---
tags: [pipeline-run:tdd-ro5u-2026-11-21-kaya-q6s-decision-evidence-link, pipeline-step:red]
---

# RED — kaya-q6s (add-decision-evidence-link)

src/ untouched (verified: `git status` shows only test files + pipeline artifacts). All tests compile against current APIs — no expected compile failures; every failure is an assertion failure or panic, as preferred.

## Files changed (tests only)

- `tests/understanding.rs` — new `cites()` helper (`Relationship::from_str("cites")` — the expect is the RED tripwire until the model admits it), new `ATTRIBUTION_STALE` marker constant, new fixture `graph_with_scoped_decision_evidence()`, two new tests, and two existing tests' expected lines gain the marker line (their fixtures are pre-relationship ledgers, so the marker is part of the new contract).
- `tests/model.rs` — four new tests mirroring the existing refusal patterns (`entity_event`/`relationship_event`/`build`/`expect_graph`/`expect_refusal`).

## Test names

Projection (tests/understanding.rs):
1. `decision_evidence_lists_only_sources_the_decision_cites` — new
2. `pre_relationship_ledger_falls_back_to_global_attribution_with_staleness_marker` — new
3. `decision_record_renders_decisions_and_evidence_references` — updated (marker line added)
4. `incomplete_sources_never_render_or_enter_the_source_map` — updated (marker line added)

Model (tests/model.rs):
5. `cites_relationship_joins_the_closed_set` — new
6. `unknown_relationship_refusal_names_cites_in_the_closed_set` — new
7. `cites_direction_is_decision_to_source_exact_pair` — new
8. `cites_to_an_unknown_node_is_refused_as_dangling` — new

## Orchestrator command

```
cargo test --test understanding --test model
```

(and after green: full `cargo test` for the meter — projection scoping + back-compat pinned there.)

## Expected failures and why (all RED today)

| test | failure | why |
|---|---|---|
| `decision_evidence_lists_only_sources_the_decision_cites` | panic: `cites is an admitted relationship` | `Relationship::from_str("cites")` returns `kaya.model.relationship_set_closed` refusal while the closed set holds nine names (src/model.rs:33,161,236-241) |
| `pre_relationship_ledger_falls_back_to_global_attribution_with_staleness_marker` | assert_eq block_lines: left has `ATTRIBUTION_STALE` as the 10th Evidence line, right renders 9 rows | `render_decision` (src/projection/report.rs:852) has no marker; fallback regime not implemented |
| `decision_record_renders_decisions_and_evidence_references` | same marker-line mismatch | its fixture `graph_with_decisions_and_evidence()` is a pre-relationship ledger; updated expectation pins the back-compat bytes |
| `incomplete_sources_never_render_or_enter_the_source_map` | same marker-line mismatch | fixture has a decision + sources, zero cites edges |
| `cites_relationship_joins_the_closed_set` | `expect_graph` panics: `graph build must succeed` | build refuses `cites` as an unknown relationship (`relationship_from_event` → from_str) |
| `unknown_relationship_refusal_names_cites_in_the_closed_set` | `assert!(message.contains("cites"))` fails | the refusal message names the current nine-name set (src/model.rs:239) |
| `cites_direction_is_decision_to_source_exact_pair` | label assert fails: `relationship_set_closed != direction_fixed` | the four wrong pairs (reversed `Source cites Decision`, `Run cites Source`, `Decision cites Decision`, `Finding cites Source`) are refused earlier as unknown relationships; after `cites` joins, they must fall to `kaya.model.direction_fixed` naming `Decision -> Source` |
| `cites_to_an_unknown_node_is_refused_as_dangling` | label assert fails: `relationship_set_closed != references_resolve` | same early refusal; after green, the dangling endpoint check (`dangling_reference`, label `kaya.model.references_resolve`) must fire |

## What green must satisfy (contract recap)

1. `cites` joins the closed set: `Relationship::Cites`, `ALL` grows to 10, wire name `"cites"`, direction `Decision -> Source`, exact pair (no other pair representable).
2. Scoped regime (≥1 cites edge in the graph): a Source renders an Evidence row only when a Decision cites it, as `- {source} — {path}@{commit} — see {decision}`; one row per citing decision; uncited Sources leave rows AND the decision-block source map; incomplete Sources stay skipped even when cited; no marker.
3. Fallback regime (zero cites edges): rows byte-identical to today (`- {source} — {path}@{commit}`, no suffix), source map stays global, and `- attribution-stale — evidence is ledger-global: this ledger records no decision-source links` is the last Evidence line.

## Post-green checks (orchestrator)

- `cargo test --test understanding --test model` green.
- Rebuild determinism intact: the marker is deterministic, so the existing rebuild/`unchanged` summary tests stay green without edits.

