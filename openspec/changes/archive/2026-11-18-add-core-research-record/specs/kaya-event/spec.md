## ADDED Requirements

### Requirement: Event record schema

A `kaya.event/v0` record SHALL carry exactly the fields `id`, `schema`,
`type`, `timestamp`, `actor`, `subject`, `data`, `provenance`, `artifacts`,
where `type` is a namespaced dotted path (for example
`experiment.run.completed`), `timestamp` is RFC 3339 UTC, and `actor` names
its type (`tool`, `human`, or `agent`) plus a name. Ingest SHALL refuse
records with unknown schema versions, missing provenance, reused ids, or
fields outside the closed set.

#### Scenario: Well-formed record validates

- **WHEN** a record with the nine closed fields, schema `kaya.event/v0`, an
  RFC 3339 UTC timestamp, a typed actor, and provenance naming tool plus
  reference is ingested
- **THEN** it validates and is passed to the ledger for admission

#### Scenario: Unknown schema version refused

- **WHEN** a record declares `schema: kaya.event/v9`
- **THEN** ingest refuses it with a labeled error naming the unsupported
  version and the supported set

#### Scenario: Extra field refused

- **WHEN** a record carries a tenth field outside the closed set
- **THEN** ingest refuses it naming the unexpected field — the field set
  grows only by a new schema revision

#### Scenario: Id reuse refused

- **WHEN** a record's `id` equals an id already present in the ledger but
  the record is not byte-identical to the stored event
- **THEN** ingest refuses it naming the colliding id

### Requirement: Events carry provenance, not conclusions

Every event SHALL record what happened (counts, paths, references) and
SHALL NOT record an interpretation of what it means — a run-completed event
reports passed/failed counts and never states that a hypothesis was
falsified. Interpretation belongs to `kaya.promotion`.

#### Scenario: Run event states counts only

- **WHEN** a test-runner adapter emits an `experiment.run.completed` event
- **THEN** `data` carries the pass/fail/skip counts and artifact references,
  and no field names a hypothesis verdict

#### Scenario: Interpretation in an event is rejected at capture

- **WHEN** a capture call includes a conclusion-style field (for example
  `verdict: hypothesis-false`) outside the closed field set
- **THEN** the capture refuses it and points at the promotion command for
  interpretation

### Requirement: Property coverage mirror

Every property row in the layer sidecar SHALL be verified by exactly one
dedicated scenario.

#### Scenario: unknown_version_refused

- **WHEN** an unknown-schema-version fixture is ingested
- **THEN** the refusal is labeled and no ledger write occurs
- **VERIFIES** [[kaya.event.unknown_version_refused]]

#### Scenario: missing_provenance_refused

- **WHEN** a record without `provenance` is ingested
- **THEN** the refusal names the missing field
- **VERIFIES** [[kaya.event.missing_provenance_refused]]

#### Scenario: id_reuse_refused

- **WHEN** a non-identical record reuses a stored id
- **THEN** the refusal names the colliding id
- **VERIFIES** [[kaya.event.id_reuse_refused]]

#### Scenario: field_set_closed_checked

- **WHEN** a record with an extra field is ingested
- **THEN** the refusal names the field and cites the closed set
- **VERIFIES** [[kaya.event.field_set_closed]]

#### Scenario: well_formed_round_trips

- **WHEN** a canonical valid record passes ingest validation
- **THEN** the parsed value serializes back to the identical input bytes,
  and a non-canonical record is accepted and normalized to the canonical
  form
- **VERIFIES** [[kaya.event.well_formed_round_trip]]

#### Scenario: timestamp_rfc3339_checked

- **WHEN** a record carries a non-RFC-3339 timestamp
- **THEN** ingest refuses it naming the timestamp field
- **VERIFIES** [[kaya.event.timestamp_rfc3339]]
