## ADDED Requirements

### Requirement: Impact mapping

Given a change set of changed paths and event ids, `kaya impact` SHALL compute
the impact set: the document IR records whose dependency edges intersect it,
directly or transitively. The report SHALL name each affected record and the
edge or rule that identified it, and SHALL name no record whose dependencies
do not intersect the change set. Impact rules SHALL be explicit project
policy maintained in the repository.

#### Scenario: Dependent records identified

- **WHEN** a change set contains a source path that a doctest record's edge
  names
- **THEN** the impact set contains that record and every record that depends
  on it, each naming the edge or rule that pulled it in

#### Scenario: Unaffected records excluded

- **WHEN** a change set touches a path that no record depends on
- **THEN** the impact set is empty and the report says so

### Requirement: Dispositions account for every impact

Each impact SHALL carry exactly one disposition from the closed vocabulary —
`updated`, `validated`, `reviewed`, `waived` — before the gate can pass; a
missing or unknown disposition is an unmet obligation, as is a stale authored
block whose scoped acknowledgment is missing. A `waived` disposition SHALL
bind to the record's dependency digest and SHALL lapse when the record's
dependencies change again.

#### Scenario: Every impact dispositioned

- **WHEN** the impact set holds three records and one has no disposition
- **THEN** the result is `fail` naming the unmet record

#### Scenario: Stale block is an unmet obligation

- **WHEN** an authored block in the impact set is stale — a cited id was
  superseded and not acknowledged
- **THEN** the gate reports that block as an unmet obligation regardless of
  any other disposition, until the scoped acknowledgment is recorded

#### Scenario: Waiver lapses when dependencies change

- **WHEN** a waived record's dependency digest changes in a later change set
- **THEN** the waiver no longer holds and the record is an unmet obligation
  again

### Requirement: Mechanical dispositions are derived

`validated` SHALL be established when the record's mandatory checks run green
against the current state, and `updated` SHALL be established when the
impacted document or block regenerates with bytes different from its state
before the change set; neither SHALL require a recorded event.

#### Scenario: Green checks establish validated

- **WHEN** an impacted record's mandatory checks run green against the
  current state
- **THEN** its disposition is `validated` with no recorded event

#### Scenario: Changed bytes establish updated

- **WHEN** an impacted document or block regenerates with bytes different
  from its state before the change set
- **THEN** its disposition is `updated` with no recorded event

### Requirement: Human dispositions are human events

`reviewed` and `waived` SHALL be recorded as events under the `verification`
family — which the research graph ignores — with the subject typed `Document`
and the record id, actor, and, for `waived`, a rationale in the event data;
only a human actor SHALL be accepted, and any other actor SHALL be refused
with a labeled error and nothing appended.

#### Scenario: Human dispositions are human events

- **WHEN** a `waived` disposition is recorded with a tool actor
- **THEN** the command fails with a labeled error and appends no event

### Requirement: Gate result contract

`kaya impact --staged | --range <revset>` SHALL emit a deterministic
machine-readable report — change set, rules fired, checks with results,
dispositions, waivers — with no run timestamps or environment details. The
result SHALL be `fail` exactly when a mandatory obligation is unmet, `review`
when only advisory findings are pending, else `pass`; the exit code SHALL be
nonzero exactly on `fail`, and advisory findings SHALL NOT block.

#### Scenario: Failed doctest blocks

- **WHEN** a doctest record in the impact set fails its run
- **THEN** the result is `fail`, the report names the record and the
  failure, and the exit code is nonzero

#### Scenario: Complete dispositions pass

- **WHEN** every impact carries a valid disposition and every mandatory
  check passed
- **THEN** the result is `pass` and the exit code is zero

#### Scenario: Advisory-only findings review

- **WHEN** the only findings are advisory and no mandatory obligation is
  unmet
- **THEN** the result is `review`, the report names each finding, and the
  exit code is zero

#### Scenario: Byte-identical reruns

- **WHEN** the gate runs twice on the same change set and ledger state
- **THEN** the two reports are byte-identical

### Requirement: Property coverage mirror

Every property row in the layer sidecar SHALL be verified by exactly one dedicated scenario.

#### Scenario: dependents_found

- **WHEN** a change touches a source path that a record's dependency edge names
- **THEN** the impact set contains the record and its transitive dependents,
  each naming the edge or rule that identified it
- **VERIFIES** [[kaya.impact.layer.impact_mapping]]

#### Scenario: unaffected_excluded

- **WHEN** a change touches a path that no record depends on
- **THEN** the impact set is empty and the report says so
- **VERIFIES** [[kaya.impact.layer.impact_mapping]]

#### Scenario: impacts_accounted

- **WHEN** an impact carries no disposition
- **THEN** the result is fail naming the unmet record
- **VERIFIES** [[kaya.impact.layer.dispositions_closed]]

#### Scenario: dispositions_derived

- **WHEN** a record's checks run green and an impacted document regenerates
  with changed bytes
- **THEN** `validated` and `updated` hold with no recorded events
- **VERIFIES** [[kaya.impact.layer.mechanical_dispositions_derived]]

#### Scenario: human_actor_required

- **WHEN** a `waived` disposition is recorded with a non-human actor
- **THEN** the command refuses with a labeled error and appends no event
- **VERIFIES** [[kaya.impact.layer.human_review_events]]

#### Scenario: stale_unmet

- **WHEN** a stale authored block is in the impact set without its scoped
  acknowledgment
- **THEN** the block is an unmet obligation regardless of any other
  disposition
- **VERIFIES** [[kaya.impact.layer.stale_as_obligation]]

#### Scenario: exit_contract

- **WHEN** the gate runs on a green state and on a state with a failed
  mandatory check
- **THEN** the exit code is zero on green and nonzero exactly on fail
- **VERIFIES** [[kaya.impact.layer.gate_contract]]

#### Scenario: report_deterministic

- **WHEN** the gate runs twice on the same state
- **THEN** the two reports are byte-identical
- **VERIFIES** [[kaya.impact.layer.gate_contract]]

#### Scenario: advisory_only_review

- **WHEN** the only findings are advisory
- **THEN** the result is `review`, the exit code is zero, and each finding
  is reported with its reason
- **VERIFIES** [[kaya.impact.layer.reproducible_blockers]]
