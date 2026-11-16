---
id: kaya.promotion.layer
kind: intent
statement: "WHEN a record's epistemic category changes, THE kaya.promotion capability SHALL restrict tools to events, land AI interpretation only as proposed assertions, require a recorded human actor for every promotion to conclusion, surface pending proposals pull-only as counts at the point of use, rate-cap proposal generation to fit one session-close without expiry or masquerade as settled, and demote challenged conclusions to a challenged status without deleting their records."
---

# kaya.promotion — dual-format layer (sidecar)

## Purpose

The defining design principle, made mechanical: events are not
conclusions. A test runner reports counts; only an approved human
interpretation may say a hypothesis was falsified. AI participation is
welcome as visible, flagged proposals — never as silent establishment.
This is the capability that keeps kaya from becoming "a machine for
producing plausible research narratives from incomplete evidence".

## Constraints

| id                           | kind      | expr                                                                                                                                                  | traces_to |
|------------------------------|-----------|--------------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| category_closed              | invariant | `a record's category is exactly one of {event, assertion, conclusion}`                                                                                  | [[kaya.promotion.layer]] |
| tools_emit_events_only       | invariant | `an actor of type tool can create events and no promotion command accepts a tool actor`                                                                  | [[kaya.promotion.layer]] |
| proposal_channel             | invariant | `an assertion carries status proposed, its author, and references to the events it interprets`                                                            | [[kaya.promotion.layer]] |
| approval_required            | invariant | `a conclusion exists only with a promotion event whose actor type is human, naming the promoted assertion id; the promotion event is a ledger record`     | [[kaya.promotion.layer]] |
| ai_proposed_only             | invariant | `an interpretation authored by an agent actor lands as a proposed assertion flagged machine-proposed — a conclusion never originates from an agent actor` | [[kaya.promotion.layer]] |
| approval_provenance          | invariant | `every promotion event names its approver and the promoted assertion id and persists in the ledger`                                                       | [[kaya.promotion.layer]] |
| challenge_without_deletion   | invariant | `a counter-assertion citing counter-events demotes a conclusion's displayed status to challenged; the conclusion's record remains byte-intact`            | [[kaya.promotion.layer]] |
| pending_proposals_bounded    | invariant | `pending proposals fit one session-close; over-cap proposal generation is ranked/capped, never pushed onto the human; proposals never expire and never masquerade as settled` | [[kaya.promotion.layer]] |
| proposal_surfacing_pulled    | invariant | `pending proposals surface only as counts at the point of use (kaya status, dossier build, dossier read) and no command blocks on or prompts for approval` | [[kaya.promotion.layer]] |

## Model

### States
- `observed`
- `proposed`
- `established`
- `challenged`

### Transitions

| id        | from         | to           | guard                                                                        |
|-----------|--------------|--------------|------------------------------------------------------------------------------|
| propose   | observed     | proposed     | [[kaya.promotion.layer.proposal_channel]] ∧ actor.type ∈ {human, agent}            |
| establish | proposed     | established  | [[kaya.promotion.layer.approval_required]] ∧ [[kaya.promotion.layer.approval_provenance]] ∧ actor.type == human |
| challenge | established  | challenged   | [[kaya.promotion.layer.challenge_without_deletion]] ∧ counter-events referenced     |
| resolve   | challenged   | established  | [[kaya.promotion.layer.approval_required]] ∧ actor.type == human                   |

## Properties

| id                            | kind | derives_from                              | generator                             | predicate                                                   |
|-------------------------------|------|-------------------------------------------|---------------------------------------|--------------------------------------------------------------|
| tool_promotion_blocked        | unit | [[kaya.promotion.layer.tools_emit_events_only]] | `tool_actor_promotion_call()`         | `refusal naming tool-writable categories`                     |
| unapproved_conclusion_refused | unit | [[kaya.promotion.layer.approval_required]]      | `conclusion_without_promotion_event()`| `write refused naming the missing approval`                   |
| proposed_visible              | unit | [[kaya.promotion.layer.ai_proposed_only]]       | `agent_authored_interpretation()`     | `stored as machine-proposed assertion ∧ no conclusion`        |
| approval_provenance_recorded  | unit | [[kaya.promotion.layer.approval_provenance]]    | `human_approval_of_assertion()`       | `promotion event names approver ∧ assertion id ∧ persisted`   |
| challenge_demotes_status      | unit | [[kaya.promotion.layer.challenge_without_deletion]] | `counter_assertion_on_conclusion()` | `status == challenged ∧ original record byte-intact`          |
| category_closed_checked       | unit | [[kaya.promotion.layer.category_closed]]  | `record_with_foreign_category()`      | `ingest ⇒ refusal naming the closed categories`               |
| proposal_channel_checked      | unit | [[kaya.promotion.layer.proposal_channel]] | `assertion_without_status_author()`   | `write ⇒ refusal naming status ∧ author ∧ event references`   |
| pending_proposals_bounded_checked | unit | [[kaya.promotion.layer.pending_proposals_bounded]] | `proposal_burst_over_one_session_close()` | `over-cap ⇒ generation ranks/caps ∧ approval queue not grown ∧ human never prompted to approve more` |
| proposal_surfacing_pulled_checked | unit | [[kaya.promotion.layer.proposal_surfacing_pulled]] | `pending_proposals_at_session_close()` | `counts visible ∧ no blocking prompt ∧ human not asked to approve` |

## Requirements
### Requirement: Layer sidecar contract

This sidecar is the dual-format layer for the `kaya.promotion` capability
and SHALL be deployed verbatim by the archive companion; the plain delta in
`spec.md` beside it is its openspec face.

#### Scenario: Archive deploys the layer

- **WHEN** the change `add-core-research-record` is archived after its
  scenarios have executing contract tests
- **THEN** the capability deploys with this layer attached fail-closed
