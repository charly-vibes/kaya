---
tags: [pipeline-run:tdd-ro5u-2026-11-22-kaya-xjp-verification-facts-census, pipeline-step:refactor]
---

REFACTOR kaya-xjp: two tidies, both behavior-preserving, both inside the ticket's own surface.

1. src/verification/facts.rs drops its four private event-type constants (CAPTURE_COMMIT, RUN_COMPLETED, CAPTURE_ARTIFACT, and the CAPTURE_ENTRY this ticket added) and imports the four already-public constants src/model.rs publishes for exactly these event types: ARTIFACT_OBSERVED_EVENT, COMMIT_OBSERVED_EVENT, RUN_COMPLETED_EVENT, ENTRY_OBSERVED_EVENT. The module had been duplicating the model's vocabulary as private literals; adding a fifth source would have extended that duplication. Now the event-type strings live in one place, and a change to the model's vocabulary cannot silently desynchronize the census. The classification arms are unchanged in meaning - only the constant names moved.

2. tests/verification.rs: the import/build/publish/commit sequence appeared twice in the new turu tests; it is now one helper, record_with_turu_source(root), mirroring the existing record_with_two_sources(root, limit) convention in the same file.

No behavior change, no new names beyond the helper, no scope expansion: the green implementation's observable contract is identical.

Verification after the refactor: cargo test --test verification 35 passed 0 failed; cargo test 371 tests passed across 31 suites; cargo clippy --all-targets --all-features -- -D warnings clean; cargo fmt --all applied (it reformatted only the lines this ticket touched - git status shows no unrelated file modified).
