---
tags: [pipeline-run:tdd-ro5u-2026-11-23-kaya-7pj-test-report-input-on-the-stable-toolchain, pipeline-step:fix-review]
---

RO5U-FIXES kaya-7pj (fix-review step): every High and Medium finding closed in the script and the documented commands.

HIGH-001 non-atomic report write - fixed: the report is written to a sibling temporary file and renamed into place (write_atomically). A rename is atomic, so the destination is either the previous report or the whole new one; a truncated file whose surviving records still parse - and which the adapter would import as a run with a smaller count - can no longer exist. A failed write removes the temporary file.

HIGH-002 a non-report document converted silently - fixed: convert() raises ValueError when the report states no test case, and main() refuses with a diagnostic naming the document, exit 1, no output file. A well-formed XML file that is not a test report, and a report of an empty suite, are both refused instead of becoming a zero-test run in the record.

MEDIUM-001 a refusal left a stale importable report - fixed as a consequence of the atomic write: the destination is replaced only by a complete report, so a refusal leaves the previous report byte-identical.

LOW clarity - fixed: docs/src/status.md now states the .config/nextest.toml profile that enables nextest's JUnit report, so the documented commands are runnable as written.

LOW excellence - recorded, not changed: the summary line names the record count and the output path; the input is on the command line.

New tests: a_document_with_no_test_cases_is_refused and a_refused_conversion_leaves_the_previous_report_intact, alongside the original three.

Re-verified green: cargo test --test test_report_input 5 passed 0 failed; cargo test 376 tests passed across 32 suites; cargo clippy --all-targets --all-features -- -D warnings clean; cargo fmt --all --check clean; ah check --run-tests 180 passed 0 findings; openspec validate --specs --strict 9 passed 0 failed; spk lint openspec/specs 0 issues.
