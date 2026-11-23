---
tags: [pipeline-run:tdd-ro5u-2026-11-22-kaya-xjp-verification-facts-census, pipeline-step:fix-review]
---

RO5U-FIXES kaya-xjp: every HIGH and MEDIUM finding closed; the LOW binding-header finding is recorded, not changed (it matches the deployed capability's current format).

HIGH-001 README vocabulary - fixed: README.md now reads '--require-source <wai|git|test-runner|model|turu>'. The README claim matches the shipped behavior and the deployed 'Assertions on request' requirement.

MEDIUM-001 deployed constraint row vs archive - fixed: the archived add-turu-source-row proposal.md and tasks.md now record the deployed facts_not_policy constraint-row edit explicitly and state why it is a direct deployed edit (an openspec archive merges requirement deltas only, never a capability's constraint table - confirmed by a scratch probe in /tmp/osp2). The archived add-record-verification layer.md - the delta that seeded this capability's constraint table - is synced to name turu, so the seeded table and the deployed table agree, matching the 'archived delta kept in sync' discipline commit 6b7dfd2 applied to the adapters requirement split. The deployed constraint table and the archived layer table now differ only in the pre-existing [[kaya.verification]] vs [[kaya.verification.layer]] trace tokens and a pre-existing conjunction typo, both untouched by this ticket.

MEDIUM-002 scenario text vs bound test - fixed: check_sources_and_kinds_canonical now asserts counts['turu'] == 0 ('no turu bundle was imported, so turu is zero'), matching the scenario text 'test-runner and turu are zero' in the deployed spec and the archived delta.

LOW-001 binding tool_version header - not changed, reason recorded in the review: the deployed kaya-verification bindings are a mix (0.12.3-authored, no tool_version on the newer ones), so the omission matches the deployed capability's current format and the generator's own output; adding a header the generator does not emit would be a manual divergence.

Re-verified green after the fixes: cargo fmt --all --check clean; cargo test 371 tests passed, 0 failed; cargo clippy --all-targets --all-features -- -D warnings clean; ah check --run-tests 180 passed 0 findings; openspec validate --specs --strict 9 passed 0 failed; openspec validate --archived --strict - change/2026-11-22-add-turu-source-row passes (the three failing archives - add-document-taxonomy, add-core-research-record, add-capture-attributes - predate this ticket); spk lint openspec/specs 0 issues; pretender check --diff-only --mode gate clean.
