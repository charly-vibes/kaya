---
id: kaya.event.layer
kind: intent
statement: "WHEN a capture source records a research event, THE kaya.event schema SHALL validate it as one kaya.event/v0 record over a closed field set with provenance and a namespaced event type, and SHALL refuse records that carry interpretation instead of observation."
---

# kaya.event — dual-format layer (sidecar)

## Purpose

Give kaya's research ledger a single, versioned record format: what a tool,
a human, or an agent observed — never what it concludes. The closed field
set makes records generatable from any language, diffable in git, and
validatable before admission; the observation/interpretation split is the
schema-level half of the promotion rules (`kaya.promotion`).

## Constraints

| id                      | kind      | expr                                                                                                                                                                                                 | traces_to |
|-------------------------|-----------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| field_set_closed        | invariant | `a record's fields are exactly {id, schema, type, timestamp, actor, subject, data, provenance, artifacts} — an unknown field is a labeled refusal, never ignored`                                      | [[kaya.event.layer]] |
| schema_version_pinned   | invariant | `the schema field equals kaya.event/v0; records naming another version are refused at ingest naming the supported set — the set grows only by a new schema revision`                                   | [[kaya.event.layer]] |
| stable_ids              | invariant | `every record id is unique within its ledger and never reused — a colliding non-identical record is a labeled refusal`                                                                                 | [[kaya.event.layer]] |
| timestamp_rfc3339       | invariant | `timestamp is RFC 3339 UTC — a non-conforming timestamp is a labeled refusal naming the field`                                                                                                          | [[kaya.event.layer]] |
| provenance_present      | invariant | `provenance names the emitting source (tool or human origin) and a reference (commit, artifact path, or session id) resolvable outside the ledger`                                                     | [[kaya.event.layer]] |
| type_namespaced         | invariant | `type is a dotted namespaced path <domain>.<subject>.<outcome> (for example experiment.run.completed) — an unnamespaced type is a labeled refusal`                                                     | [[kaya.event.layer]] |
| observation_only        | invariant | `a capture event records observations (counts, paths, references, statuses as emitted by its source) and carries no top-level interpretation field in data — promotion.* records are the sanctioned interpretation channel; hypothesis verdicts and conclusions outside it are promotion's territory, not the schema's` | [[kaya.event.layer]] |
| envelope_independence   | extension_point | `kaya.event/v0 records are ledger payloads, distinct from CLI command envelopes (genesis conventions); the seam where CLI I/O wraps events is declared here and revisited after the first end-to-end dossier` | [[kaya.event.layer]] |

## Model

### States
- `raw`
- `validated`
- `refused`

### Transitions

| id       | from       | to         | guard                                                                        |
|----------|------------|------------|------------------------------------------------------------------------------|
| parse    | raw        | validated  | [[kaya.event.layer.field_set_closed]] ∧ [[kaya.event.layer.schema_version_pinned]] ∧ [[kaya.event.layer.type_namespaced]] |
| stamp    | validated  | validated  | [[kaya.event.layer.timestamp_rfc3339]] ∧ [[kaya.event.layer.provenance_present]] ∧ [[kaya.event.layer.observation_only]] |
| reject   | raw        | refused    | `¬([[kaya.event.layer.field_set_closed]] ∧ [[kaya.event.layer.schema_version_pinned]] ∧ [[kaya.event.layer.stable_ids]])` |

## Properties

| id                         | kind | derives_from                          | generator                        | predicate                                                              |
|----------------------------|------|---------------------------------------|----------------------------------|------------------------------------------------------------------------|
| unknown_version_refused    | unit | [[kaya.event.layer.schema_version_pinned]]  | `record_with_unknown_schema()`   | `ingest ⇒ labeled refusal ∧ no ledger write`                            |
| missing_provenance_refused | unit | [[kaya.event.layer.provenance_present]]     | `record_without_provenance()`    | `ingest ⇒ refusal naming provenance`                                    |
| id_reuse_refused           | unit | [[kaya.event.layer.stable_ids]]             | `stored_id_with_changed_body()`  | `ingest ⇒ refusal naming the colliding id`                              |
| well_formed_round_trip     | unit | [[kaya.event.layer.field_set_closed]]       | `arbitrary_valid_record()`       | `parse ∘ serialize == identity on canonical lines; non-canonical input is normalized to the canonical form` |
| timestamp_rfc3339_checked   | unit | [[kaya.event.layer.timestamp_rfc3339]]      | `record_with_loose_timestamp()`  | `ingest ⇒ refusal naming timestamp`                                     |
| field_set_closed_checked   | unit | [[kaya.event.layer.field_set_closed]]       | `record_with_extra_field()`      | `ingest ⇒ refusal naming the extra field`                               |
| type_namespaced_checked     | unit | [[kaya.event.layer.type_namespaced]]        | `record_with_bare_type()`        | `ingest ⇒ refusal naming the type grammar`                              |
| facts_not_verdicts          | unit | [[kaya.event.layer.observation_only]]       | `record_with_verdict_field()`    | `capture ⇒ refusal pointing at promotion for interpretation`             |
| envelope_seam_declared      | unit | [[kaya.event.layer.envelope_independence]]  | `cli_envelope_fixture()`         | `CLI I/O uses genesis envelopes while ledger rows stay kaya.event/v0`    |

## Requirements
### Requirement: Layer sidecar contract

This sidecar is the dual-format layer for the `kaya.event` capability and
SHALL be deployed verbatim by the archive companion; the plain delta in
`spec.md` beside it is its openspec face.

#### Scenario: Archive deploys the layer

- **WHEN** the change `add-core-research-record` is archived after its
  scenarios have executing contract tests
- **THEN** the capability deploys with this layer attached fail-closed
