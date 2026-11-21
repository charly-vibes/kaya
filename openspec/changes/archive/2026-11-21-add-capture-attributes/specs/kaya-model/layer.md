---
id: kaya.model.layer
kind: intent
statement: "THE kaya.model capability SHALL connect admitted events into a research graph over a closed thirteen-entity vocabulary with a closed set of typed, direction-fixed relationships, admitting capture events whose observations carry only bounded content-derived capture attributes, refusing unknown entities, unknown relationships, dangling references, and influence cycles, with namespace-qualified node ids so ledgers compose across repositories."
---

# kaya.model — dual-format layer (sidecar)

## Purpose

The research vocabulary: a closed set of thirteen entity kinds and typed,
direction-fixed relationships that connect admitted events into a graph of
the research. Capture events carry a bounded, content-derived title
and its digest — an attribute of the source file's own structure — carried on the
`capture.artifact.observed` event's data map, and never copied onto Artifact
nodes, which keep only `path` and `kind` — and never enough to make the event
id content-dependent.

## Constraints

| id                       | kind      | expr                                                                                                                                                                                                                     | traces_to |
|--------------------------|-----------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| vocabulary_closed        | invariant | `node types are exactly {Project, Question, Idea, Hypothesis, Model, Experiment, Run, Artifact, Observation, Claim, Finding, Decision, Source} — another type is a labeled refusal`                                          | [[kaya.model]] |
| relationship_set_closed  | invariant | `relationships come from the closed set {investigates, predicts, executes, observes, supports, challenges, influences, realizes, derives} — growth only by a schema revision`                                                 | [[kaya.model]] |
| references_resolve       | invariant | `every relationship endpoint resolves to an existing node — a dangling endpoint is a labeled refusal naming it`                                                                                                               | [[kaya.model]] |
| influence_acyclic        | invariant | `the subgraph of influences and derives edges is acyclic — a cycle is a labeled refusal naming its members`                                                                                                                   | [[kaya.model]] |
| direction_fixed          | invariant | `each relationship's direction is fixed by grammar: Run executes Experiment, Observation observes Run, Finding derives from Observation, Decision influenced by Finding, Model realizes Hypothesis — reversed edges are refusals, never coercions` | [[kaya.model]] |
| namespaced_ids           | invariant | `node ids are <project>/<TYPE>/<key> — distinct ledgers compose without collision`                                                                                                                                            | [[kaya.model]] |
| capture_attribute_bound  | invariant | `content-derived capture attributes are limited to a bounded first-heading title and its digest; absence when no heading exists; event id derivation unaffected`                                                              | [[kaya.model.layer]] |

## Model

### States
- `building`
- `built`
- `refused`

### Transitions

| id       | from      | to       | guard                                                                                  |
|----------|-----------|----------|----------------------------------------------------------------------------------------|
| admit    | building  | building | [[kaya.model.layer.vocabulary_closed]] ∧ [[kaya.model.layer.relationship_set_closed]] ∧ [[kaya.model.layer.direction_fixed]] |
| link     | building  | built    | [[kaya.model.layer.references_resolve]] ∧ [[kaya.model.layer.influence_acyclic]] ∧ [[kaya.model.layer.namespaced_ids]] ∧ [[kaya.model.layer.capture_attribute_bound]] |
| reject   | building  | refused  | `a guard violation is recorded with the offending node, relationship, or cycle`          |

## Properties

| id                           | kind | derives_from                            | generator                          | predicate                                      |
|------------------------------|------|-----------------------------------------|------------------------------------|------------------------------------------------|
| unknown_entity_refused       | unit | [[kaya.model.layer.vocabulary_closed]]        | `node_of_foreign_type()`           | `build ⇒ refusal naming the type`               |
| unknown_relationship_refused | unit | [[kaya.model.layer.relationship_set_closed]]  | `edge_of_foreign_type()`           | `build ⇒ refusal naming the relationship`       |
| dangling_ref_refused         | unit | [[kaya.model.layer.references_resolve]]       | `edge_to_absent_node()`            | `build ⇒ refusal naming the endpoint`           |
| influence_cycle_refused      | unit | [[kaya.model.layer.influence_acyclic]]        | `two_node_influence_cycle()`       | `build ⇒ refusal naming the cycle`              |
| direction_grammar_checked    | unit | [[kaya.model.layer.direction_fixed]]          | `reversed_executes_edge()`         | `build ⇒ refusal naming the direction rule`     |
| graph_round_trip             | unit | [[kaya.model.layer.namespaced_ids]]           | `arbitrary_built_graph()`          | `serialize ∘ read == identity`                  |
| title_digest_deterministic   | unit | [[kaya.model.layer.capture_attribute_bound]]  | `same_artifact_captured_twice()`   | `title ∧ title_digest byte-identical across captures` |
| content_edit_keeps_event_id  | unit | [[kaya.model.layer.capture_attribute_bound]]  | `same_path_edited_heading()`       | `event id unchanged when only the heading moves` |
| multibyte_title_truncated_safely | unit | [[kaya.model.layer.capture_attribute_bound]] | `fixture_with_multibyte_heading()` | `title cut on a char boundary, never a panic`  |

## Requirements
### Requirement: Layer sidecar contract

This sidecar is the dual-format layer for the `kaya.model` capability and
SHALL be deployed verbatim by the archive companion; the plain delta in
`spec.md` beside it is its openspec face.

#### Scenario: Archive deploys the layer

- **WHEN** the change `add-capture-attributes` is archived after its
  scenarios have executing contract tests
- **THEN** the capability deploys with this layer attached fail-closed
