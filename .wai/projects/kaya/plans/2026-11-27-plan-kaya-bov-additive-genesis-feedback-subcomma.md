---
tags: [pipeline-run:tdd-ro5u-2026-11-27-kaya-bov-adopt-genesis-feedback-subcommand, pipeline-step:plan]
---

PLAN kaya-bov: additive genesis::feedback subcommand.

DESIGN
- src/commands/feedback.rs (new, mirrors commands/status.rs conventions): pub(crate) cmd_feedback(kind, title, dry_run, from_last_error) -> Result<Envelope, Failure>. Builds genesis::feedback::FeedbackArgs, calls handle_feedback(&args, "kaya", env!(CARGO_PKG_VERSION), "charly-vibes/kaya", cwd) (dulce precedent). Success: Envelope::ok("feedback", data) where GhResult maps Created{url,number}->{outcome:"created",url,number}, FallbackUrl->{outcome:"fallback",url}, LocalFile->{outcome:"local-file",path}. Err(msg) -> Failure(label "kaya.feedback", msg) -> Envelope::error("error", label, message) at the dispatch arm (existing convention).
- src/main.rs: Command::Feedback { kind, title, dry_run, from_last_error } variant (arg set mirrors dulce cli.rs:113-128) + dispatch arm.

OUT OF SCOPE
- scratch recording (write_scratch_best_effort on non-zero exits): dont-only extension that changes existing error paths; forbidden by this ticket's 'no existing behavior changes'. Follow-up candidate if the human wants --from-last-error functional.
- real gh filing tests (network).

RED FIRST (tests/cli.rs, 3 tests)
1. feedback_dry_run_prints_fallback_envelope: kaya feedback bug --dry-run -> exit 0, envelope_kind 'feedback', ok true, data.outcome=='fallback', data.url ends 'charly-vibes/kaya/issues/new'.
2. feedback_unknown_kind_is_a_labeled_error: kaya feedback frotz -> ok false, error envelope, label 'kaya.feedback', message contains 'unknown kind'.
3. feedback_from_last_error_without_record_errors: --from-last-error --dry-run -> error envelope, message contains 'No recent error recorded'.
Fails today: no feedback subcommand (clap usage error envelope, exit 1).

NARROW CHECK: cargo test --test cli
FULL VERIFY: cargo test; ah check --run-tests; openspec validate --all; pretender gate via lefthook on commit

FILES: src/commands/feedback.rs (new), src/main.rs, tests/cli.rs, src/commands/mod.rs (module decl), Cargo.lock (none — genesis already a dep)

COMMITS: one feature commit, no tidy expected.
