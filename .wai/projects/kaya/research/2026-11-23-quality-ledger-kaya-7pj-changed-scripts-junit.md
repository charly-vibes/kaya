---
tags: [pipeline-run:tdd-ro5u-2026-11-23-kaya-7pj-test-report-input-on-the-stable-toolchain, pipeline-step:quality-ledger]
---

QUALITY LEDGER kaya-7pj:

Changed:
- scripts/junit-to-libtest.py (new) - converts a nextest JUnit XML report into the libtest JSONL the test-runner adapter consumes: one 'test' record per <testcase> in report order, the event named by the outcome element (<failure> or <error> fails, <skipped> is ignored, no outcome element passed), and the name qualified by the binary the test ran in. Refuses a report it cannot read, a document that states no test case, and writes the report atomically.
- tests/test_report_input.rs (new) - five cases: the conversion names every test and its outcome; a converted report imports once and dedupes the second time, with the counts reaching the record; a malformed report is refused and writes nothing; a document with no test cases is refused; a refused conversion leaves the previous report byte-identical.
- docs/src/status.md - a 'Test report input' section recording the decision, the .config/nextest.toml profile that enables nextest's JUnit report, the exact commands, the outcome mapping, and why the nightly invocation was rejected.
- README.md - the import-test line points at the decision.
- CHANGELOG.md - the stable-toolchain input path recorded under Unreleased/Added.
- .beads/issues.jsonl - the tracked bd export (kaya-7pj claimed).

Verified:
- cargo test --test test_report_input - 5 passed, 0 failed.
- cargo test - 376 tests passed, 0 failed, across 32 suites.
- cargo clippy --all-targets --all-features -- -D warnings - clean.
- cargo fmt --all --check - clean.
- ah check --run-tests - 180 passed, 0 findings.
- openspec validate --specs --strict - 9 passed, 0 failed.
- spk lint openspec/specs - 0 issues.

Review:
- RO5U verdict NEEDS_REVISION (CRITICAL 0; 2 HIGH, 1 MEDIUM, 1 LOW), then all HIGH and MEDIUM fixed and re-verified green. Findings in reviews/2026-11-23-ro5u-review-kaya-7pj-test-report-input.md. Remaining: LOW-001 (the summary line names the record count and the output path but not the input report) - recorded, not changed; EDGE-001 (<flakyFailure>/<rerunFailure> are attempt history and must not be read as a failure) - handled by design and documented in outcome().

Risks:
- The converter trusts nextest's JUnit shape. A JUnit dialect that names a test's outcome in an element the converter does not know would report that test as passed. The mapping covers the elements nextest emits.
- The report's counts come from the outcome elements, so a test that a JUnit producer reports in a non-standard way is counted as passed rather than refused. The adapter is unchanged by design, so the same convention applies to every libtest JSONL.
- The pretender gate now reports pre-existing violations in tests/verification.rs: the file was already over the 400-line limit before this ticket, but the xjp commit pulled it into the gate's diff scope, so 'pretender check --diff-only --mode gate' exits 1. It is not caused by this ticket's code and the fix is either pushing the branch (which empties the scope) or splitting the file.

Next:
- Ship-close: commit the script, its tests, and the decision record; close kaya-7pj with bd; export the beads JSONL; create the session handoff. Do NOT push - fleet policy requires explicit user authorization for push, deploy, or release.
