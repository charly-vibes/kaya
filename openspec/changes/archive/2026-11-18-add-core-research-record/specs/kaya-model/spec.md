## ADDED Requirements

### Requirement: Closed research vocabulary

The research graph SHALL use exactly the entity types Project, Question,
Idea, Hypothesis, Model, Experiment, Run, Artifact, Observation, Claim,
Finding, Decision, Source — a node of any other type is a labeled refusal.
Node ids SHALL be namespace-qualified (`<project>/<TYPE>/<key>`) so ledgers
from different repositories compose without collision.

#### Scenario: Unknown entity refused

- **WHEN** a graph build encounters a node typed `Task`
- **THEN** the build refuses with a labeled error naming the unknown type
  and the closed thirteen-type set

#### Scenario: Namespace-qualified ids compose

- **WHEN** two ledgers from different projects both define node
  `EXP-001`
- **THEN** their qualified ids (`alpha/Experiment/EXP-001`,
  `beta/Experiment/EXP-001`) remain distinct through any merge

### Requirement: Typed relationship closure

Relationships SHALL come from the closed typed set — investigates,
predicts, executes, observes, supports, challenges, influences, realizes,
derives — with direction fixed by the relationship (for example a Run
executes an Experiment, never the reverse). A reference to a node that does
not exist is a labeled refusal; influence and derives edges SHALL remain
acyclic.

#### Scenario: Unknown relationship refused

- **WHEN** an event asserts a relationship outside the closed set
- **THEN** the graph build refuses naming the unknown relationship

#### Scenario: Dangling reference refused

- **WHEN** a Claim references a Finding id absent from the ledger
- **THEN** the graph build refuses naming the dangling reference

#### Scenario: Influence cycle refused

- **WHEN** Decision A influences Decision B and B influences A
- **THEN** the graph build refuses naming the cycle's members — influence
  cannot become circular

### Requirement: Direction discipline

Relationship direction SHALL follow the vocabulary's fixed grammar: a Run
executes an Experiment; an Observation observes a Run; a Finding derives
from Observations; a Decision is influenced by Findings; an Implementation
Model realizes a Hypothesis. Reversed edges are labeled refusals, not
coercions.

#### Scenario: Reversed edge refused

- **WHEN** an event asserts `Experiment X executes Run Y`
- **THEN** the graph build refuses naming the direction rule for executes

### Requirement: Property coverage mirror

Every property row in the layer sidecar SHALL be verified by exactly one
dedicated scenario.

#### Scenario: unknown_entity_refused

- **WHEN** a node carries a type outside the closed set
- **THEN** the build refuses naming the type
- **VERIFIES** [[kaya.model.vocabulary_closed]]

#### Scenario: unknown_relationship_refused

- **WHEN** a relationship outside the closed set is asserted
- **THEN** the build refuses naming the relationship
- **VERIFIES** [[kaya.model.relationship_set_closed]]

#### Scenario: dangling_ref_refused

- **WHEN** a reference names an absent node
- **THEN** the build refuses naming the dangling reference
- **VERIFIES** [[kaya.model.references_resolve]]

#### Scenario: influence_cycle_refused

- **WHEN** influence edges form a cycle
- **THEN** the build refuses naming the cycle members
- **VERIFIES** [[kaya.model.influence_acyclic]]

#### Scenario: direction_grammar_checked

- **WHEN** a reversed executes edge is asserted
- **THEN** the build refuses naming the direction rule
- **VERIFIES** [[kaya.model.direction_fixed]]

#### Scenario: graph_round_trip

- **WHEN** a built graph is serialized and re-read
- **THEN** nodes, relationships, and ids are identical
- **VERIFIES** [[kaya.model.namespaced_ids]]
