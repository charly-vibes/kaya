---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-8ot-kaya-check-facts, pipeline-step:fix-review]
---

RO5U-FIXES kaya-8ot: frontier now selects among capture events that name a string sha, ordered by parsed RFC3339 timestamp then id, so unnamed observations cannot mask older named commits and fractional seconds order correctly. Added test-runner import coverage, a two-commit/author-date frontier test with an independent expectation, and rebound facts-without-a-second-parser to the multi-source test-runner test. Verified: verification suite 23/23; full cargo test green; clippy -D warnings clean; fmt.

