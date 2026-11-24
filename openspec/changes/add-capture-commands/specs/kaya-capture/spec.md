## ADDED Requirements

### Requirement: Capture commands record one model event each

Each capture command SHALL append at most one `model.entity.created`
event — exactly one when it appends — mapping the command text onto its
matching closed-vocabulary entity type, and SHALL print one JSON
envelope naming the outcome (`appended`, `duplicate`, or `corrected`)
and the event id (the stored event's id when nothing is appended).

#### Scenario: note_command_creates_an_observation_event

- **WHEN** `kaya note "buffers drain on shutdown" --actor sasha` runs
  against an existing ledger
- **THEN** exactly one `model.entity.created` event is appended whose
  subject type is `Observation` and the envelope reports `appended`

#### Scenario: question_command_creates_a_question_event

- **WHEN** `kaya question "does the tail repair preserve digests?"
  --actor sasha` runs against an existing ledger
- **THEN** exactly one `model.entity.created` event is appended whose
  subject type is `Question`

#### Scenario: decision_command_creates_a_decision_event

- **WHEN** `kaya decision "keep JSONL, no database" --actor charly`
  runs against an existing ledger
- **THEN** exactly one `model.entity.created` event is appended whose
  subject type is `Decision`

#### Scenario: finding_command_creates_a_finding_event

- **WHEN** `kaya finding "recovery appends, never truncates complete
  lines" --actor charly` runs against an existing ledger
- **THEN** exactly one `model.entity.created` event is appended whose
  subject type is `Finding`

#### Scenario: capture_against_a_missing_ledger_refused

- **WHEN** a capture command runs in a directory with no
  `.kaya/ledger.jsonl`
- **THEN** the command refuses with `missing-ledger`, names
  `kaya init`, and appends nothing

### Requirement: Capture events carry the shipped model shape

Every capture event SHALL use subject id `<project>/<TYPE>/<key>` — the
namespace-qualified node id the graph builder already resolves — and
SHALL carry exactly the data keys `text` (the redacted command text) and
`key` (the node key), plus `supersedes` (the prior effective event id)
on correction events. Its provenance SHALL name tool `kaya`, the CLI
version, and the reference `<subcommand>/<key>` — a ledger-internal
label by design, not a resolvable path.

#### Scenario: capture_subject_id_is_namespace_qualified

- **WHEN** `kaya note "..." --actor sasha --key drain-behaviour
  --project kaya` runs
- **THEN** the event's subject id is `kaya/Observation/drain-behaviour`

#### Scenario: capture_data_carries_exactly_text_and_key

- **WHEN** a capture command appends a first capture for its key
- **THEN** the event's data is exactly `{text, key}` with no other keys
  and no interpretation keys
- **THEN** on a correction the data is exactly `{text, key}` plus one
  further key `supersedes`

#### Scenario: capture_provenance_names_kaya_and_the_subcommand

- **WHEN** `kaya decision "..." --actor charly --key storage` runs
- **THEN** the event's provenance is tool `kaya` with reference
  `decision/storage`

### Requirement: Payload bounds refuse, never truncate

The command text SHALL be non-empty after redaction and at most 2000
characters; the key SHALL be 1–64 characters of `[a-z0-9-]`. An empty
text SHALL be refused with `kaya.capture.empty_text` and an over-bound
payload with `kaya.capture.bound_exceeded`, naming the field and its
limit; nothing is appended. Capture output is authored text: never
silently truncated, never silently rewritten.

#### Scenario: capture_text_over_bound_refused_without_truncation

- **WHEN** a capture command is given a text of 2001 characters
- **THEN** the command refuses with `kaya.capture.bound_exceeded`
  naming `text` and 2000, and the ledger gains no event

#### Scenario: capture_key_over_bound_or_off_alphabet_refused

- **WHEN** a capture command is given a key longer than 64 characters
  or containing a character outside `[a-z0-9-]`
- **THEN** the command refuses with `kaya.capture.bound_exceeded`
  naming `key`, and the ledger gains no event

#### Scenario: capture_empty_key_or_empty_text_refused

- **WHEN** a capture command is given an empty key or empty text
- **THEN** it refuses — `kaya.capture.empty_text` for the text,
  `kaya.capture.bound_exceeded` naming `key` for the key — and the
  ledger gains no event

### Requirement: Pasted secrets are redacted before storage

Every capture command SHALL pass its text through the redaction filter
before bounding, id derivation, and append. The filter SHALL replace
each matched secret with `[redacted]` in place, preserving the
surrounding text — never the whole attribute, never a silent truncation
— SHALL treat a base64 or hex run of ≥ 32 characters as secret
by default, and SHALL apply the full add-agent-session-capture pattern
list, enumerated by
`redaction_filter_covers_the_inherited_pattern_list`.

#### Scenario: pasted_api_key_replaced_in_place_with_marker

- **WHEN** a capture text contains `sk-proj-abc123def456ghijkl` between
  other words
- **THEN** the stored `data.text` reads the surrounding words with
  `[redacted]` where the token was, and the token appears nowhere in the
  ledger

#### Scenario: high_entropy_run_treated_as_secret

- **WHEN** a capture text contains an unlisted base64 run of 40
  characters
- **THEN** the stored `data.text` carries `[redacted]` in its place

#### Scenario: benign_text_survives_redaction_unchanged

- **WHEN** a capture text names `kaya.ledger.id_reuse` and the word
  `password` policy without a value
- **THEN** the stored `data.text` is byte-identical to the input text

#### Scenario: redaction_filter_covers_the_inherited_pattern_list

- **WHEN** a capture text contains representatives of the inherited
  families — a `sk-` token, a `ghp_` token, a `github_pat_` token, an
  AWS `AKIA` id, an AWS `ASIA` id, a Slack token, a JWT, private-key
  armour including PGP and `PuTTY-User-Key-File` headers, and a
  `token: value` assignment
- **THEN** every match is replaced with `[redacted]` and no other part
  of the text changes

### Requirement: Capture ids are deterministic and replay idempotent

Capture event ids SHALL be content-derived: sha256 over the
`\x1f`-joined identity payload (tool, CLI version, project, timestamp,
subject id, serialized data), `evt-` plus the first 26 hex characters —
the adapters' derivation scheme over this payload. Two captures of
different content SHALL never share an id, and the ledger's byte-exact
dedup SHALL absorb a byte-identical replay as a duplicate rather than
refusing it.

#### Scenario: redacted_text_hashes_deterministically

- **WHEN** the same text containing the same pasted secret is captured
  twice at the same timestamp
- **THEN** both invocations derive the same event id from the redacted
  bytes, and the second append is acknowledged as a byte-exact duplicate

#### Scenario: distinct_captures_never_share_an_id

- **WHEN** two different texts are captured within the same second under
  the same project
- **THEN** the two events carry different ids

### Requirement: Edits supersede, never rewrite

Re-capturing an existing key SHALL compare the new redacted text against
the key's effective stored text — the newest stored event for its
subject id in ledger order. Unchanged text SHALL append nothing and
report `duplicate`; changed text SHALL append one correction carrying
`data.supersedes` naming the prior effective id, with the
generation-salted deterministic correction id, leaving the node id and
stored bytes untouched. Re-running the same correction SHALL append
nothing.

#### Scenario: key_without_stored_text_treated_as_absent

- **WHEN** a capture command runs with a key whose subject id has
  stored events carrying no `data.text` — for example seed-imported
  attributes
- **THEN** the key counts as absent and the capture is a first capture

#### Scenario: unchanged_recapture_appends_nothing

- **WHEN** a capture command runs with a key whose effective stored text
  equals the new redacted text
- **THEN** nothing is appended and the envelope reports `duplicate`

#### Scenario: edited_capture_emits_a_superseding_correction

- **WHEN** a capture command runs with an existing key and changed text
- **THEN** one correction event is appended whose `data.supersedes`
  names the prior effective event id and whose `data.text` is the new
  redacted text

#### Scenario: correction_preserves_the_subject_id

- **WHEN** a correction is appended for `kaya/Observation/drain-behaviour`
- **THEN** the correction's subject id is unchanged and the graph's
  node id for the key never changes

#### Scenario: correction_replay_appends_nothing

- **WHEN** the same correction invocation is re-run against the same
  ledger
- **THEN** nothing is appended and the envelope reports `duplicate`

### Requirement: Only human and agent actors author captures

A capture command SHALL require `--actor NAME` and SHALL accept
`--actor-type` of `human` (the default) or `agent`; a `tool` actor type
SHALL be refused with `kaya.capture.actor_required`, appending nothing.
Captures create vocabulary nodes; they never establish conclusions, and
the promotion approval rules remain untouched.

#### Scenario: tool_actor_refused

- **WHEN** a capture command runs with `--actor-type tool`
- **THEN** the command refuses with `kaya.capture.actor_required` and
  the ledger gains no event

#### Scenario: default_actor_is_human

- **WHEN** a capture command omits `--actor-type`
- **THEN** the appended event's actor type is `human` with the
  `--actor` name

#### Scenario: agent_actor_recorded

- **WHEN** a capture command runs with `--actor-type agent`
- **THEN** the appended event's actor type is `agent` with the
  `--actor` name, carried on the stored event
