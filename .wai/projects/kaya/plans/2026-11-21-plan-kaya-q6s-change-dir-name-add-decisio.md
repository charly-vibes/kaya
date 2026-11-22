---
tags: [pipeline-run:tdd-ro5u-2026-11-21-kaya-q6s-decision-evidence-link, pipeline-step:plan]
---

# PLAN — kaya-q6s

Change dir name: **`add-decision-evidence-link`** (openspec/changes/add-decision-evidence-link/, mirroring archive/2026-11-21-add-capture-attributes layout: proposal.md, tasks.md, specs/kaya-model/{spec.md,layer.md}).

## Relationship shape (the recorded decision)

- `Relationship::Cites` joins the closed set (`ALL` grows 9→10, `RELATIONSHIP_SET` gains `cites`).
- Direction: `fixed_direction(Cites) = Some((EntityType::Decision, EntityType::Source))`; `requires_exact_pair(Cites) = true` (joins Executes/Observes/Derives/Realizes). Wire name `"cites"`; Display "cites".
- No acyclicity change (only `influences`+`derives` subgraph is cycle-checked; a Source cannot cite).

## File-by-file changes (green run; this run writes tests only)

### src/model.rs
1. `Relationship` enum: add `Cites` variant; `ALL` gains it (append after `Derives` — vocabulary order); `as_str` → `"cites"`.
2. `fixed_direction`: `Self::Cites => Some((EntityType::Decision, EntityType::Source))`.
3. `requires_exact_pair`: add `Self::Cites` to the matches! list.
4. `RELATIONSHIP_SET` const: append `, cites`; update the "closed nine-relationship set" phrase (`model.rs:239`) to a cardinality-free wording (e.g. "the closed relationship set is {…}") so growth doesn't re-stale it.
5. Module doc header: relationship set sentence mentions Decision→Source attribution.

### src/projection/report.rs (`render_decision`, report.rs:852)
1. Collect cited sources: for each edge with `Relationship::Cites` where `from` resolves to a Decision and `to` resolves to a Source, build a scoped row `format!("- {} — {reference} — see {}", source.id, decision.id)` with tuple key `(source.id, format!("{reference} — see {decision.id}"))`; skip Sources failing `source_reference` (incomplete stays skipped, cited or not).
2. Choose regime: `let scoped = graph.edges().any(|e| e.relationship == Relationship::Cites)`.
   - scoped: evidence = existing finding/decision rows + scoped source rows; un-cited Sources render nothing and enter no source map.
   - not scoped (pre-relationship ledger): evidence = today's rows unchanged; append marker line `- attribution-stale — evidence is ledger-global: this ledger records no decision-source links` as the last Evidence line.
3. Sorting unchanged: `evidence.sort(); evidence.dedup()` over the (id, reference-key, line) tuples — id order, then decision id for multi-cited sources.
4. Doc comment of `render_decision` rewritten: Sources render when cited (scoped regime) or globally with the staleness marker (pre-relationship regime).
5. Export `pub const ATTRIBUTION_STALE_ROW: &str = "- attribution-stale — evidence is ledger-global: this ledger records no decision-source links";` alongside the other block constants.

## Red tests (written this run; all fail before src/ changes)

### tests/model.rs (new tests)
| test | pins | RED failure mode |
|---|---|---|
| `cites_relationship_joins_the_closed_set` | `build` admits a `model.relationship.asserted` "cites" Decision→Source event; edge present in `graph.edges` | assertion panic in `expect_graph` (from_str refuses "cites" today) |
| `unknown_relationship_refusal_names_cites_in_the_closed_set` | the `kaya.model.relationship_set_closed` refusal message names `cites` alongside the existing nine | `assert!(message.contains("cites"))` fails |
| `cites_direction_is_decision_to_source_exact_pair` | reversed `Source cites Decision`, wrong pairs `Run cites Source`, `Decision cites Decision`, `Finding cites Source` all refused, label `kaya.model.direction_fixed`, rule text `Decision -> Source` | refusal never fires / label mismatch |
| `cites_to_an_unknown_node_is_refused_as_dangling` | cites edge naming an absent Source → label `kaya.model.references_resolve` naming the endpoint | build refuses earlier as unknown relationship |

### tests/understanding.rs (new tests + two expectation updates)
| test | pins | RED failure mode |
|---|---|---|
| `decision_evidence_lists_only_sources_the_decision_cites` | fixture `graph_with_scoped_decision_evidence`: d1 cites src1+src3, d2 cites src2+src3, complete src4 uncited, incomplete src5 cited — Evidence lines exactly `### Decisions` d1,d2 + `### Evidence`: f1—see d1; src1—see d1; src2—see d2; src3—see d1; src3—see d2; src4 and src5 absent; marker absent; source map == [d1,d2,f1,src1,src2,src3] | panic in `cites()` helper (relationship not yet admitted) |
| `pre_relationship_ledger_falls_back_to_global_attribution_with_staleness_marker` | reuses `graph_with_decisions_and_evidence()` (zero cites edges): all complete Sources render globally with today's exact `- {id} — {ref}` rows, marker line is the last Evidence line, source map stays global [d1,d2,f1,f2,src1,src2] | panic in `cites()` helper? No — this test needs no cites; it fails on the marker assertion (marker not implemented) |
| (update) `decision_record_renders_decisions_and_evidence_references` | expected block_lines gain the marker line (its fixture is a pre-relationship ledger) | currently green; stays green only after marker exists |
| (update) `incomplete_sources_never_render_or_enter_the_source_map` | expected block_lines gain the marker line (fixture has decisions + sources, no cites) | same |

Compile strategy: tests reference `Relationship::from_str("cites")` (existing API), never a not-yet-existing `Relationship::Cites` variant, so **all red tests compile against current APIs and fail as assertion/panic failures**, no expected compile failures. `RED suite command: cargo test --test understanding --test model`.

### Fixture shape (new, tests/understanding.rs)

```text
p/Decision/d1 "Ship the cache", p/Decision/d2 "Defer the migration"
p/Finding/f1 "Cache hit rate rose"  (d1 influences f1 — finding rows stay exercised)
p/Source/src1 {path src/cache.rs, commit abc123}   cited by d1
p/Source/src2 {path docs/notes.md, commit def456}  cited by d2
p/Source/src3 {path shared/notes.md, commit 789aaa} cited by d1 AND d2 (row-per-reference)
p/Source/src4 {path reports/x.md, commit 111bbb}   complete but uncited (must vanish)
p/Source/src5 {path only, no commit}               cited by d1 (incomplete stays skipped)
```

## kaya.model delta rows (<500 chars each)

MODIFIED `relationship_set_closed` (constraint expr):
`relationships come from the closed set {investigates, predicts, executes, observes, supports, challenges, influences, realizes, derives, cites} — growth only by a schema revision`

MODIFIED `direction_fixed` (constraint expr):
`each relationship's direction is fixed by grammar: Run executes Experiment, Observation observes Run, Finding derives from Observation, Decision influenced by Finding, Model realizes Hypothesis, Decision cites Source — reversed edges are refusals, never coercions`

ADDED Requirement **Decision evidence scoping** (goes in specs/kaya-model/spec.md delta):

> Decisions SHALL attribute their evidence: a `cites` relationship is direction-fixed Decision → Source, admitting no other pair. In a ledger holding at least one `cites` edge, `understanding.decision-record` SHALL render each Source only in rows attributed to a decision that cites it (`- {source} — {path}@{commit} — see {decision}`, one row per citing decision) and SHALL omit Sources no decision cites from rows and source map.

ADDED Requirement **Pre-relationship ledger attribution**:

> A ledger holding no `cites` edge predates Decision-to-Source attribution; the append-only ledger cannot retroactively hold links. `understanding.decision-record` SHALL fall back to ledger-global source evidence rendered exactly as before the relationship existed, and SHALL append one explicit staleness marker line (`- attribution-stale — evidence is ledger-global: this ledger records no decision-source links`) to the Evidence section so global attribution is never mistaken for per-decision scoping. Re-import is opt-in and out of scope here.

### Requirement text <500 chars — verified above (both ADDED requirements ≈480/470 chars body).

## Scenarios (heading → slug → overlay file)

| Requirement | Scenario heading | slug | overlay toml (.espectacular/changes/add-decision-evidence-link/kaya-model/) |
|---|---|---|---|
| Typed relationship closure (MODIFIED) | Cites joins the closed set | `cites-joins-the-closed-set` | cites-joins-the-closed-set.toml |
| Typed relationship closure (MODIFIED) | Cites to an unknown node refused | `cites-to-an-unknown-node-refused` | cites-to-an-unknown-node-refused.toml |
| Direction discipline (MODIFIED) | Cites direction follows the grammar | `cites-direction-follows-the-grammar` | cites-direction-follows-the-grammar.toml |
| Decision evidence scoping (ADDED) | Evidence lists only cited sources | `evidence-lists-only-cited-sources` | evidence-lists-only-cited-sources.toml |
| Decision evidence scoping (ADDED) | Source cited by two decisions renders one row per reference | `source-cited-by-two-decisions-renders-one-row-per-reference` | source-cited-by-two-decisions-renders-one-row-per-reference.toml |
| Pre-relationship ledger attribution (ADDED) | Pre-relationship ledger falls back to global attribution with staleness marker | `pre-relationship-ledger-falls-back-to-global-attribution-with-staleness-marker` | pre-relationship-ledger-falls-back-to-global-attribution-with-staleness-marker.toml |

Overlay format (from archived 2026-11-21-add-capture-attributes): `tool_version = "0.12.2"`, `id` == scenario slug == filename stem, `description` = scenario mirror sentence, `archetype = "PF"`, `status = "active"`, `superseded_by = ""`, `authored_with = "0.12.2"`, `[tests] cargo = [{flags = "<test_fn_name>"}]`. Bindings:

- cites-joins-the-closed-set → `cites_relationship_joins_the_closed_set`
- cites-to-an-unknown-node-refused → `cites_to_an_unknown_node_is_refused_as_dangling`
- cites-direction-follows-the-grammar → `cites_direction_is_decision_to_source_exact_pair`
- evidence-lists-only-cited-sources → `decision_evidence_lists_only_sources_the_decision_cites`
- source-cited-by-two-decisions-renders-one-row-per-reference → `decision_evidence_lists_only_sources_the_decision_cites` (same test fn covers the row-per-reference assertion)
- pre-relationship-ledger-falls-back-… → `pre_relationship_ledger_falls_back_to_global_attribution_with_staleness_marker`

## Spec-delta scenario bodies (for specs/kaya-model/spec.md, WHEN/THEN/VERIFIES)

- **Cites joins the closed set** / WHEN an event asserts `cites` from a Decision to a Source / THEN the graph admits the edge / VERIFIES [[kaya.model.relationship_set_closed]]
- **Cites to an unknown node refused** / WHEN a `cites` edge names a Source id absent from the ledger / THEN the build refuses naming the dangling endpoint / VERIFIES [[kaya.model.references_resolve]]
- **Cites direction follows the grammar** / WHEN an event asserts `Source cites Decision` or `Finding cites Source` / THEN the build refuses naming `Decision -> Source` / VERIFIES [[kaya.model.direction_fixed]]
- **Evidence lists only cited sources** / WHEN a ledger has two decisions citing distinct sources and one uncited complete source / THEN the decision record's evidence renders only cited sources, each attributed with `— see {decision}`, and the uncited source is absent / VERIFIES [[kaya.model.decision_evidence_scoping]]
- **Source cited by two decisions renders one row per reference** / WHEN two decisions cite the same source / THEN the evidence section renders one row per citing decision, sorted by source id then decision id / VERIFIES [[kaya.model.decision_evidence_scoping]]
- **Pre-relationship ledger falls back to global attribution with staleness marker** / WHEN a ledger records decisions and sources but no `cites` edge / THEN the decision record renders all complete sources globally as before AND appends the attribution-staleness marker line / VERIFIES [[kaya.model.pre_relationship_attribution]]

## Risks

1. **Existing byte-pins**: `decision_record_renders_decisions_and_evidence_references` and `incomplete_sources_never_render_or_enter_the_source_map` must gain the marker line in the same change as src (done here in test files); missing it breaks green with two failures that look like regressions.
2. **Mixed ledger semantics**: ≥1 cites edge ⇒ scoped regime; a decision without cites in such a ledger shows no source evidence rows and no marker. Accepted: the marker describes the ledger regime, not per-decision completeness. If the stranger-reads test later wants per-decision staleness, that is a schema-revision-sized change, not this one.
3. **Cardinality wording** ("closed nine-relationship set", `model.rs:239`; `ALL: [Self; 9]`) — grows in src; the new tests assert containment only so cardinality phrasing can't break them.
4. **Publications capability** reads `understanding.decision-record` blocks (kaya-publications spec.md:112-121 maps concept/architecture-design to it). Scoped rows change block bytes for post-relationship ledgers only; pre-relationship ledgers change bytes by exactly the marker line. No publication test today pins decision-record evidence bytes (verified: grep found no Evidence assertions outside understanding.rs) — flag for the 0.2.0 release notes.
5. **`Relationship::ALL` order**: appended after `Derives`; `serde` rename_all lowercase keeps wire format stable. Ledger round-trip tests (`graph_round_trips_identically`) don't enumerate variants exhaustively, so growth is safe.

## Out of scope (this run)

- No src/ edits, no openspec change dir creation (green run creates it from this plan), no `.espectacular` overlay files (list above is the contract), no archive/ship actions, no cargo/ah/openspec commands.

