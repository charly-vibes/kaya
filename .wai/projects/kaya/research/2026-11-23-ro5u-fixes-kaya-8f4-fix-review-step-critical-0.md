---
tags: [pipeline-run:tdd-ro5u-2026-11-23-kaya-8f4-publication-block-vocabulary, pipeline-step:fix-review]
---

RO5U-FIXES kaya-8f4 (fix-review step): CRITICAL 0, HIGH 0, MEDIUM 0 — nothing to fix. Two LOW findings recorded as boundaries, neither a defect of this change and both cheaper to record than to expand:

LOW-001 the vocabulary admits collection-tier block names (publication.collection.<name>) while collections are unimplemented - consistent with the closed tier vocabulary the publications spec states (the collection tier is closed; whether a collection can be declared is separate from whether the block name is in vocabulary), and forward-compatible with the collection work.

LOW-002 a page file whose name the build neither declares nor omits is never read, so its blocks are never validated - the build's pre-existing contract (it touches the pages it owns), unchanged by this ticket; the orphan-page concern is the stale-page-removal surface, not the block vocabulary this ticket decides.

Review artifact: reviews/2026-11-23-ro5u-review-kaya-8f4-publication-block-vocab.md; verdict READY, CRITICAL 0, CONVERGED.

Re-verified green: cargo test --test publications 30 passed 0 failed; cargo test 379 passed across 32 suites; cargo clippy --all-targets --all-features -- -D warnings clean; cargo fmt --all --check clean; ah check --run-tests 184 passed 0 findings; openspec validate --specs --strict 9 passed 0 failed; spk lint openspec/specs 0 issues.
