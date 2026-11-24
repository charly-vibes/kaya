---
id: kaya.documents.layer
kind: intent
statement: "WHEN the compiler projects a document from the research record, THE kaya.documents capability SHALL classify it by one family and kind from the closed vocabulary, keep declared intent and observed execution in separate blocks, name each generated block <family>.<kind>, omit kinds with no records, render reuse and handoff kinds as references to their authoritative sources, and render every declared relationship in the research map."
---

# kaya.documents — dual-format layer (sidecar)

## Purpose

kaya manages two things at once: the research data, and the documentation
compiled from it. This capability is the record-facing half — the eleven
document kinds that answer "what do we know, and what did we intend?" — so the
client-facing publication layer has a stable interface to compose from.

The taxonomy this layer fixes was recovered from the origin conversation
(seven document families, later consolidated into four groups and eleven
kinds). Its two load-bearing rules are older than the taxonomy: intent is not
record, and reuse documents carry references instead of becoming a second
knowledge store.

## Constraints

| id                         | kind      | expr                                                                                                                                                                                                                                                                                  | traces_to                |
|----------------------------|-----------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|--------------------------|
| vocabulary_closed          | invariant | `every projected document carries exactly one family and one kind from the closed vocabulary; an unlisted value is a labeled refusal naming the value, never a fallback kind`                                                                                                         | [[kaya.documents.layer]] |
| intent_record_separated    | invariant | `experiment-specification sources only intent records (Experiment, Hypothesis, intent relationships); experiment-record sources only observed records (Run, Artifact, observation events); neither block derives content from the other`                                              | [[kaya.documents.layer]] |
| block_family_scoped        | invariant | `each generated block is named <family>.<kind> and contains only records admissible for that kind; a block name outside the closed block vocabulary is refused before writing`                                                                                                        | [[kaya.documents.layer]] |
| empty_kind_omitted         | invariant | `a kind with no source records emits no section, no placeholder row and no empty-value line; the build summary reports it once with family, kind and reason`                                                                                                                          | [[kaya.documents.layer]] |
| reuse_by_reference         | invariant | `technical-reference, reproduction-package, knowledge-brief and handoff render references to authoritative sources only — no ledger entity and no payload copy is created for referenced content`                                                                                     | [[kaya.documents.layer]] |
| relationship_rendered      | invariant | `research-map renders every declared relationship as a typed edge naming both endpoints and lists admitted entities that carry no edges`                                                                                                                                              | [[kaya.documents.layer]] |
| doctest_contracts_declared | invariant | `only a fenced block declaring a doctest execution contract becomes a first-class IR record with a stable id and a content hash; undeclared code stays prose; a contract's expected output changes only through a human-recorded supersede event, and no run result ever rewrites it` | [[kaya.documents.layer]] |

## Model

### States
- `collected`
- `classified`
- `projected`
- `omitted`

### Transitions

| id        | from       | to         | guard                                                                                                                     |
|-----------|------------|------------|---------------------------------------------------------------------------------------------------------------------------|
| classify  | collected  | classified | [[kaya.documents.layer.vocabulary_closed]]                                                                                 |
| project   | classified | projected  | [[kaya.documents.layer.block_family_scoped]] ∧ [[kaya.documents.layer.intent_record_separated]] ∧ [[kaya.documents.layer.reuse_by_reference]] |
| omit      | classified | omitted    | [[kaya.documents.layer.empty_kind_omitted]]                                                                                |

## Properties

| id                              | kind | derives_from                                        | generator                             | predicate                                                           |
|---------------------------------|------|-----------------------------------------------------|---------------------------------------|---------------------------------------------------------------------|
| unknown_kind_refused            | unit | [[kaya.documents.layer.vocabulary_closed]]          | `projection_with_unlisted_kind()`     | `labeled refusal naming the value ∧ no document written`            |
| intent_record_split             | unit | [[kaya.documents.layer.intent_record_separated]]    | `experiment_with_intent_and_run()`    | `spec rows ⊆ intent ∧ record rows ⊆ observed`                       |
| legacy_block_refused            | unit | [[kaya.documents.layer.block_family_scoped]]        | `document_with_legacy_block_name()`   | `build refuses ∧ names block ∧ no write`                            |
| empty_kind_silent               | unit | [[kaya.documents.layer.empty_kind_omitted]]         | `experiment_without_runs()`           | `no section ∧ summary reports kind once`                            |
| reuse_reference_only            | unit | [[kaya.documents.layer.reuse_by_reference]]         | `knowledge_brief_from_turu_entries()` | `∀ item: source reference ∧ entity count unchanged`                 |
| edges_rendered                  | unit | [[kaya.documents.layer.relationship_rendered]]      | `graph_with_typed_edges()`            | `∀ edge: rendered with type ∧ both endpoints`                       |
| doctest_record_is_first_class   | unit | [[kaya.documents.layer.doctest_contracts_declared]] | `document_with_declared_doctest()`    | `IR record with stable id ∧ content hash ∧ authored form preserved` |
| undeclared_stays_prose          | unit | [[kaya.documents.layer.doctest_contracts_declared]] | `document_with_plain_fence()`         | `no doctest record ∧ no runner invoked`                             |
| failing_run_records_fact        | unit | [[kaya.documents.layer.doctest_contracts_declared]] | `doctest_run_fails()`                 | `failed fact ∧ expected output unchanged`                           |
| doctest_facts_count_test_runner | unit | [[kaya.documents.layer.doctest_contracts_declared]] | `doctest_run_recorded()`              | `test-runner count +1 ∧ kind doctest ∧ no other source`             |
| supersede_human_only            | unit | [[kaya.documents.layer.doctest_contracts_declared]] | `supersede_by_tool_actor()`           | `labeled refusal ∧ human event names record ∧ to_hash ∧ chain`      |

## Requirements
### Requirement: Layer sidecar contract

This sidecar is the dual-format layer for the `kaya.documents` capability and SHALL be deployed verbatim by the archive companion; the plain delta in `spec.md` beside it is its openspec face.

#### Scenario: Archive deploys the layer

- **WHEN** the change `add-document-taxonomy` is archived after its scenarios
  have executing contract tests
- **THEN** the capability deploys with this layer attached fail-closed