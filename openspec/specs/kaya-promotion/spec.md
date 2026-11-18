---
id: kaya.promotion
kind: intent
statement: "WHEN a record's epistemic category changes, THE kaya.promotion capability SHALL restrict tools to events, land AI interpretation only as proposed assertions, require a recorded human actor for every promotion to conclusion, surface pending proposals pull-only as counts at the point of use, rate-cap proposal generation to fit one session-close without expiry or masquerade as settled, and demote challenged conclusions to a challenged status without deleting their records."
---

# kaya-promotion Specification

## Purpose
The epistemic rules that separate observation from interpretation: tools
record events, AI interpretation lands only as proposed assertions, and every
promotion to conclusion names a recorded human actor. Pending proposals
surface pull-only as counts, are rate-capped to fit one session-close without
expiring or masquerading as settled, and a challenged conclusion demotes to a
challenged status without deleting its record.

## Constraints

| id                           | kind      | expr                                                                                                                                                  | traces_to |
|------------------------------|-----------|--------------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| category_closed              | invariant | `a record's category is exactly one of {event, assertion, conclusion}`                                                                                  | [[kaya.promotion]] |
| tools_emit_events_only       | invariant | `an actor of type tool can create events and no promotion command accepts a tool actor`                                                                  | [[kaya.promotion]] |
| proposal_channel             | invariant | `an assertion carries status proposed, its author, and references to the events it interprets`                                                            | [[kaya.promotion]] |
| approval_required            | invariant | `a conclusion exists only with a promotion event whose actor type is human, naming the promoted assertion id; the promotion event is a ledger record`     | [[kaya.promotion]] |
| ai_proposed_only             | invariant | `an interpretation authored by an agent actor lands as a proposed assertion flagged machine-proposed — a conclusion never originates from an agent actor` | [[kaya.promotion]] |
| approval_provenance          | invariant | `every promotion event names its approver and the promoted assertion id and persists in the ledger`                                                       | [[kaya.promotion]] |
| challenge_without_deletion   | invariant | `a counter-assertion citing counter-events demotes a conclusion's displayed status to challenged; the conclusion's record remains byte-intact`            | [[kaya.promotion]] |
| pending_proposals_bounded    | invariant | `pending proposals fit one session-close; over-cap proposal generation is ranked/capped, never pushed onto the human; proposals never expire and never masquerade as settled` | [[kaya.promotion]] |
| proposal_surfacing_pulled    | invariant | `pending proposals surface only as counts at the point of use (kaya status, dossier build, dossier read) and no command blocks on or prompts for approval` | [[kaya.promotion]] |

## Model

### States
- `observed`
- `proposed`
- `established`
- `challenged`

### Transitions

| id        | from         | to           | guard                                                                        |
|-----------|--------------|--------------|------------------------------------------------------------------------------|
| propose   | observed     | proposed     | [[kaya.promotion.proposal_channel]] ∧ actor.type ∈ {human, agent}            |
| establish | proposed     | established  | [[kaya.promotion.approval_required]] ∧ [[kaya.promotion.approval_provenance]] ∧ actor.type == human |
| challenge | established  | challenged   | [[kaya.promotion.challenge_without_deletion]] ∧ counter-events referenced     |
| resolve   | challenged   | established  | [[kaya.promotion.approval_required]] ∧ actor.type == human                   |

## Properties

| id                            | kind | derives_from                              | generator                             | predicate                                                   |
|-------------------------------|------|-------------------------------------------|---------------------------------------|--------------------------------------------------------------|
| tool_promotion_blocked        | unit | [[kaya.promotion.tools_emit_events_only]] | `tool_actor_promotion_call()`         | `refusal naming tool-writable categories`                     |
| unapproved_conclusion_refused | unit | [[kaya.promotion.approval_required]]      | `conclusion_without_promotion_event()`| `write refused naming the missing approval`                   |
| proposed_visible              | unit | [[kaya.promotion.ai_proposed_only]]       | `agent_authored_interpretation()`     | `stored as machine-proposed assertion ∧ no conclusion`        |
| approval_provenance_recorded  | unit | [[kaya.promotion.approval_provenance]]    | `human_approval_of_assertion()`       | `promotion event names approver ∧ assertion id ∧ persisted`   |
| challenge_demotes_status      | unit | [[kaya.promotion.challenge_without_deletion]] | `counter_assertion_on_conclusion()` | `status == challenged ∧ original record byte-intact`          |
| category_closed_checked       | unit | [[kaya.promotion.category_closed]]  | `record_with_foreign_category()`      | `ingest ⇒ refusal naming the closed categories`               |
| proposal_channel_checked      | unit | [[kaya.promotion.proposal_channel]] | `assertion_without_status_author()`   | `write ⇒ refusal naming status ∧ author ∧ event references`   |
| pending_proposals_bounded_checked | unit | [[kaya.promotion.pending_proposals_bounded]] | `proposal_burst_over_one_session_close()` | `over-cap ⇒ generation ranks/caps ∧ approval queue not grown ∧ human never prompted to approve more` |
| proposal_surfacing_pulled_checked | unit | [[kaya.promotion.proposal_surfacing_pulled]] | `pending_proposals_at_session_close()` | `counts visible ∧ no blocking prompt ∧ human not asked to approve` |

## Requirements

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

#### Scenario: Conclusion write without a promotion event refused

- **WHEN** a conclusion write lacks a promotion event with a human actor
- **THEN** the write is refused, naming the missing approval

#### Scenario: AI interpretation flagged

- **WHEN** an agent actor records an interpretation
- **THEN** it is stored as a `proposed` assertion flagged
  machine-proposed, and no conclusion exists for it

#### Scenario: Approval event names its human actor

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

### Requirement: Proposal budget is pulled, never pushed

Promotion approval SHALL be pulled at the point of use: a pending proposal
SHALL surface as a count in `kaya status` and on dossier build and dossier
read, and no command SHALL block on or prompt for approval. Pending
proposals SHALL fit one session-close; where they would not, proposal
generation SHALL be ranked or capped instead of growing the human's
approval queue. An unapproved proposal SHALL remain visibly `proposed`
indefinitely and SHALL never be displayed as an established conclusion.

#### Scenario: Proposals surface without interrupting

- **WHEN** proposals are pending
- **THEN** they appear as a count in `kaya status` and on dossier build and
  read, and no command blocks or prompts for approval

#### Scenario: Pending proposals fit one session-close

- **WHEN** pending proposals exceed what one session-close can approve
- **THEN** generation ranks or caps proposals instead of increasing the
  human's approval queue

#### Scenario: Proposals never expire or settle silently

- **WHEN** a proposal is unapproved
- **THEN** it stays visibly `proposed` indefinitely and is never displayed
  as an established conclusion

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

#### Scenario: pending_proposals_bounded_checked

- **WHEN** pending proposals exceed one session-close's approval budget
- **THEN** generation ranks or caps them, the approval queue does not grow,
  and the human is never prompted to approve more
- **VERIFIES** [[kaya.promotion.pending_proposals_bounded_checked]]

#### Scenario: proposal_surfacing_pulled_checked

- **WHEN** proposals are pending at `kaya status`, dossier build, or dossier
  read
- **THEN** they surface only as counts, no command blocks or prompts for
  approval, and the human is never asked to approve
- **VERIFIES** [[kaya.promotion.proposal_surfacing_pulled_checked]]
