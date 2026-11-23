---
tags: [pipeline-run:tdd-ro5u-2026-11-23-kaya-7pj-test-report-input-on-the-stable-toolchain, pipeline-step:green]
---

GREEN kaya-7pj: the minimum production change is one new script and the decision it records.

1. scripts/junit-to-libtest.py - reads one JUnit XML report and writes libtest JSONL, file to file, like the other scripts in that directory (update-homebrew.py takes its paths as arguments). It uses only the Python standard library (xml.etree.ElementTree), so the repository gains no dependency.

The mapping: one record per <testcase>, in report order, each a JSON object {type: test, event: <outcome>, name: <qualified name>}. The event comes from the outcome element the case carries - <failure> or <error> is 'failed', <skipped> is 'ignored', no outcome element is 'ok'. Only those three elements decide the outcome: nextest also writes <flakyFailure> and <rerunFailure> for a test's earlier attempts, and those record history rather than the result the report states.

The name is the case's class name and name joined with '::' - nextest's classname is the binary the test ran in, and libtest qualifies its own names the same way, so two binaries running a test of the same name stay distinguishable.

The script refuses a report it cannot parse or read: it prints a diagnostic naming the report and exits 1, and it writes the output file only after every record is in hand, so a refused conversion leaves no report at all.

2. docs/src/status.md - a 'Test report input' section recording the choice, the exact nextest configuration and commands, and why the nightly alternative was rejected.

3. README.md and CHANGELOG.md - the import-test line points at the decision, and the changelog records the new stable-toolchain path.

No adapter change: src/adapters/testrunner.rs keeps its contract, its event-id derivation (tool, version, project, the report's mtime, the report's path, and the counts), and its atomic refusal behavior. The converted file is exactly the libtest JSONL the adapter already consumed.

The round trip holds because the adapter's event id digests the report's mtime, path, and counts: importing the same file twice yields the same id and the same bytes, and the ledger answers AppendOutcome::Duplicate - the second import appends nothing and reports one duplicate.

Verification after the change: cargo test --test test_report_input 3 passed 0 failed; cargo test 374 tests passed across 32 suites; cargo clippy --all-targets --all-features -- -D warnings clean; cargo fmt --all --check clean.
