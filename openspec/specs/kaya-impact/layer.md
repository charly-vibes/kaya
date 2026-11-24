---
id: kaya.impact.layer

kind: intent

statement: "WHEN a change set is submitted, THE kaya.impact capability SHALL map it to an impact set over document IR dependency edges under explicit project policy, require exactly one closed-vocabulary disposition per impact with human dispositions recorded as human events, run the mandatory mechanical checks, treat an unacknowledged stale authored block as an unmet obligation, and emit a deterministic report whose fail result — and nonzero exit — is reserved for reproducible unmet obligations."

---

# kaya.impact — dual-format layer (sidecar)

## Purpose

The gate that answers a different question than the generator: not "what can
we publish from the available information?" but "can this change merge
without misleading readers?". The gate operates on the relationship between
changed artifacts and documented records — never on whether a markdown file
changed — and it demands a disposition for every identified impact, not a
documentation edit for each. Hard blockers are reproducible results only;
advisory findings produce review obligations, because an unreviewable gate
trains developers to bypass it.

## Constraints

| id                       | kind      | expr                                                                                                                                                                                                                                                               | traces_to |
|--------------------------|-----------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| impact_mapping           | invariant | `the impact set is exactly the document IR records whose dependency edges intersect the change set, directly or transitively; impact rules are explicit project policy maintained in the repository; the report names the edge or rule that identified each record` | [[kaya.impact.layer]] |
| dispositions_closed      | invariant | `each impact carries exactly one disposition from the closed vocabulary updated, validated, reviewed, waived; a missing or out-of-vocabulary disposition is an unmet obligation; a waived disposition binds to the record's dependency digest and lapses when the dependencies change again`                                                                                                                                     | [[kaya.impact.layer]] |
| mechanical_dispositions_derived | invariant | `validated is established by the record's mandatory checks running green on the current state, and updated by the impacted document or block regenerating with bytes different from its state before the change set; neither requires a recorded event` | [[kaya.impact.layer]] |
| human_review_events      | invariant | `reviewed and waived are recorded as events under the verification family — which the research graph ignores — with the subject typed Document and the record id, actor, and rationale for waived in the event data; only a human actor is accepted, and any other actor is a labeled refusal appending nothing`                      | [[kaya.impact.layer]] |
| stale_as_obligation      | invariant | `an authored block reported stale by the projection — a cited id superseded without the scoped acknowledgment — is an unmet obligation in the gate regardless of any other disposition; the scoped acknowledgment acts as the block's reviewed disposition`           | [[kaya.impact.layer]] |
| gate_contract            | invariant | `kaya impact emits a deterministic machine-readable report with no run timestamps or environment details; the result is fail exactly when a mandatory obligation is unmet; the exit code is nonzero exactly on fail; advisory findings are reported and never block` | [[kaya.impact.layer]] |
| reproducible_blockers    | invariant | `a hard blocker is a reproducible result — a failed mechanical check, a missing or invalid disposition, or an unacknowledged stale block; no finding that is not reproducible blocks a commit`                                                                      | [[kaya.impact.layer]] |

## Model

### States
- `submitted`
- `mapped`
- `dispositioned`
- `validated`
- `gated`

### Transitions

| id          | from           | to            | guard                                                                                     |
|-------------|----------------|---------------|---------------------------------------------------------------------------------------------|
| map         | submitted      | mapped        | [[kaya.impact.layer.impact_mapping]]                                                          |
| disposition | mapped         | dispositioned | [[kaya.impact.layer.dispositions_closed]] ∧ [[kaya.impact.layer.human_review_events]]         |
| validate    | dispositioned  | validated     | [[kaya.impact.layer.reproducible_blockers]]                                                   |
| gate        | validated      | gated         | [[kaya.impact.layer.gate_contract]] ∧ [[kaya.impact.layer.stale_as_obligation]]               |

## Properties

| id                   | kind | derives_from                                        | generator                            | predicate                                                              |
|----------------------|------|------------------------------------------------------|----------------------------------------|--------------------------------------------------------------------------|
| dependents_found     | unit | [[kaya.impact.layer.impact_mapping]]                  | `change_touching_source_path()`        | `impact set == edge-intersecting records ∧ rule named per record`          |
| unaffected_excluded  | unit | [[kaya.impact.layer.impact_mapping]]                  | `change_on_orphan_path()`              | `impact set empty ∧ report says so`                                        |
| impacts_accounted    | unit | [[kaya.impact.layer.dispositions_closed]]             | `impact_without_disposition()`         | `fail naming the unmet record`                                             |
| dispositions_derived | unit | [[kaya.impact.layer.mechanical_dispositions_derived]] | `green_checks_and_changed_bytes()`     | `validated ∧ updated in force without recorded events`                     |
| human_actor_required | unit | [[kaya.impact.layer.human_review_events]]             | `waive_with_non_human_actor()`         | `labeled refusal ∧ no event appended`                                      |
| stale_unmet          | unit | [[kaya.impact.layer.stale_as_obligation]]             | `stale_block_in_impact_set()`          | `unmet until the scoped acknowledgment is recorded`                        |
| exit_contract        | unit | [[kaya.impact.layer.gate_contract]]                   | `gate_on_green_and_broken_states()`    | `exit nonzero exactly on fail`                                             |
| report_deterministic | unit | [[kaya.impact.layer.gate_contract]]                   | `gate_twice_same_state()`              | `byte-identical reports`                                                   |
| advisory_only_review | unit | [[kaya.impact.layer.reproducible_blockers]]           | `advisory_finding_only_state()`        | `result review ∧ exit zero ∧ finding reported with reason`                 |

## Requirements

### Requirement: Layer sidecar contract

This sidecar is the dual-format layer for the `kaya.impact` capability and
SHALL be deployed verbatim by the archive companion; the plain delta in
`spec.md` beside it is its openspec face.

#### Scenario: Archive deploys the layer

- **WHEN** the change `add-document-ir-commit-gate` is archived after its
  scenarios have executing contract tests
- **THEN** the capability deploys with this layer attached fail-closed
