---
tags: [pipeline-run:tdd-ro5u-2026-11-23-kaya-7pj-test-report-input-on-the-stable-toolchain, pipeline-step:ro5u-review]
---

RO5U-FIXES kaya-7pj: every HIGH and MEDIUM finding closed; the LOW clarity finding fixed; the LOW excellence finding recorded, not changed.

HIGH-001 non-atomic report write - fixed: the report is now written to a sibling temporary file and renamed into place (write_atomically). A rename is atomic, so the destination is either the previous report or the whole new one; a truncated file whose surviving records still parse - and which the adapter would import as a run with a smaller count - can no longer exist. A failed write removes the temporary file.

HIGH/MEDIUM-002 a non-report document converted silently - fixed: convert() now raises ValueError when the report states no test case, and main() refuses with a diagnostic naming the document ('is not a test report: the report states no test case'), exit 1, no output file. A well-formed XML file that is not a test report, and a report of an empty suite, are both refused instead of becoming a zero-test run in the record.

MEDIUM-002 a refusal left a stale importable report - fixed as a consequence of the atomic write: the destination is replaced only by a complete report, so a refusal leaves the previous report byte-identical. Pinned by a test.

LOW clarity - fixed: docs/src/status.md now states the .config/nextest.toml profile ([profile.ci.junit] path = 'junit.xml') that enables nextest's JUnit report, so the documented commands are runnable as written - nextest writes JUnit XML only when a profile configures it.

LOW excellence - recorded, not changed: the summary line names the record count and the output path; the input is on the command line.

New tests: a_document_with_no_test_cases_is_refused and a_refused_conversion_leaves_the_previous_report_intact, alongside the original three.

Re-verified green after the fixes: cargo test --test test_report_input 5 passed 0 failed; cargo test 376 tests passed across 32 suites; cargo clippy --all-targets --all-features -- -D warnings clean; cargo fmt --all --check clean; ah check --run-tests 180 passed 0 findings; openspec validate --specs --strict 9 passed 0 failed; spk lint openspec/specs 0 issues.
