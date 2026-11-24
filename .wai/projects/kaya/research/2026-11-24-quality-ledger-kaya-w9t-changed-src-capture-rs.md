---
tags: [pipeline-run:tdd-ro5u-2026-11-24-kaya-w9t-capture-commands-implementation, pipeline-step:ro5u-review]
---

QUALITY LEDGER kaya-w9t - Changed: src/capture.rs (new module: redaction-first filter, bounds, deterministic ids, duplicate-vs-correction predicate, refuse-not-serve stored_events), src/main.rs (mod capture; four Note/Question/Decision/Finding variants + dispatch + shared cmd_capture), tests/capture.rs (new, 30 tests incl. 3 review regressions), beads export. Review: RO5U reviewer pass, 4 findings (1 P2 hex family, 3 P3 armour/stored_events/race) all fixed. Verified: capture 30/30; FULL suite 481 passed / 0 failed; clippy -D warnings 0; cargo fmt clean. Openspec validate --strict unchanged green (no spec edits). Commit will include only src/capture.rs, src/main.rs, tests/capture.rs, beads export. Human ship-close approval: granted under the in-session orchestration pattern. No push per fleet policy.
