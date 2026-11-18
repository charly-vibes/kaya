## ADDED Requirements

### Requirement: Source authority retained

An adapter SHALL read its source's records and SHALL never write to the
source's store, and SHALL never author content that duplicates what the
source already owns — kaya holds projections and interpretation, the source
holds its own truth.

#### Scenario: Adapter writes nothing to its source

- **WHEN** any adapter runs against its source
- **THEN** the source's file tree is byte-identical before and after the
  adapter completes

#### Scenario: No duplicate authoring

- **WHEN** the same wai decision is captured twice
- **THEN** the second capture is acknowledged as a duplicate of the first
  (by source reference), not a second authored copy

### Requirement: Translate, never interpret

An adapter SHALL emit events describing what happened at its source (a
decision exists, a test run completed, a bundle was packed) and SHALL NOT
emit conclusions about what the events mean.

#### Scenario: Facts-only events

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
