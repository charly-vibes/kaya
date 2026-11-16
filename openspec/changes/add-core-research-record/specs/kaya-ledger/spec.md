## ADDED Requirements

### Requirement: Append-only ledger

The research ledger SHALL be a single append-only JSONL file: once a line is
written its bytes are immutable, the sole mutation is appending, and lines
are written atomically so a crash leaves at most a truncated final line that
recovery truncates away before any read.

#### Scenario: Written bytes never change

- **WHEN** any kaya command completes against an existing ledger
- **THEN** the file's byte prefix is unchanged and the only possible diff is
  appended lines

#### Scenario: Crash-tail recovery

- **WHEN** the ledger's final line is truncated (no closing newline, invalid
  JSON) at read time
- **THEN** recovery truncates the partial line, reports it, and the ledger
  reads cleanly afterwards

### Requirement: Corrections supersede, never edit

A correction SHALL be recorded as a new event of a correction type whose
`data` names the superseded event id; the superseded event's line SHALL
remain byte-identical in the ledger.

#### Scenario: Correction round trip

- **WHEN** a captured event is corrected by appending a correction event
  naming it in `data.supersedes`
- **THEN** the original line is byte-identical before and after, and graph
  readers resolve the event to its correction

### Requirement: Idempotent replay

Appending a record byte-identical to one already stored SHALL be
acknowledged as a duplicate without changing the ledger's bytes; only a
record with a fresh id appends.

#### Scenario: Duplicate replay acknowledged

- **WHEN** the same valid record (identical id and bytes) is appended twice
- **THEN** the second append returns a duplicate acknowledgment and the
  ledger file is byte-identical to before the second append

### Requirement: Digest-bound reads

Every ledger read SHALL verify the ledger digest (ordered SHA-256 over
lines) before serving projections or graph queries, and SHALL refuse a
mismatched ledger with a labeled error rather than serving partial history.

#### Scenario: Tampered ledger refused

- **WHEN** a line is edited in place after writing and a read begins
- **THEN** the digest check fails, the read is refused with a labeled error
  naming the offending line number, and no projection output is produced

### Requirement: Property coverage mirror

Every property row in the layer sidecar SHALL be verified by exactly one
dedicated scenario.

#### Scenario: append_only_checked

- **WHEN** commands append across a session
- **THEN** each write's diff against the prior bytes is appended-only
- **VERIFIES** [[kaya.ledger.append_only_bytes]]

#### Scenario: supersede_round_trip

- **WHEN** a correction event supersedes a stored event
- **THEN** the original bytes persist and readers resolve to the correction
- **VERIFIES** [[kaya.ledger.correction_via_supersede]]

#### Scenario: replay_dedupe

- **WHEN** an identical record is appended twice
- **THEN** the ledger bytes are unchanged after the second append
- **VERIFIES** [[kaya.ledger.idempotent_replay]]

#### Scenario: digest_stability

- **WHEN** the same ledger content is digested twice
- **THEN** the digests match byte-for-byte
- **VERIFIES** [[kaya.ledger.digest_bound]]

#### Scenario: crash_tail_checked

- **WHEN** a truncated final line exists
- **THEN** recovery truncates and reports before reads proceed
- **VERIFIES** [[kaya.ledger.atomic_lines]]

#### Scenario: git_diffable

- **WHEN** a ledger with appended lines is committed
- **THEN** `git diff` shows only added lines
- **VERIFIES** [[kaya.ledger.git_versionable]]
