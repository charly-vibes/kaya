---
tags: [pipeline-run:tdd-ro5u-2026-11-23-kaya-7pj-test-report-input-on-the-stable-toolchain, pipeline-step:refactor]
---

REFACTOR kaya-7pj: two tidies inside the new script, both behavior-preserving.

1. scripts/junit-to-libtest.py - the outcome check reads as set intersection ('children & FAILED') instead of an 'any(... in ...)' scan over the same set, and FAILED/SKIPPED are sets rather than tuples, so the membership test is the operation it is. The mapping is unchanged: <failure> and <error> fail, <skipped> is ignored, everything else passed.

2. scripts/junit-to-libtest.py - the comprehension variable in the JSONL writer no longer shadows the record() function it is a product of ('entry' rather than 'record'), so the name carries one meaning in the module.

Verification after the refactor: cargo test --test test_report_input 3 passed 0 failed; the script's --help still prints its purpose.
