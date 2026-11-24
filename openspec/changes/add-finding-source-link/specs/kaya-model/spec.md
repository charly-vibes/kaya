## MODIFIED Requirements

### Requirement: Typed relationship closure

Relationships SHALL come from the closed typed set — investigates,
predicts, executes, observes, supports, challenges, influences, realizes,
derives, cites, grounds — with direction fixed by the relationship (for
example a Run executes an Experiment, never the reverse). A reference to a
node that does not exist is a labeled refusal; influence and derives edges
SHALL remain acyclic.

#### Scenario: Relationship outside the closed set refused

- **WHEN** an event asserts a relationship outside the closed set
- **THEN** the graph build refuses naming the unknown relationship

#### Scenario: Dangling reference refused

- **WHEN** a Claim references a Finding id absent from the ledger
- **THEN** the graph build refuses naming the dangling reference

#### Scenario: Influence edge forming a cycle refused

- **WHEN** Decision A influences Decision B and B influences A
- **THEN** the graph build refuses naming the cycle's members — influence
  cannot become circular

#### Scenario: A cites edge between a Decision and a Source is admitted

- **WHEN** an event asserts `cites` from a Decision to a Source
- **THEN** the graph admits the edge

#### Scenario: A cites edge naming an absent Source is refused

- **WHEN** a `cites` edge names a Source id absent from the ledger
- **THEN** the build refuses naming the dangling endpoint

#### Scenario: A grounds edge between a Finding and a Source is admitted

- **WHEN** an event asserts `grounds` from a Finding to a Source
- **THEN** the graph admits the edge

#### Scenario: A grounds edge naming an absent Source is refused

- **WHEN** a `grounds` edge names a Source id absent from the ledger
- **THEN** the build refuses naming the dangling endpoint

### Requirement: Direction discipline

Relationship direction SHALL follow the vocabulary's fixed grammar: a Run
executes an Experiment; an Observation observes a Run; a Finding derives
from Observations; a Decision is influenced by Findings; an Implementation
Model realizes a Hypothesis; a Decision cites a Source; a Finding grounds
a Source. Reversed edges are labeled refusals, not coercions.

#### Scenario: Reversed edge refused

- **WHEN** an event asserts `Experiment X executes Run Y`
- **THEN** the graph build refuses naming the direction rule for executes

#### Scenario: A reversed cites edge is refused

- **WHEN** an event asserts `Source cites Decision` or `Finding cites
  Source`
- **THEN** the build refuses naming `Decision -> Source`

#### Scenario: A reversed grounds edge is refused

- **WHEN** an event asserts `Source grounds Finding` or `Decision grounds
  Source`
- **THEN** the build refuses naming `Finding -> Source`

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

#### Scenario: cites_joins_the_closed_set

- **WHEN** an event asserts `cites` from a Decision to a Source
- **THEN** the graph admits the edge
- **VERIFIES** [[kaya.model.relationship_set_closed]]

#### Scenario: grounds_joins_the_closed_set

- **WHEN** an event asserts `grounds` from a Finding to a Source
- **THEN** the graph admits the edge
- **VERIFIES** [[kaya.model.relationship_set_closed]]

#### Scenario: unknown_relationship_refusal_names_cites_in_the_closed_set

- **WHEN** an event asserts a relationship outside the closed set
- **THEN** the refusal names `cites` among the closed set
- **VERIFIES** [[kaya.model.relationship_set_closed]]

#### Scenario: unknown_relationship_refusal_names_grounds_in_the_closed_set

- **WHEN** an event asserts a relationship outside the closed set
- **THEN** the refusal names `grounds` among the closed set
- **VERIFIES** [[kaya.model.relationship_set_closed]]

#### Scenario: cites_to_an_unknown_node_refused

- **WHEN** a `cites` edge names a Source id absent from the ledger
- **THEN** the build refuses naming the dangling endpoint
- **VERIFIES** [[kaya.model.references_resolve]]

#### Scenario: grounds_to_an_unknown_node_refused

- **WHEN** a `grounds` edge names a Source id absent from the ledger
- **THEN** the build refuses naming the dangling endpoint
- **VERIFIES** [[kaya.model.references_resolve]]

#### Scenario: cites_direction_follows_the_grammar

- **WHEN** an event asserts `Source cites Decision`
- **THEN** the build refuses naming `Decision -> Source`
- **VERIFIES** [[kaya.model.direction_fixed]]

#### Scenario: grounds_direction_follows_the_grammar

- **WHEN** an event asserts `Source grounds Finding`
- **THEN** the build refuses naming `Finding -> Source`
- **VERIFIES** [[kaya.model.direction_fixed]]

#### Scenario: evidence_lists_only_cited_sources

- **WHEN** a decision cites a Source and the ledger holds another complete
  Source no decision cites
- **THEN** the decision's evidence renders only the cited Source
- **VERIFIES** [[kaya.model.decision_evidence_scoping]]

#### Scenario: source_cited_by_two_decisions_renders_one_row_per_reference

- **WHEN** two decisions cite the same Source
- **THEN** the evidence renders one row per citing decision
- **VERIFIES** [[kaya.model.decision_evidence_scoping]]

#### Scenario: mixed_ledger_scopes_the_cited_decision_and_marks_the_uncited_one

- **WHEN** one decision cites a Source and another cites none
- **THEN** the cited decision scopes its evidence and the uncited one keeps
  global evidence with a staleness marker naming it
- **VERIFIES** [[kaya.model.decision_evidence_scoping]]

#### Scenario: duplicate_citation_renders_one_row_and_incomplete_citation_renders_none

- **WHEN** a decision cites the same Source twice and cites an incomplete
  Source
- **THEN** the evidence renders one row for the repeated citation and none
  for the incomplete one
- **VERIFIES** [[kaya.model.decision_evidence_scoping]]

#### Scenario: pre_relationship_ledger_falls_back_to_global_attribution_with_staleness_marker

- **WHEN** a ledger records a decision that cites no Source
- **THEN** the decision's evidence stays ledger-global and carries a
  staleness marker naming that decision
- **VERIFIES** [[kaya.model.pre_relationship_attribution]]

#### Scenario: decisions_only_ledger_renders_no_evidence_section

- **WHEN** a ledger records a decision and no Source
- **THEN** the decision record renders no evidence section and no marker
- **VERIFIES** [[kaya.model.pre_relationship_attribution]]
