---
id: kaya.ledger.layer
kind: intent
statement: "THE kaya.ledger capability SHALL persist admitted events as an append-only JSONL stream whose written bytes are immutable, where corrections are new superseding events, identical replays are acknowledged without duplication, and every read verifies a content digest before serving."
---

# kaya.ledger — dual-format layer (sidecar)

## Purpose

Preserve what happened as immutable evidence. The ledger is the ground
truth every projection derives from: plain JSONL so any language can append,
git-versionable without a database, digest-checked so projections never
serve from silently mutated history, and append-only at the byte level so
the evidence record cannot be rewritten after the fact.

## Constraints

| id                       | kind      | expr                                                                                                                                                     | traces_to |
|--------------------------|-----------|----------------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| append_only_bytes        | invariant | `a written line's bytes are immutable; the sole mutation of the ledger file is appending lines`                                                            | [[kaya.ledger.layer]] |
| correction_via_supersede | invariant | `a correction is a new event whose data names the superseded event id; the superseded line remains byte-identical`                                          | [[kaya.ledger.layer]] |
| idempotent_replay        | invariant | `appending a record byte-identical to a stored one is acknowledged as a duplicate and changes no bytes`                                                     | [[kaya.ledger.layer]] |
| digest_bound             | invariant | `the ledger digest is the ordered SHA-256 over lines; a read refuses a mismatched ledger with a labeled error before serving anything`                      | [[kaya.ledger.layer]] |
| atomic_lines             | invariant | `lines are written atomically: a crash leaves at most one truncated final line, which recovery truncates and reports before any read`                       | [[kaya.ledger.layer]] |
| git_versionable          | invariant | `the ledger is one plain file under version control — no database, no sidecar state; a commit's diff is appended lines only`                                | [[kaya.ledger.layer]] |

## Model

### States
- `open`
- `verified`
- `refused`

### Transitions

| id       | from     | to       | guard                                                   |
|----------|----------|----------|---------------------------------------------------------|
| recover  | open     | open     | `final line truncated ⇒ truncate ∧ report`              |
| verify   | open     | verified | [[kaya.ledger.layer.digest_bound]] ∧ [[kaya.ledger.layer.atomic_lines]] |
| refuse   | open     | refused  | `¬[[kaya.ledger.layer.digest_bound]] ∧ digest mismatch recorded` |

## Properties

| id                       | kind | derives_from                           | generator                       | predicate                                                   |
|--------------------------|------|----------------------------------------|---------------------------------|--------------------------------------------------------------|
| append_only_checked      | unit | [[kaya.ledger.layer.append_only_bytes]]      | `arbitrary_session_of_appends()` | `∀ write: diff(bytes_before, bytes_after) is appended-only`   |
| supersede_round_trip     | unit | [[kaya.ledger.layer.correction_via_supersede]] | `stored_event_with_correction()` | `original bytes unchanged ∧ readers resolve to correction`    |
| replay_dedupe            | unit | [[kaya.ledger.layer.idempotent_replay]]      | `same_record_appended_twice()`  | `second append ⇒ duplicate ack ∧ bytes unchanged`             |
| digest_stability         | unit | [[kaya.ledger.layer.digest_bound]]           | `same_ledger_twice()`           | `digest(content) == digest(content)`                          |
| crash_tail_checked       | unit | [[kaya.ledger.layer.atomic_lines]]           | `ledger_with_truncated_tail()`  | `recovery truncates ∧ reports ∧ subsequent read succeeds`     |
| git_diffable             | unit | [[kaya.ledger.layer.git_versionable]]        | `ledger_with_appended_lines()`  | `git diff shows additions only`                               |

## Requirements
### Requirement: Layer sidecar contract

This sidecar is the dual-format layer for the `kaya.ledger` capability and
SHALL be deployed verbatim by the archive companion; the plain delta in
`spec.md` beside it is its openspec face.

#### Scenario: Archive deploys the layer

- **WHEN** the change `add-core-research-record` is archived after its
  scenarios have executing contract tests
- **THEN** the capability deploys with this layer attached fail-closed
