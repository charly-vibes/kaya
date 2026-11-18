## ADDED Requirements

### Requirement: Citation markers

An authored block MAY declare its evidence with one or more
`<!-- kaya:cites <id> [<id>…] -->` markers. Every cited id SHALL resolve to an
event present in the ledger. A marker naming an unknown id SHALL be refused by
`build` and `check` with a labeled error naming the document, the marker, and
the id. Markers appearing outside an authored block SHALL be treated as
ordinary prose. Marker bytes SHALL never be rewritten.

#### Scenario: Known ids accepted

- **WHEN** an authored block cites an event id present in the ledger
- **THEN** `build` succeeds and staleness detection uses that id

#### Scenario: Unknown id refused

- **WHEN** an authored block cites an id the ledger does not contain
- **THEN** `build` fails with a labeled error naming the document, the marker,
  and the unknown id

#### Scenario: Marker outside an authored block ignored

- **WHEN** a `kaya:cites` marker appears in uncategorized prose
- **THEN** it is left byte-identical and has no effect on staleness

### Requirement: Staleness trigger scope

A cited id SHALL count as changed only when a later event supersedes it and
changes its effective data. Newly appended events that do not supersede a cited
id SHALL NOT mark a block stale. The stale report SHALL name every changed
cited id of the block and SHALL leave the block's bytes unchanged.

#### Scenario: New evidence does not age a block

- **WHEN** a new capture event arrives and no cited id is superseded
- **THEN** the authored block is not reported stale

#### Scenario: Correction ages a block

- **WHEN** an event superseding a cited id changes that id's effective data
- **THEN** the block is reported stale naming the changed id

### Requirement: Acknowledgment events

An acknowledgment SHALL be recorded as one `verification.acknowledged` event
naming the document, the authored block, and the exact cited ids it covers,
with a human actor. Its subject SHALL be typed `Document`; `verification.*`
events are not graph records and the research model SHALL ignore them, creating
no node or edge. An acknowledgment SHALL clear staleness only for that block
and those ids; any unacknowledged changed cited id keeps the block stale. It
SHALL NOT alter authored bytes.

#### Scenario: Acknowledgment clears the covered ids

- **WHEN** a stale block's changed ids are acknowledged in full
- **THEN** the next build reports the block as not stale and the authored bytes
  are unchanged

#### Scenario: Partial acknowledgment leaves the block stale

- **WHEN** only one of two changed cited ids is acknowledged
- **THEN** the block remains stale naming the unacknowledged id

#### Scenario: Acknowledgment is not graph evidence

- **WHEN** an acknowledgment event is replayed
- **THEN** the research graph and generated blocks are identical to their
  state before the acknowledgment

### Requirement: Acknowledgment command

`kaya acknowledge <document> --block <name> --events <id>... --actor <name>`
SHALL append exactly one acknowledgment event and SHALL refuse a tool actor
with a labeled error, appending nothing. It SHALL refuse an acknowledgment whose
document does not exist or whose block name is not an authored block in that
document. Ids that are no longer cited SHALL be accepted and SHALL have no
effect.

#### Scenario: Tool actor refused

- **WHEN** `kaya acknowledge` is invoked with a tool actor
- **THEN** it fails with a labeled error and appends no event

#### Scenario: Unknown block refused

- **WHEN** `kaya acknowledge` names a document or block that does not exist
- **THEN** it fails with a labeled error and appends no event

### Requirement: Property coverage mirror

The three properties this change adds to the `kaya.projection` layer SHALL each
be verified by exactly one dedicated scenario.

#### Scenario: markers_validated

- **WHEN** an authored block cites an unknown id
- **THEN** the build refuses naming marker and id
- **VERIFIES** [[kaya.projection.citation_markers]]

#### Scenario: corrections_only

- **WHEN** new evidence arrives without superseding a cited id
- **THEN** the block is not stale
- **VERIFIES** [[kaya.projection.staleness_scope]]

#### Scenario: ack_scoped

- **WHEN** a block has two changed cited ids and only one is acknowledged
- **THEN** the block remains stale naming the other id, bytes unchanged
- **VERIFIES** [[kaya.projection.acknowledgment_effect]]