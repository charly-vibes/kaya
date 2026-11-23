---
id: kaya.verification.layer
kind: intent
statement: "WHEN a repository must know whether its committed record matches its projections, THE kaya.verification capability SHALL compare expected and committed bytes without writing anything, honor the git index when asked, report structured findings with an explicit exit contract, surface record facts for repository policy, and enforce staleness only when asked."
---

# kaya.verification — dual-format layer (sidecar)

## Purpose

The compiler writes; something must be able to say "this is wrong" without
also fixing it. Verification is a separate consumer of the projection and the
ledger: it renders the expected bytes in memory, compares them with the
committed candidate (worktree or index), and reports. That separation is what
makes a pre-commit gate possible at all — a writer in a hook repairs the tree
behind the author's back, and a command that insists on `HEAD` cannot check a
staged ledger.

## Constraints

| id                    | kind      | expr                                                                                                                                                                             | traces_to |
|-----------------------|-----------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| read_only             | invariant | `verification creates, modifies, and deletes no file, appends no event, and refuses an unverifiable ledger instead of relaxing verification; expected bytes exist only in memory`                                                                       | [[kaya.verification.layer]] |
| index_semantics       | invariant | `with --index every compared path is read from the git index (:path); the worktree is never consulted for those paths`                                                                | [[kaya.verification.layer]] |
| exit_contract         | invariant | `exit 0 iff no violation applies; 1 iff a violation applies; 2 for usage or I/O failure; violations are structured and named, never prose`                                              | [[kaya.verification.layer]] |
| facts_not_policy      | invariant | `verification reports record facts (capture frontier; counts under canonical sources wai, git, test-runner, model, turu; dotted family.kind lists) and evaluates no threshold or allowlist`                                             | [[kaya.verification.layer]] |
| assertions_opt_in     | invariant | `a required kind (dotted family.kind) or source (canonical name) fails only when requested by a flag; the repository owns the list`                                                                                          | [[kaya.verification.layer]] |
| stale_enforcement     | invariant | `staleness fails the run only under --fail-on-stale-authored; build keeps reporting without failing`                                                                                    | [[kaya.verification.layer]] |

## Model

### States
- `unverified`
- `clean`
- `drifted`
- `refused`

### Transitions

| id        | from       | to        | guard                                                                                          |
|-----------|------------|-----------|------------------------------------------------------------------------------------------------|
| verify    | unverified | clean     | [[kaya.verification.layer.read_only]] ∧ [[kaya.verification.layer.exit_contract]]              |
| report    | unverified | drifted   | [[kaya.verification.layer.read_only]] ∧ [[kaya.verification.layer.exit_contract]]              |
| refuse    | unverified | refused   | ledger history unverifiable                                                                     |

## Properties

| id                 | kind | derives_from                                  | generator                              | predicate                                                       |
|--------------------|------|-----------------------------------------------|----------------------------------------|-----------------------------------------------------------------|
| no_side_effects    | unit | [[kaya.verification.layer.read_only]]          | `drifted_record_checked()`              | `tree hash identical before and after ∧ drift reported`          |
| index_bytes        | unit | [[kaya.verification.layer.index_semantics]]    | `partially_staged_tree()`               | `findings follow :path bytes, not worktree bytes`                |
| drift_exit         | unit | [[kaya.verification.layer.exit_contract]]      | `drifted_block()`                       | `exit 1 ∧ drift names path, block, kind`                          |
| facts_reported     | unit | [[kaya.verification.layer.facts_not_policy]]   | `record_with_two_sources()`             | `last_capture_commit ∧ events_by_source ∧ kind lists present`    |
| required_items     | unit | [[kaya.verification.layer.assertions_opt_in]]  | `record_missing_required_kind()`        | `exit 1 ∧ kind named with reason`                                 |
| stale_violation    | unit | [[kaya.verification.layer.stale_enforcement]]  | `stale_block_under_flag()`              | `exit 1 ∧ block and evidence ids named ∧ bytes intact`            |

## Requirements

### Requirement: Layer sidecar contract

This sidecar is the dual-format layer for the `kaya.verification` capability
and SHALL be deployed verbatim by the archive companion; the plain delta in
`spec.md` beside it is its openspec face.

#### Scenario: Archive deploys the layer

- **WHEN** the change `add-record-verification` is archived after its
  scenarios have executing contract tests
- **THEN** the capability deploys with this layer attached fail-closed