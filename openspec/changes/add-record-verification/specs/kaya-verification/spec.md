## ADDED Requirements

### Requirement: Read-only verification

`kaya check` SHALL compute the projections that `build` and `publish` would
write and report byte differences without creating, modifying, or deleting any
file, and without appending to the ledger. It SHALL verify the ledger's
committed history with the same rules as `build` and SHALL refuse an
unverifiable ledger before reporting any finding.

#### Scenario: Drift reported without writing

- **WHEN** `kaya check` runs over a record whose generated block has drifted
- **THEN** every file's bytes and the ledger bytes are identical before and
  after the run, no directory is created, and the drift is reported

### Requirement: No relaxed verification

`kaya check` SHALL provide no option that relaxes committed-history
verification. A worktree ledger that does not extend its committed bytes SHALL
be refused, and staged content SHALL be verified with `--index`.

#### Scenario: Unverifiable ledger refused

- **WHEN** the worktree ledger does not extend its committed `HEAD` bytes
- **THEN** `check` fails with `kaya.ledger.uncommitted_history` and reports no
  drift findings

### Requirement: Staged-content verification

With `--index`, `kaya check` SHALL read `.kaya/ledger.jsonl` and every
projection document from the git index (`:<path>`), so the result matches the
tree a commit would produce, and SHALL NOT consult the worktree for those
paths. When `HEAD` carries no copy of the ledger, the staged ledger SHALL be
treated as having empty committed history for this mode.

#### Scenario: Partial staging is honored

- **WHEN** a generated document is staged with drift while the worktree copy
  is clean
- **THEN** `--index` reports the drift

#### Scenario: Worktree noise is ignored

- **WHEN** the worktree copy of a generated document is drifted while the
  staged copy is clean
- **THEN** `--index` reports no drift

#### Scenario: First commit is verifiable

- **WHEN** the ledger is staged in a repository whose `HEAD` does not contain
  it
- **THEN** `--index` verifies the staged record instead of refusing it

### Requirement: Index read failures

When a compared path cannot be read from the index — an unmerged entry or a
git failure — `kaya check` SHALL fail with `kaya.verification.check_io` naming
the path and SHALL NOT fall back to the worktree.

#### Scenario: Unreadable index path fails loudly

- **WHEN** a compared path is unmerged or `git show :<path>` fails
- **THEN** `check` exits 2 with `kaya.verification.check_io` naming the path
  and does not read the worktree instead

### Requirement: Drift report

`kaya check` SHALL emit one envelope whose `drift` entries name the document
path, the block name when the finding concerns a managed block, and the
violation kind (`missing`, `extra`, or `changed`). The envelope SHALL report
`ok: true` for an executed check; the exit code carries the verdict.

#### Scenario: Drifted block

- **WHEN** a managed block's committed bytes differ from the projection
- **THEN** `check` exits 1 and names the document path, the block name, and
  `changed`

#### Scenario: Clean record

- **WHEN** the committed record matches its projections
- **THEN** `check` exits 0 with an empty `drift` list

#### Scenario: Missing generated document

- **WHEN** the projection emits a document that the worktree or index lacks
- **THEN** `check` exits 1 and names the document with kind `missing`

### Requirement: Exit contract

`kaya check` SHALL exit 0 when no defect and no enabled assertion applies, 1
when a defect or an enabled assertion or enforcement flag applies, and 2 for
usage or I/O failure. Failures SHALL use the `kaya.verification.*` id
namespace.

#### Scenario: Usage error exits 2

- **WHEN** `check` is invoked with an invalid flag value
- **THEN** it exits 2 with a `kaya.verification.*` error envelope

### Requirement: Reported paths and projection roots

Reported paths SHALL be relative to the invocation root. Projections default
to `docs/research` and `docs/public`, moved by `--out` and `--publish-out`;
`--root` sets the invocation root and the ledger location.

#### Scenario: Paths follow the root

- **WHEN** `check --root` names a repository root from a subdirectory
- **THEN** reported document paths are relative to that root

### Requirement: Record facts for policy

`kaya check` SHALL report `events`, `ledger_digest`, `last_capture_commit`,
`newest_event_at`, `events_by_source`, `emitted_kinds`, and `omitted_kinds`, so
a repository can assert currency and coverage without reading the ledger
itself. Kaya SHALL NOT evaluate thresholds against these facts.

#### Scenario: Facts without a second parser

- **WHEN** the record contains events from more than one source
- **THEN** the report carries the event count, digest, capture frontier,
  per-source counts, and kind lists

### Requirement: Capture frontier fact

`last_capture_commit` SHALL be the newest commit named by a
`capture.commit.observed` event, or null when none exists. A commit that is no
longer reachable SHALL still be reported as recorded.

#### Scenario: Unreachable capture commit

- **WHEN** the newest captured commit was rewritten by a rebase
- **THEN** `last_capture_commit` still names the recorded sha

### Requirement: Canonical source and kind vocabulary

`events_by_source` SHALL use the canonical source names `wai`, `git`,
`test-runner`, and `model`, derived from the event's capture family and
provenance — never from the actor name. `emitted_kinds` and `omitted_kinds`
SHALL be dotted `family.kind` names; an omitted entry's `family` and `kind`
SHALL be joined with a dot.

#### Scenario: Sources and kinds are canonical

- **WHEN** the record holds seed model events, wai artifacts, git commits, and
  no run events
- **THEN** the counts appear under `model`, `wai`, and `git` (never a human
  actor name), `test-runner` is zero, and both kind lists use dotted names

### Requirement: Assertions on request

`--require-kind <kind>` and `--require-source <source>` SHALL be repeatable.
`<kind>` SHALL be a dotted `family.kind` name as reported by `emitted_kinds` and
`omitted_kinds`; `<source>` SHALL be one of `wai`, `git`, `test-runner`, or
`model`. A required kind that the projection omits for `no-records`, or a
required source with zero events, SHALL be a violation naming the missing item
and its reason.

#### Scenario: Required kind omitted

- **WHEN** `--require-kind understanding.decision-record` is passed and the
  record holds no decision
- **THEN** `check` exits 1 and names the kind with reason `no-records`

#### Scenario: Required source absent

- **WHEN** `--require-source test-runner` is passed and the ledger holds no
  test-run events
- **THEN** `check` exits 1 and names the source

### Requirement: Staleness enforcement

`--fail-on-stale-authored` SHALL convert a non-empty stale report into a
violation naming the block and its evidence ids. Without the flag, `check`
SHALL report staleness and exit 0. `kaya build` SHALL keep reporting staleness
without failing.

#### Scenario: Stale authored block fails the check

- **WHEN** a correction supersedes an event cited by an authored block and
  `--fail-on-stale-authored` is passed
- **THEN** `check` exits 1 and names the block and the changed evidence ids

#### Scenario: Advisory by default

- **WHEN** the same stale block is checked without the flag
- **THEN** `check` exits 0 and reports the block as stale

### Requirement: Property coverage mirror

Every property row in the layer sidecar SHALL be verified by exactly one
dedicated scenario.

#### Scenario: no_side_effects

- **WHEN** a drifted record is checked
- **THEN** all bytes are unchanged and the drift is reported
- **VERIFIES** [[kaya.verification.read_only]]

#### Scenario: index_reads

- **WHEN** index mode runs over a partially staged tree
- **THEN** findings follow the index bytes, not the worktree
- **VERIFIES** [[kaya.verification.index_semantics]]

#### Scenario: drift_exit

- **WHEN** drift exists
- **THEN** the command exits 1 and structures the finding
- **VERIFIES** [[kaya.verification.exit_contract]]

#### Scenario: facts_reported

- **WHEN** the report is emitted
- **THEN** the capture frontier, canonical per-source counts, and dotted kind
  names are present
- **VERIFIES** [[kaya.verification.facts_not_policy]]

#### Scenario: required_items

- **WHEN** a required kind or source is absent
- **THEN** the command exits 1 naming it
- **VERIFIES** [[kaya.verification.assertions_opt_in]]

#### Scenario: stale_violation

- **WHEN** staleness is enforced
- **THEN** the command exits 1 naming block and evidence, bytes intact
- **VERIFIES** [[kaya.verification.stale_enforcement]]