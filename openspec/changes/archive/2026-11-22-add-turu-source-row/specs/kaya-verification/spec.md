## MODIFIED Requirements

### Requirement: Canonical source and kind vocabulary

`events_by_source` SHALL use the canonical source names `wai`, `git`,
`test-runner`, `model`, and `turu`, derived from the event's capture family
and provenance — never from the actor name. `emitted_kinds` and `omitted_kinds`
SHALL be dotted `family.kind` names; an omitted entry's `family` and `kind`
SHALL be joined with a dot.

#### Scenario: Sources and kinds are canonical

- **WHEN** the record holds seed model events, wai artifacts, git commits, and
  no run events
- **THEN** the counts appear under `model`, `wai`, and `git` (never a human
  actor name), `test-runner` and `turu` are zero, and both kind lists use
  dotted names

#### Scenario: Turu entries count under the turu source

- **WHEN** the record holds a turu whisper-bundle entry
- **THEN** the entry is counted under `turu`, and no other source names it

### Requirement: Assertions on request

`--require-kind <kind>` and `--require-source <source>` SHALL be repeatable.
`<kind>` SHALL be a dotted `family.kind` name as reported by `emitted_kinds` and
`omitted_kinds`; `<source>` SHALL be one of `wai`, `git`, `test-runner`,
`model`, or `turu`. A required kind that the projection omits for `no-records`,
or a required source with zero events, SHALL be a violation naming the
missing item and its reason.

#### Scenario: Required kind omitted

- **WHEN** `--require-kind understanding.decision-record` is passed and the
  record holds no decision
- **THEN** `check` exits 1 and names the kind with reason `no-records`

#### Scenario: Required source absent

- **WHEN** `--require-source test-runner` is passed and the ledger holds no
  test-run events
- **THEN** `check` exits 1 and names the source

#### Scenario: Required turu source satisfied

- **WHEN** `--require-source turu` is passed and the ledger holds a turu
  entry observation
- **THEN** `check` exits 0 and reports no unmet requirement

#### Scenario: Required turu source absent

- **WHEN** `--require-source turu` is passed and the ledger holds no turu
  entry observation
- **THEN** `check` exits 1 and names `turu` with reason `no-events`
