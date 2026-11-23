---
tags: [pipeline-run:tdd-ro5u-2026-11-23-kaya-7pj-test-report-input-on-the-stable-toolchain, pipeline-step:plan]
---

PLAN kaya-7pj: a stable-toolchain path to the test-runner adapter.

DECISION: ship scripts/junit-to-libtest.py - convert a nextest JUnit XML report into the libtest JSONL the adapter already consumes - and record the decision with its exact commands in docs/src/status.md.

Why this and not the nightly invocation: cargo test's --format json is unstable and nightly-only ('cargo +nightly test -Z unstable-options --format json'), and rust-lang/rust#109044 merged a change that makes stable libtest reject -Zunstable-options outright. nextest's own --message-format libtest-json is experimental (needs NEXTEST_EXPERIMENTAL_LIBTEST_JSON=1; its format spec upstream is 'TODO'). nextest's JUnit XML output is stable and documented. So the JUnit-to-libtest converter is the only one of the ticket's two options that serves a stable toolchain.

DESIRED BEHAVIOR:
- scripts/junit-to-libtest.py reads one JUnit XML file and writes libtest JSONL to stdout: one JSON object per line, a 'test' record per <testcase>, its 'event' mapped from the case's outcome element (no failure/error/skipped child -> 'ok'; <failure> -> 'failed'; <error> -> 'failed'; <skipped> -> 'ignored'), plus the suite's own records ignored. The name is the case name; counts the adapter tallies come from the event names, so the converter only has to name outcomes correctly.
- The converted file is consumable by 'kaya import test --report <path>': one import appends one run event with the run's counts, and importing the SAME file again dedupes (same mtime, same path, same counts -> same event id -> AppendOutcome::Duplicate).
- docs/src/status.md records the decision with the exact commands: 'cargo nextest run --profile ci' (which writes the JUnit report via [profile.ci.junit]) then 'python3 scripts/junit-to-libtest.py <junit.xml> > <report.jsonl>' then 'kaya import test --report <report.jsonl>'.

OUT OF SCOPE:
- No adapter change: src/adapters/testrunner.rs keeps its contract, its event-id derivation, and its atomic refusal behavior.
- No change to the model, ledger, projection, or the CLI.
- No new event type, no SOURCE_VERSION bump, no new dependency (xml.etree is in the Python standard library).
- Not a general JUnit->libtest converter for other CI systems: the mapping covers the outcomes nextest emits (pass, fail, error, skip, and the flaky/rerun children nextest adds).

TEST CASES TO WRITE FIRST (red):
1. A Rust integration test in tests/ that writes a nextest-shaped JUnit XML fixture, runs scripts/junit-to-libtest.py, and asserts the emitted JSONL lines: one 'test' record per case, 'ok' for a clean case, 'failed' for <failure> and <error>, 'ignored' for <skipped>. Expected failure today: the script does not exist.
2. The same test drives the converted file through 'kaya import test --report': the first import appends one event carrying the run's counts; the second import of the same file appends nothing and reports one duplicate. Expected failure today: no script, so no report to import.
3. A malformed-JUnit case: the script exits non-zero, prints a diagnostic naming the file, and writes no partial output.

NARROW TEST COMMAND: cargo test --test test_report_input
FULL VERIFICATION COMMAND: cargo test && cargo clippy --all-targets --all-features -- -D warnings && just ci && ah check --run-tests && openspec validate --specs --strict

FILES TOUCHED:
- scripts/junit-to-libtest.py (new; shebang, Purpose/Responsibilities/Rationale docstring per file-headers, argparse, explicit exit codes 0/1/2 like the other scripts).
- tests/test_report_input.rs (new; drives the script with python3 and the kaya binary, mirroring tests/release_scripts.rs for the script half).
- docs/src/status.md (the decision, its commands, and why the nightly route was rejected).
- README.md (the import-test line names the input format; it may need the stable path named).
- CHANGELOG.md (a stable-toolchain input path is user-visible).

COMMIT SHAPE: one feature commit (script + tests + decision record + docs). No separate tidy commit expected - the script is new, so there is nothing to tidy.

CONSTRAINTS: the script must write nothing on failure (no partial JSONL), the converted report must round-trip through the adapter's existing vocabulary without changing it, and the docs claim must be verifiable from the script's own --help.
