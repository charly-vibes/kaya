---
tags: [pipeline-run:tdd-ro5u-2026-11-23-kaya-7pj-test-report-input-on-the-stable-toolchain, pipeline-step:red]
---

RED kaya-7pj: command=cargo test --test test_report_input -> 0 passed, 3 failed.

Expected failure, missing behavior and not setup breakage: all three fail because scripts/junit-to-libtest.py does not exist yet - python3 exits 2 with "can't open file '.../scripts/junit-to-libtest.py': [Errno 2] No such file or directory". The fixture, the harness and the assertions are otherwise in place.

The three cases:
1. conversion_names_every_test_and_its_outcome - the converter must map four nextest-shaped <testcase> elements to four libtest 'test' records, named in report order, with the event named by the outcome element: no outcome element -> 'ok', <failure> -> 'failed', <error> -> 'failed', <skipped> -> 'ignored'.
2. converted_report_imports_once_and_dedupes_the_second_time - 'kaya init' then 'kaya import test --report' twice over the converted file: the first import appends one event (appended 1, duplicates 0) carrying the counts (passed 1, failed 2, ignored 1, total 4); the second import appends nothing and reports one duplicate (appended 0, duplicates 1), because the event id digests the report's mtime, path, and counts - the same file yields the same id and the same bytes, and the ledger answers AppendOutcome::Duplicate.
3. malformed_report_is_refused_and_writes_nothing - a malformed XML document must exit non-zero, name the report in its diagnostic, and leave no output file at all.

The nextest JUnit shape in the fixture is taken from nextest's own documented example (nexte.st/docs/machine-readable/junit): one <testsuite> per test binary, one <testcase> per test with a name and a classname, and the outcome in a child element (<failure>, <error>, <skipped>).
