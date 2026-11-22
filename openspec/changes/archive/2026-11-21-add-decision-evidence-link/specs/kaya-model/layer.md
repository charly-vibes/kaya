---
id: kaya.model.layer
kind: intent
statement: "THE kaya.model capability SHALL connect admitted events into a research graph over a closed thirteen-entity vocabulary with a closed set of typed, direction-fixed relationships — including Decision-to-Source attribution via cites — refusing unknown entities, unknown relationships, dangling references, and influence cycles, with namespace-qualified node ids so ledgers compose across repositories."
---

# kaya.model — dual-format layer (sidecar)

## Purpose

The research vocabulary: a closed set of thirteen entity kinds and typed,
direction-fixed relationships that connect admitted events into a graph of
the research. Decisions attribute their evidence: the `cites` edge fixes a
Decision to the Source revision backing it, and the decision record scopes
its Evidence rows to cited sources — falling back to explicit,
marker-flagged ledger-global attribution for ledgers that predate the
relationship.

## Constraints

| id                       | kind      | expr                                                                                                                                                                                                                     | traces_to |
|--------------------------|-----------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| vocabulary_closed        | invariant | `node types are exactly {Project, Question, Idea, Hypothesis, Model, Experiment, Run, Artifact, Observation, Claim, Finding, Decision, Source} — another type is a labeled refusal`                                          | [[kaya.model]] |
| relationship_set_closed  | invariant | `relationships come from the closed set {investigates, predicts, executes, observes, supports, challenges, influences, realizes, derives, cites} — growth only by a schema revision`                                                 | [[kaya.model]] |
| references_resolve       | invariant | `every relationship endpoint resolves to an existing node — a dangling endpoint is a labeled refusal naming it`                                                                                                               | [[kaya.model]] |
| influence_acyclic        | invariant | `the subgraph of influences and derives edges is acyclic — a cycle is a labeled refusal naming its members`                                                                                                                   | [[kaya.model]] |
| direction_fixed          | invariant | `each relationship's direction is fixed by grammar: Run executes Experiment, Observation observes Run, Finding derives from Observation, Decision influenced by Finding, Model realizes Hypothesis, Decision cites Source — reversed edges are refusals, never coercions` | [[kaya.model]] |
| namespaced_ids           | invariant | `node ids are <project>/<TYPE>/<key> — distinct ledgers compose without collision`                                                                                                                                            | [[kaya.model]] |
| decision_evidence_scoping | invariant | `a cites edge is direction-fixed Decision -> Source, exact pair; a decision citing at least one Source renders its evidence only from the Sources it cites, one row per citing decision, and omits Sources no decision cites from its rows and source map` | [[kaya.model]] |
| pre_relationship_attribution | invariant | `a decision citing no Source predates Decision-to-Source attribution: its evidence stays ledger-global as before the relationship existed and carries a staleness marker naming that decision; a decision record with no complete Source renders no evidence section and no marker` | [[kaya.model]] |

## Model

### States
- `building`
- `built`
- `refused`

### Transitions

| id       | from      | to       | guard                                                                                  |
|----------|-----------|----------|----------------------------------------------------------------------------------------|
| admit    | building  | building | [[kaya.model.layer.vocabulary_closed]] ∧ [[kaya.model.layer.relationship_set_closed]] ∧ [[kaya.model.layer.direction_fixed]] |
| link     | building  | built    | [[kaya.model.layer.references_resolve]] ∧ [[kaya.model.layer.influence_acyclic]] ∧ [[kaya.model.layer.namespaced_ids]] ∧ [[kaya.model.layer.decision_evidence_scoping]] ∧ [[kaya.model.layer.pre_relationship_attribution]] |
| reject   | building  | refused  | `a guard violation is recorded with the offending node, relationship, or cycle`          |

## Properties

| id                           | kind | derives_from                            | generator                          | predicate                                      |
|------------------------------|------|-----------------------------------------|------------------------------------|------------------------------------------------|
| unknown_entity_refused       | unit | [[kaya.model.layer.vocabulary_closed]]        | `node_of_foreign_type()`           | `build ⇒ refusal naming the type`               |
| unknown_relationship_refused | unit | [[kaya.model.layer.relationship_set_closed]]  | `edge_of_foreign_type()`           | `build ⇒ refusal naming the relationship`       |
| cites_joins_the_closed_set   | unit | [[kaya.model.layer.relationship_set_closed]]  | `decision_cites_source_event()`    | `build ⇒ admits the cites edge`                 |
| cites_to_an_unknown_node_refused | unit | [[kaya.model.layer.references_resolve]]       | `cites_edge_to_absent_source()`    | `build ⇒ refusal naming the endpoint`           |
| dangling_ref_refused         | unit | [[kaya.model.layer.references_resolve]]       | `edge_to_absent_node()`            | `build ⇒ refusal naming the endpoint`           |
| influence_cycle_refused      | unit | [[kaya.model.layer.influence_acyclic]]        | `two_node_influence_cycle()`       | `build ⇒ refusal naming the cycle`              |
| direction_grammar_checked    | unit | [[kaya.model.layer.direction_fixed]]          | `reversed_executes_edge()`         | `build ⇒ refusal naming the direction rule`     |
| cites_direction_follows_the_grammar | unit | [[kaya.model.layer.direction_fixed]]          | `reversed_cites_edge()`            | `build ⇒ refusal naming Decision -> Source`     |
| evidence_lists_only_cited_sources | unit | [[kaya.model.layer.decision_evidence_scoping]] | `ledger_with_cited_and_uncited_sources()` | `decision record renders only cited sources, attributed per citing decision` |
| source_cited_by_two_decisions_renders_one_row_per_reference | unit | [[kaya.model.layer.decision_evidence_scoping]] | `two_decisions_citing_one_source()` | `one row per citing decision, sorted by source id then decision id` |
| mixed_ledger_scopes_the_cited_decision_and_marks_the_uncited_one | unit | [[kaya.model.layer.decision_evidence_scoping]] | `ledger_with_one_cited_and_one_uncited_decision()` | `cited decision scopes its evidence, uncited decision keeps global evidence with its marker` |
| duplicate_citation_renders_one_row_and_incomplete_citation_renders_none | unit | [[kaya.model.layer.decision_evidence_scoping]] | `decision_citing_one_source_twice_and_one_incomplete_source()` | `one row for the repeated citation, none for the incomplete citation` |
| unknown_relationship_refusal_names_cites_in_the_closed_set | unit | [[kaya.model.layer.relationship_set_closed]] | `edge_of_foreign_type()` | `refusal names cites among the closed set` |
| pre_relationship_ledger_falls_back_to_global_attribution_with_staleness_marker | unit | [[kaya.model.layer.pre_relationship_attribution]] | `ledger_with_sources_but_no_cites()` | `global rows unchanged ∧ marker line names the decision` |
| decisions_only_ledger_renders_no_evidence_section | unit | [[kaya.model.layer.pre_relationship_attribution]] | `ledger_with_a_decision_and_no_source()` | `no evidence section ∧ no marker` |
| graph_round_trip             | unit | [[kaya.model.layer.namespaced_ids]]           | `arbitrary_built_graph()`          | `serialize ∘ read == identity`                  |

## Requirements
### Requirement: Layer sidecar contract

This sidecar is the dual-format layer for the `kaya.model` capability and
SHALL be deployed verbatim by the archive companion; the plain delta in
`spec.md` beside it is its openspec face.

#### Scenario: Archive deploys the layer

- **WHEN** the change `add-decision-evidence-link` is archived after its
  scenarios have executing contract tests
- **THEN** the capability deploys with this layer attached fail-closed
