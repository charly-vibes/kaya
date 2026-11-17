## ADDED Requirements

### Requirement: Family vocabulary

A document projection SHALL classify every projected document by exactly one family and one kind drawn from the closed vocabulary — orientation {project-overview, research-map}, exploration {exploration-brief, experiment-specification, experiment-record}, understanding {findings-and-analysis, decision-record}, reuse-handoff {technical-reference, reproduction-package, knowledge-brief, handoff} — and SHALL refuse a family or kind outside it with a labeled error naming the offending value.

#### Scenario: Known family and kind admitted

- **WHEN** a projection declares family `exploration` and kind `experiment-record`
- **THEN** the document builds and its build entry names both the family and
  the kind

#### Scenario: Unknown kind refused

- **WHEN** a projection declares kind `experiment-report`
- **THEN** the build refuses with a labeled error naming `experiment-report`
  and lists the closed kind vocabulary, and writes no document

### Requirement: Intent is not record

The compiler SHALL project declared intent exclusively through `experiment-specification`, sourced from Experiment and Hypothesis entities and intent-bearing relationships, and observed execution exclusively through `experiment-record`, sourced from Run, Artifact, and observation events; no generated block SHALL mix records from both kinds, and a specification statement SHALL NOT be derived from a run or observation event.

#### Scenario: Specification not inferred from a run

- **WHEN** an experiment has run events but no declared hypothesis
- **THEN** `experiment-specification` is omitted and `experiment-record`
  renders the run, and neither block shows the other's content

#### Scenario: Record not asserted in the specification

- **WHEN** an experiment declares a hypothesis and has no run events
- **THEN** `experiment-specification` renders the hypothesis and
  `experiment-record` is omitted without a placeholder row

### Requirement: Family-scoped generated blocks

Every generated block in a projected document SHALL be named `<family>.<kind>` and SHALL contain only records admissible for that family and kind; the build SHALL refuse a generated block whose name is outside the closed block vocabulary.

#### Scenario: Block name outside the vocabulary refused

- **WHEN** a document contains `<!-- kaya:begin generated:experiment-report -->`
- **THEN** the build refuses with a labeled error naming the block and the
  closed block vocabulary

#### Scenario: Block content limited to its kind

- **WHEN** a `understanding.findings-and-analysis` block is generated from a
  ledger that also holds run events
- **THEN** the block renders only findings and their evidence references,
  never run rows

### Requirement: Empty kinds omitted

A family-kind with no source records SHALL be omitted from the projection — no generated section, no placeholder row, no empty-value line — and the build summary SHALL report it as omitted, naming the family, the kind, and the reason.

#### Scenario: Missing runs leave no section

- **WHEN** an experiment has no run events
- **THEN** the report contains no runs section and the build summary lists
  kind `experiment-record` as omitted with reason `no-records`

#### Scenario: Omission reported once

- **WHEN** several family-kinds in the projection set have no source records
- **THEN** the build summary names each omitted kind exactly once

### Requirement: Reuse families reference, never copy

`technical-reference`, `reproduction-package`, `knowledge-brief`, and `handoff` SHALL be assembled from references to their authoritative sources — specodelic spec ids, git revision plus commands, turu/whisper entry ids, wai handoff ids — and SHALL NOT create ledger entities or payload copies of the referenced content; every rendered item SHALL carry a source reference resolvable without kaya running.

#### Scenario: Brief adds no knowledge

- **WHEN** a knowledge-brief is built from turu entries
- **THEN** the ledger's entity count is unchanged and every rendered item
  names its entry id

#### Scenario: Reproduction package points at a revision

- **WHEN** a reproduction package is projected for a run
- **THEN** it renders the source revision and the commands as references, and
  no command text is stored as a kaya entity

### Requirement: Research map renders relationships

The `research-map` kind SHALL render every declared relationship among admitted entities as a typed edge naming both endpoints, so the map is a graph view rather than a type-grouped list, and it SHALL render admitted entities that carry no edges.

#### Scenario: Typed edge rendered

- **WHEN** the graph declares `supports` from a Run to a Hypothesis
- **THEN** the research map shows the edge with both endpoints and its
  relationship type

#### Scenario: Isolated entity still listed

- **WHEN** an admitted entity has no relationships
- **THEN** the research map lists it once and invents no edge for it

### Requirement: Property coverage mirror

Every property row in the layer sidecar SHALL be verified by exactly one dedicated scenario.

#### Scenario: unknown_kind_refused

- **WHEN** a projection declares a family or kind outside the closed
  vocabulary
- **THEN** the build refuses with a label naming the offending value
- **VERIFIES** [[kaya.documents.unknown_kind_refused]]

#### Scenario: intent_record_split

- **WHEN** an experiment has both an intent record and a run event
- **THEN** the specification block contains only intent rows and the record
  block only observed rows
- **VERIFIES** [[kaya.documents.intent_record_split]]

#### Scenario: legacy_block_refused

- **WHEN** a generated block name is outside the closed block vocabulary
- **THEN** the build refuses before writing, naming the block
- **VERIFIES** [[kaya.documents.legacy_block_refused]]

#### Scenario: empty_kind_silent

- **WHEN** a family-kind has no source records
- **THEN** no section is emitted and the build summary reports the omission
- **VERIFIES** [[kaya.documents.empty_kind_silent]]

#### Scenario: reuse_reference_only

- **WHEN** a reuse-handoff document is projected
- **THEN** each item carries a source reference and the ledger gains no
  entity
- **VERIFIES** [[kaya.documents.reuse_reference_only]]

#### Scenario: edges_rendered

- **WHEN** a research map is generated from a graph with edges
- **THEN** every declared edge appears with both endpoints and its type
- **VERIFIES** [[kaya.documents.edges_rendered]]