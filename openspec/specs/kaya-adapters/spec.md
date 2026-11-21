---
id: kaya.adapters
kind: intent
statement: "THE kaya.adapters capability SHALL read source-owned records and emit observation events without ever writing to its sources or interpreting what it observes, fail atomically with no partial events, carry every event's provenance naming the source tool, its version, and the source contract reference, and mirror every layer sidecar property with a dedicated verifying scenario."
---

# kaya-adapters Specification

## Purpose
The ingest boundary: observations from wai, git, test runners, turu, dont,
and specodelic enter the ledger as facts. An adapter reads its source's
records and emits events describing what happened at the source — it never
writes to the source, never authors content the source already owns, and
never emits conclusions about what the events mean.

## Constraints

| id                       | kind      | expr                                                                                                                                                             | traces_to |
|--------------------------|-----------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| source_authority         | invariant | `an adapter reads its source and never writes to the source's store, and never authors content duplicating what the source owns — a re-capture is acknowledged as a duplicate by source reference` | [[kaya.adapters]] |
| translate_not_interpret  | invariant | `adapter events describe what happened at the source; no adapter event carries an interpretation or conclusion field`                                                | [[kaya.adapters]] |
| failure_atomic           | invariant | `a failed adapter run ends with a labeled error naming source and record, and writes no events from that run to the ledger`                                          | [[kaya.adapters]] |
| run_failed               | effect    | `kaya.adapters.run_failed(detail)`                                                                                                                                   | [[kaya.adapters]] |
| provenance_linked        | invariant | `every emitted event's provenance names the source tool, its version, and a source-specific reference (artifact path, claim id, bundle digest, commit sha, run output path)` | [[kaya.adapters]] |
| contract_reference       | invariant | `each adapter's registration names the source-owned contract it consumes: wai (.wai artifact tree), dont (claim store), turu (bundle per whisper shared-store), specodelic (spk parse IR envelope), git (commits and tags), test runner (machine-readable run output)` | [[kaya.adapters]] |

## Model

### States
- `registered`
- `walking`
- `emitted`
- `failed` (emits: [[kaya.adapters.run_failed]])

### Transitions

| id      | from        | to       | guard                                                  |
|---------|-------------|----------|--------------------------------------------------------|
| walk    | registered  | walking  | `source reachable ∧ contract reference named`           |
| emit    | walking     | emitted  | [[kaya.adapters.translate_not_interpret]] ∧ [[kaya.adapters.provenance_linked]] ∧ [[kaya.adapters.source_authority]] |
| abort   | walking     | failed   | `¬([[kaya.adapters.translate_not_interpret]] ∧ [[kaya.adapters.provenance_linked]] ∧ [[kaya.adapters.source_authority]])` |

## Properties

| id                           | kind | derives_from                              | generator                              | predicate                                                  |
|------------------------------|------|-------------------------------------------|----------------------------------------|-------------------------------------------------------------|
| no_source_writes             | unit | [[kaya.adapters.source_authority]]        | `fixture_workspace_all_sources()`      | `∀ source: tree byte-identical after run`                    |
| facts_only_events            | unit | [[kaya.adapters.translate_not_interpret]] | `fixture_capture_run()`                | `no emitted event carries an interpretation field`           |
| atomic_failure               | unit | [[kaya.adapters.failure_atomic]]          | `mid_walk_invalid_record()`            | `labeled error ∧ zero events from the run`                   |
| provenance_per_adapter       | unit | [[kaya.adapters.provenance_linked]]       | `first_event_per_adapter()`            | `provenance names tool ∧ version ∧ reference`                |
| contract_reference_recorded  | unit | [[kaya.adapters.contract_reference]]      | `adapter_registry_fixture()`           | `registration names the consumed source contract`            |
| run_failed_labeled           | unit | [[kaya.adapters.run_failed]]              | `missing_bundle_file_is_a_labeled_failure()` | `refusal label equals kaya.adapters.failure_atomic ∧ ledger gains no events` |

## Requirements

### Requirement: Source authority retained

An adapter SHALL read its source's records and SHALL never write to the
source's store, and SHALL never author content that duplicates what the
source already owns — kaya holds projections and interpretation, the source
holds its own truth. A bounded content-derived attribute != text ownership;
wai keeps full text: the wai adapter MAY copy only a bounded first-heading
title and its digest from a record's own structure, and SHALL never copy the
artifact body or any arbitrary text. A bounded first-heading title is an
observation of the file's own structure, not an interpretation.

#### Scenario: Adapter writes nothing to its source

- **WHEN** any adapter runs against its source
- **THEN** the source's file tree is byte-identical before and after the
  adapter completes

#### Scenario: No duplicate authoring

- **WHEN** the same wai decision is captured twice
- **THEN** the second capture is acknowledged as a duplicate of the first
  (by source reference), not a second authored copy

#### Scenario: First hash-space heading becomes the title

- **WHEN** the wai adapter captures an artifact whose text carries a line
  starting with `# `
- **THEN** the event's `data` carries `title` (the heading's remainder,
  trimmed) and `title_digest` (the first 16 hex of sha256 over the title
  bytes), and the artifact body stays wai's truth

#### Scenario: Multibyte title truncated without char boundary panic

- **WHEN** the first heading's text contains multibyte characters and is
  longer than the bounded title length
- **THEN** the captured title is cut on a character boundary and the
  capture never panics

#### Scenario: Title attributes omitted when file empty or heading-less

- **WHEN** an artifact is empty or carries no `# ` heading line
- **THEN** the event's `data` omits the `title` and `title_digest` keys
  entirely

#### Scenario: Long title truncated to the bounded constant

- **WHEN** the first heading's text exceeds the bounded title length
  (`TITLE_MAX_LEN`, 80 characters)
- **THEN** the captured title is the first 80 characters of the heading

#### Scenario: Renamed path event id keeps stem digest stability

- **WHEN** only an artifact's heading text changes between captures
- **THEN** the event id is unchanged — id derivation stays tool, version,
  project, timestamp, reference, and the path-identity payload, never file
  content

### Requirement: Translate, never interpret

An adapter SHALL emit events describing what happened at its source (a
decision exists, a test run completed, a bundle was packed) and SHALL NOT
emit conclusions about what the events mean.

#### Scenario: Adapters emit facts only

- **WHEN** the wai adapter walks `.wai/projects/*/designs/*.md` and finds
  three design artifacts
- **THEN** it emits three artifact-added style events referencing the files,
  and no event asserts why the designs were chosen

### Requirement: Atomic failure

When an adapter cannot read, parse, or validate a source record, it SHALL
fail with a labeled error naming the source and record, and SHALL have
written no partial events to the ledger for that run.

#### Scenario: Failure writes nothing

- **WHEN** a source record fails validation mid-walk
- **THEN** the run ends with a labeled error and the ledger contains no
  events from that adapter run

### Requirement: Per-source contracts

Each adapter SHALL name the source-owned contract it consumes and SHALL
carry that reference in every event's provenance: wai (`.wai/` artifact
tree), dont (claim store), turu (bundle per whisper's shared-store
capability), specodelic (`spk parse` IR envelope), git (commits and tags),
test runners (machine-readable run output).

#### Scenario: Provenance names source contract

- **WHEN** any adapter emits an event
- **THEN** provenance names the source tool, its version, and the
  source-specific reference (artifact path, claim id, bundle digest, commit
  sha, or run output path)

#### Scenario: Turu bundle consumed via shared-store contract

- **WHEN** the turu adapter captures knowledge entries
- **THEN** it consumes a bundle produced by turu's own bundle command
  (transport-agnostic, keyed by repo key) and never reads whisper's
  workspace internals directly

### Requirement: Property coverage mirror

Every property row in the layer sidecar SHALL be verified by exactly one
dedicated scenario.

#### Scenario: no_source_writes

- **WHEN** adapters run across all six sources
- **THEN** each source's tree is byte-identical after the run
- **VERIFIES** [[kaya.adapters.source_authority]]

#### Scenario: facts_only_events

- **WHEN** adapters capture a known fixture workspace
- **THEN** no emitted event carries an interpretation field
- **VERIFIES** [[kaya.adapters.translate_not_interpret]]

#### Scenario: atomic_failure

- **WHEN** a source record fails validation mid-walk
- **THEN** the ledger gains no events from that run
- **VERIFIES** [[kaya.adapters.failure_atomic]]

#### Scenario: provenance_per_adapter

- **WHEN** each adapter emits its first event on the fixture
- **THEN** provenance names tool, version, and source reference
- **VERIFIES** [[kaya.adapters.provenance_linked]]

#### Scenario: contract_reference_recorded

- **WHEN** an adapter is registered
- **THEN** its registration names the source-owned contract it consumes
- **VERIFIES** [[kaya.adapters.contract_reference]]

#### Scenario: run_failed_labeled

- **WHEN** an adapter cannot read, parse, or validate its source
- **THEN** the run ends with the labeled atomic refusal and the ledger
  gains no events from that run
- **VERIFIES** [[kaya.adapters.run_failed]]
