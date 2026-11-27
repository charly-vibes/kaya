# RO5U Review — kaya-bov (genesis::feedback subcommand)

Diff: src/commands/feedback.rs (new), src/commands/mod.rs, src/main.rs, tests/cli.rs.
Pre-existing dirty docs/src/specs/* excluded (not this ticket).

## Convergence
CONVERGED at stage 2 (0 new criticals; no further findings in stages 3-5). Clippy --all-targets clean.

## Findings
- [EXCL-001] LOW — src/commands/feedback.rs:27: conditional with_title via
  `let mut args` + `if let Some` could be a chained builder expression.
  Equivalent readability; not flagged by clippy. Deferred.

## Verified invariants
- Additive only: ImportSource untouched, dispatch/envelope paths untouched,
  existing suites green (full cargo test).
- Error path: genesis Err(msg) → Failure("kaya.feedback", msg) → dispatch's
  existing Envelope::error("error", label, message) mapping (verified by
  feedback_unknown_kind_is_a_labeled_error and
  feedback_from_last_error_without_record_errors).
- Dry-run: body preview on stderr, single JSON envelope on stdout (test
  parses stdout as the only JSON document).
- `kaya feedback` with missing kind falls through to the pre-existing
  generic usage envelope, same as every other subcommand.
- No network in tests: happy path uses --dry-run (genesis prints preview and
  returns FallbackUrl without invoking gh).

## Verdict: READY
