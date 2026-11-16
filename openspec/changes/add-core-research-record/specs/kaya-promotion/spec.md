## ADDED Requirements

### Requirement: Epistemic categories are closed

A record's epistemic category SHALL be exactly one of `event` (what a
source observed), `assertion` (a proposed interpretation), or `conclusion`
(an accepted interpretation). Tools SHALL create events only; assertions
SHALL carry their author and `proposed` status; conclusions SHALL exist
only with a recorded human approval.

#### Scenario: Tool cannot propose

- **WHEN** an actor of type `tool` calls a promotion command
- **THEN** the command refuses, naming the categories a tool may write
  (events only)

#### Scenario: Assertion lands as proposed

- **WHEN** a human or agent records an assertion ("this result contradicts
  H-001")
- **THEN** the record carries status `proposed`, its author, and a
  reference to the events it interprets

### Requirement: Approval is human and recorded

Promoting an assertion to a conclusion SHALL require a promotion event
whose actor type is `human`; the promotion event SHALL name the assertion
id and remain in the ledger. An AI-authored interpretation SHALL land only
as a `proposed` assertion, visibly flagged as machine-proposed.

#### Scenario: Unapproved conclusion refused

- **WHEN** a conclusion write lacks a promotion event with a human actor
- **THEN** the write is refused, naming the missing approval

#### Scenario: AI interpretation flagged

- **WHEN** an agent actor records an interpretation
- **THEN** it is stored as a `proposed` assertion flagged
  machine-proposed, and no conclusion exists for it

#### Scenario: Approval provenance recorded

- **WHEN** a human approves an assertion
- **THEN** the ledger contains the promotion event naming both the approver
  and the promoted assertion id

### Requirement: Conclusions can be challenged

A conclusion SHALL be challengeable: a new assertion citing counter-events
SHALL attach to it, demoting its displayed status to `challenged` until a
human resolves the challenge — without deleting or editing the conclusion's
record.

#### Scenario: Counter-assertion demotes status

- **WHEN** an assertion citing counter-events attaches to an established
  conclusion
- **THEN** the conclusion's displayed status is `challenged` and its
  original record is byte-intact in the ledger

### Requirement: Property coverage mirror

Every property row in the layer sidecar SHALL be verified by exactly one
dedicated scenario.

#### Scenario: tool_promotion_blocked

- **WHEN** a tool actor invokes promotion
- **THEN** the command refuses naming the tool-writable categories
- **VERIFIES** [[kaya.promotion.tools_emit_events_only]]

#### Scenario: unapproved_conclusion_refused

- **WHEN** a conclusion lacks a human promotion event
- **THEN** the write refuses naming the missing approval
- **VERIFIES** [[kaya.promotion.approval_required]]

#### Scenario: proposed_visible

- **WHEN** an agent records an interpretation
- **THEN** it exists as a machine-proposed assertion, no conclusion
- **VERIFIES** [[kaya.promotion.ai_proposed_only]]

#### Scenario: approval_provenance_recorded

- **WHEN** a human approves an assertion
- **THEN** the promotion event names approver and assertion id
- **VERIFIES** [[kaya.promotion.approval_provenance]]

#### Scenario: challenge_demotes_status

- **WHEN** a counter-assertion attaches to a conclusion
- **THEN** status reads challenged and the original record is byte-intact
- **VERIFIES** [[kaya.promotion.challenge_without_deletion]]

#### Scenario: category_closed_checked

- **WHEN** a record carries a category outside event/assertion/conclusion
- **THEN** ingest refuses naming the closed categories
- **VERIFIES** [[kaya.promotion.category_closed]]

#### Scenario: proposal_channel_checked

- **WHEN** an assertion is written without status, author, or event references
- **THEN** the write refuses naming the missing fields
- **VERIFIES** [[kaya.promotion.proposal_channel]]
