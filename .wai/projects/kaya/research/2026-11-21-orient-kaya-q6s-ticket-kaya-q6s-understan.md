---
tags: [pipeline-run:tdd-ro5u-2026-11-21-kaya-q6s-decision-evidence-link, pipeline-step:orient]
---

# ORIENT — kaya-q6s

Ticket: kaya-q6s — "Understanding decision evidence has no Decision-to-Source link"
Metadata confirmed: files `src/model.rs`, `src/projection/report.rs`; base fbec965 (HEAD is f608235, the q5e ship commit — expected); labels complexity:m lane:afk; BLOCKS kaya-3tm (0.2.0).

## Acceptance as testable statements

1. **Scoping**: given a ledger with two Decisions each citing a distinct Source, `understanding.decision-record`'s Evidence section attributes each Source row to the decision that cites it; a Source no decision cites renders no row and enters no source-map entry.
2. **Back-compat**: given a pre-relationship ledger (Decisions + Sources, zero Decision→Source links), the record keeps today's ledger-global evidence rows AND carries an explicit staleness marker in the rendered output.
3. **Validation**: a `cites` edge to an unknown node is a dangling refusal; a reversed or wrong-entity-pair `cites` edge is a direction refusal — per the existing model grammar.
4. Meter: `cargo test` — a projection test asserts per-decision scoping; a second asserts the global fallback + staleness marker.

## Current renderer behavior (file:line evidence)

`src/projection/report.rs:852` `fn render_decision(graph: &Graph)`:

- `report.rs:865-873` — the global-attribution code path, verbatim:

```rust
    for node in graph
        .nodes()
        .filter(|node| node.entity == EntityType::Source)
    {
        let Some(reference) = source_reference(node) else {
            continue;
        };
        evidence.push((
            node.id.clone(),
            reference.clone(),
            format!("- {} — {reference}", node.id),
        ));
```

Every `Source` node in the ledger with a complete `path`+`commit` (`source_reference`, `src/projection/mod.rs:341`) becomes an Evidence row, regardless of any decision. The doc comment owns it: `report.rs:846-847` — "Sources are ledger-global repo pins". The source map inherits the same global set (`report.rs:875` `sources.insert(node.id.clone())`).

Finding rows DO name their decision (`report.rs:889-892`, `- {finding} — see {decision}` via `Relationship::Influences`); Source rows name nothing — that asymmetry is the ticket.

Pinned today by `tests/understanding.rs:444` `decision_record_renders_decisions_and_evidence_references`, which asserts both `src1` and `src2` rows render with no link concept present.

## Model vocabulary today

- `src/model.rs:33` `RELATIONSHIP_SET = "{investigates, predicts, executes, observes, supports, challenges, influences, realizes, derives}"`; `Relationship::ALL: [Self; 9]` (`model.rs:161`).
- Direction grammar: `fixed_direction` (`model.rs`, `impl Relationship`) maps Executes→(Run,Experiment), Observes→(Observation,Run), Derives→(Finding,Observation), Influences→(Decision,Finding), Realizes→(Model,Hypothesis); `requires_exact_pair` (Executes|Observes|Derives|Realizes) refuses every non-exact pair; `Influences` is reversal-only (Decision→Decision influence stays representable, acyclicity guards cycles).
- Refusals: unknown relationship → `kaya.model.relationship_set_closed` (`model.rs:239`, message names "the closed nine-relationship set"); dangling endpoint → `kaya.model.references_resolve` (`dangling_reference`); reversed/wrong pair → `kaya.model.direction_fixed` (`check_direction`).
- Edges enter only via `model.relationship.asserted` (`relationship_from_event`) or `GraphBuilder::relationship`; `build()` re-checks direction and dangling endpoints after forward references resolve.

## Decision (stated)

**Relationship shape: extend the closed set with `cites`, direction-fixed `Decision -> Source`, exact pair.**

Reasoning, kept boring:

1. **Direct Decision→Source beats Finding→Source.** The scoping rule the ticket asks for is per-decision evidence; a Finding→Source link would force the renderer to infer decision scope through `influences` and leave decisions-without-findings unattributable. One edge, one hop, no inference.
2. **Name `cites`.** Existing names are lowercase single verbs reading left-to-right ("Decision cites Source" parses like "Run executes Experiment"). No collision with the closed nine; consistent with the projection's existing evidence vocabulary (`kaya:cites` markers already mean "names my evidence").
3. **Exact pair** (join Derives/Realizes/Executes/Observes, not Influences): only Decision→Source is representable. Evidence attribution is a structural fact, not a semantic judgment, so the tighter grammar refuses `Finding cites Source`, `Run cites Source`, and the reversed `Source cites Decision` as direction errors rather than coercing them. Cycles are unreachable (a Source cannot cite), so no acyclicity changes.
4. **Renderer scoping rule:** a Source row renders iff some Decision cites it, and the row carries the decision using the convention Finding rows already have: `- {source} — {path}@{commit} — see {decision}`. A Source cited by two decisions renders one row per reference (exactly the existing finding convention, `report.rs:884-896` comment: "one finding influenced by two decisions keeps one evidence row per reference"), sorted by id then reference. Uncited Sources render no row and enter no source map. Finding→decision rows are unchanged.
5. **Back-compat rule:** if the graph contains zero `cites` edges (pre-relationship ledger — append-only, cannot retroactively hold links), the renderer keeps today's exact global rows (no `— see` suffix) and appends one explicit staleness marker line at the end of the Evidence section:

   `- attribution-stale — evidence is ledger-global: this ledger records no decision-source links`

   Graph-level predicate, not per-decision: the marker states the ledger's attribution regime. Re-import is opt-in elsewhere; the marker is what makes the unsoundness legible instead of silent. Per the v2 addendum, fallback+marker is the required behavior, not an error.

## Uncertainties

- Marker wording is pinned by the RED tests (chosen above); the green run implements the constant. Exact line equality is asserted so there is no drift surface.
- A post-relationship ledger where one decision cites nothing: scoping applies (graph has ≥1 cites edge), the decision simply has no Source evidence rows. The marker is deliberately absent — it describes the ledger, not a decision. Recorded as accepted semantics in plan.md risks.
- `Relationship::ALL` doc ("vocabulary order") and the `nine-relationship` phrase in `model.rs:239` go stale on growth — green-run concerns in src/, not test scope; the new model tests assert containment (`contains("cites")`), not the stale cardinality wording.
- Existing tests `decision_record_renders_decisions_and_evidence_references` (tests/understanding.rs:444) and `incomplete_sources_never_render_or_enter_the_source_map` (tests/understanding.rs:400) pin the current global bytes for no-cites graphs; post-green the marker line appears, so both expectations must gain the marker line — done in this red change (test files only).

