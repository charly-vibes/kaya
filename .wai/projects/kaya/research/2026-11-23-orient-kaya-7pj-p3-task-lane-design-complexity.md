---
tags: [pipeline-run:tdd-ro5u-2026-11-23-kaya-7pj-test-report-input-on-the-stable-toolchain, pipeline-step:orient]
---

ORIENT kaya-7pj: P3 task, lane:design, complexity:s, files [scripts/junit-to-libtest.py, docs/src/status.md], base_commit fbec965. Acceptance as testable statements: (1) a decision is recorded in docs/src/status.md naming the exact commands; (2) the chosen path produces test events on one import and a second import of the same report dedupes (appended 0, duplicates 1); (3) the chosen path runs on a stable toolchain - no nightly.

Surface map (read, no edits yet):
- src/adapters/testrunner.rs - the adapter consumes libtest JSONL: one JSON value per line; a non-object line is skipped; a record whose type != 'test' is skipped; a type=='test' record without a string 'event' is an atomic refusal; 'event' must be one of started/ok/failed/ignored or it is an atomic refusal; ok/failed/ignored are counted into passed/failed/ignored. The event id digests tool, version, project, timestamp (the report file's mtime, second resolution), reference (the report path), and the serialized counts - so re-importing the SAME report file yields the same id and the same bytes, and the ledger answers AppendOutcome::Duplicate.
- src/ledger.rs:245-258 - append returns Duplicate for byte-identical lines and refuses an id reused with different bytes (kaya.ledger.id_reuse).
- src/main.rs:548-592 - cmd_import opens .kaya/ledger.jsonl (kaya init creates it), appends each event, and the envelope reports source, events, appended, duplicates, ledger.
- docs/src/status.md - the decision surface: a table (Area|Status|Notes) plus Implemented / In progress / Dogfooding / Notes sections; the 'kaya import wai|git|test|model' bullet (line 40) names 'one libtest JSONL run' as the input.
- README.md:33 - 'kaya import test --report <path>    # observe one libtest JSONL run'.
- scripts/ - every script is Python 3 with a shebang, a Purpose/Responsibilities/Rationale docstring, argparse, and explicit exit codes (0 ok, 1 failure, 2 usage). tests/release_scripts.rs drives a python script from a Rust integration test - the precedent for testing a script.

Upstream facts that decide the ticket's recommendation (primary sources):
- cargo test's --format json is unstable and nightly-only: it needs 'cargo +nightly test -- -Z unstable-options --format json'. rust-lang/rust#109044 'Prevent stable libtest from supporting -Zunstable-options' was merged, so the stable toolchain rejects that flag outright - the nightly path needs a nightly toolchain AND an unstable format.
- nextest's own '--message-format libtest-json' is experimental: it requires NEXTEST_EXPERIMENTAL_LIBTEST_JSON=1 and its format specification is 'TODO' upstream. Not a stable contract to depend on.
- nextest's JUnit XML output IS stable and documented: [profile.ci.junit] path = 'junit.xml', Jenkins XML flavour, one <testsuite> per test binary, one <testcase> per test, <failure>/<error>/<skipped> children, and <flakyFailure>/<rerunFailure> for retries.

Conclusion: the ticket's recommendation (a JUnit-XML-to-libtest-JSONL converter) is the only one of the two that serves a STABLE toolchain. The alternative - documenting the nightly invocation - is a nightly toolchain plus an unstable upstream format, which the ticket title rules out.

Not in scope: no adapter change (the adapter's contract and event-id derivation stay as they are), no model/ledger/projection change, no new event type, no new dependency (the converter uses only the Python standard library, xml.etree).
