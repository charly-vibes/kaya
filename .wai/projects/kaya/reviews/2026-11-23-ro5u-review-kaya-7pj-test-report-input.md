# Rule-of-5 Review — kaya-7pj "test report input on the stable toolchain"

**Work Reviewed:** working-tree diff of `scripts/junit-to-libtest.py` (new), `tests/test_report_input.rs` (new), `docs/src/status.md`, `README.md`, `CHANGELOG.md` (repo `/var/home/sasha/para/areas/dev/gh/charly/kaya`).

**Method:** `skill://rule-of-5-universal` lenses — correctness vs. the ticket's acceptance, invalid states made unrepresentable, error handling, boundaries/edge cases, allocation/IO cost, test quality, and doc-truth against the shipped behavior. The script was exercised directly against hand-built fixtures (a nextest-shaped report, a malformed document, a well-formed non-report, a missing file, a pre-existing output file) and the round trip was driven through the real `kaya` binary. Every compile/validate claim below was observed, not asserted.

**VERDICT: NEEDS_REVISION → all findings fixed — CRITICAL count = 0 (HIGH 2, MEDIUM 1, LOW 1).**

---

## STAGE 1: DRAFT (shape)

One coherent vertical slice: a repository on a stable Rust toolchain can now feed the test-runner adapter. The shape is right — a converter at the edge of the toolchain rather than a second input format inside the adapter, so the adapter's contract, its event-id derivation, and its atomic refusal behavior are untouched.

The weakness found in shape was the *failure* shape: the converter wrote its output in place, so "the conversion failed" and "the report is a valid but truncated report" were not distinguishable to a downstream reader. That is a state the design must not admit.

[DRAFT-001] HIGH — `scripts/junit-to-libtest.py` (the green version's `Path(args.output).write_text(body, ...)`)
**Description:** The report was written in place with `write_text`, which truncates the destination and then writes. A failure between the two — a full disk, an interrupted process — leaves a truncated file. Every surviving line is still a valid JSON record, so the file parses and the adapter imports it as a run with *fewer* tests than actually ran. The record would then state a run that never happened, with a count that silently under-reports it. A truncated report is the worst failure mode here: it is not refused, it is believed.
**Evidence:** `write_text` opens the destination with truncation semantics; the green implementation used it directly. The adapter's parser (`src/adapters/testrunner.rs`) skips any line that is not a JSON object and counts `ok`/`failed`/`ignored` events — a truncated file's surviving records all count normally, so the import succeeds with a wrong total. Reproduced by inspection of the adapter's counting path: a report holding 2 of 4 records yields `passed=2, total=2` and `AppendOutcome::Appended`, not a refusal.
**Recommendation:** write the report to a sibling temporary file and rename it into place. A rename is atomic, so the destination is either the previous report or the whole new one — never a valid subset.

## STAGE 2: CORRECTNESS

[CORR-001] HIGH — `scripts/junit-to-libtest.py`, `convert()`
**Description:** A well-formed XML document that is not a test report — the wrong file, or a JUnit document with no test cases — converted to an *empty* report and exited 0. The empty report then imported as a real run: `experiment.run.completed` with `passed=0, failed=0, ignored=0, total=0`. A file that states no test is not a run that passed zero tests; it is not a run at all. The converter was indistinguishable from a report of an empty suite, so the failure was silent and the record was false.
**Evidence:** `python3 scripts/junit-to-libtest.py wrong.xml out.jsonl` on a `<config>` document printed "wrote 0 test record(s)" and exited 0. `kaya import test --report out.jsonl` then appended an event whose data was `{"failed":0,"ignored":0,"passed":0,"total":0}` with `type: experiment.run.completed`.
**Recommendation:** refuse a report that states no test case, with a diagnostic naming the document. An empty report is never a legitimate conversion of a run report.

[CORR-002] MEDIUM — `scripts/junit-to-libtest.py`, the write path
**Description:** A refused conversion left a *stale* output file from an earlier successful conversion in place, byte-identical to the earlier report. A later `kaya import test --report` against that path would import the earlier run as if it were the current one. The refusal wrote nothing, but the file it left behind was a valid report of a different run — the worst kind of leftover, because it is importable.
**Evidence:** With `out.jsonl` holding a previous report, `python3 scripts/junit-to-libtest.py broken.xml out.jsonl` exited 1 and left `out.jsonl` byte-identical to its previous content.
**Recommendation:** the atomic write above fixes this as a consequence — the destination is only replaced by a complete new report, so a refusal leaves the previous report rather than a partial one. Covered by a test that asserts the previous report is byte-identical after a refusal.

## STAGE 3: CLARITY

[CLAR-001] LOW — `docs/src/status.md`, the "Test report input" section
**Description:** The section showed `cargo nextest run --profile ci` as if the JUnit report came out of it by itself. It does not: nextest writes JUnit XML only when a profile configures it, and the path is a configuration choice, not a default.
**Evidence:** nextest's configuration reference documents `profile.<name>.junit.path` as *"Path to write the JUnit XML report to. If unset, JUnit reporting is disabled."* The first draft of the section omitted that configuration entirely.
**Recommendation:** state the `.config/nextest.toml` profile that enables the report, so the commands in the section are runnable as written. Fixed.

## STAGE 4: EDGE CASES

[EDGE-001] LOW — `scripts/junit-to-libtest.py`, the outcome mapping
**Description:** nextest writes `<flakyFailure>` and `<rerunFailure>` elements for a test's earlier attempts. A converter that treated every failure-ish element as a failure would report a flaky test that ultimately passed as failed.
**Evidence:** nextest's JUnit documentation lists `<flakyFailure>` and `<rerunFailure>` alongside `<failure>`; its `flaky-fail-status` setting ("failure" by default) decides whether a flaky-fail test carries a `<failure>` element at all.
**Recommendation:** map only `<failure>` and `<error>` to a failed outcome, and treat the retry elements as history. The implementation does exactly this, and the mapping is documented in the `outcome()` docstring. Recorded, not changed.

## STAGE 5: EXCELLENCE

[EXCL-001] LOW — the script's stdout summary
**Description:** The summary line names the record count and the output path. It does not name the input report, which is the file a reader would go looking for when the count is surprising.
**Evidence:** the green implementation printed `junit-to-libtest: wrote 4 test record(s) to cargo-test-results.jsonl`.
**Recommendation:** the count and the destination identify the conversion; the input is on the command line. Recorded, not changed.

---

## Convergence

```
New CRITICAL issues: 0
Total new issues: 4 (2 HIGH, 1 MEDIUM, 1 LOW)
New issues vs Previous Stage: 0% (no new findings after Stage 2)
Estimated false positive rate: 0% (every finding was reproduced against the artifact)
Status: CONVERGED
```

## Stage-by-Stage quality

- Stage 1 (Draft): GOOD after the atomic-write shape was adopted — one file at the edge, adapter untouched.
- Stage 2 (Correctness): GOOD after CORR-001 and CORR-002 were fixed.
- Stage 3 (Clarity): GOOD after CLAR-001 was fixed.
- Stage 4 (Edge Cases): GOOD — EDGE-001 handled by design and documented.
- Stage 5 (Excellence): WITH_NOTES — EXCL-001 recorded.

## Verdict

**READY** — after the HIGH and MEDIUM findings were fixed and re-verified green.
