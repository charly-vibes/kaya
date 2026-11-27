---
tags: [pipeline-run:tdd-ro5u-2026-11-27-kaya-bov-adopt-genesis-feedback-subcommand, pipeline-step:quality-ledger]
---

QUALITY LEDGER kaya-bov: 

CHANGED:
- src/commands/feedback.rs (new) — cmd_feedback delegates to genesis::feedback::handle_feedback('kaya', 'charly-vibes/kaya'); maps GhResult onto data {outcome,url,number|path}; genesis errors carry label 'kaya.feedback'
- src/commands/mod.rs — declare feedback module
- src/main.rs — top-level Command::Feedback {kind,title,from_last_error,dry_run} + dispatch arm
- tests/cli.rs — 3 tests: dry-run fallback envelope (stdin body), unknown-kind labeled error, from-last-error without scratch record

VERIFIED:
- cargo test: all suites green (13/13 in tests/cli.rs incl. the 3 new)
- cargo clippy --all-targets: clean
- ah check --run-tests: OK 235 passed, 0 findings
- openspec validate --all: 13 passed, 0 failed

REVIEW:
- RO5U converged stage 2, 0 criticals; 1 LOW deferred (EXCL-001 builder chaining style). Verdict READY. Artifact: .wai/projects/kaya/reviews/ro5u-kaya-bov-findings.md

RISKS:
- --from-last-error reports genesis's 'No recent error recorded for kaya' until scratch recording is adopted (dont-style write_scratch_best_effort on non-zero exits) — deliberately out of scope: the ticket forbids existing-behavior changes; follow-up candidate
- real gh filing untested (network); genesis's gh module carries its own tests

NEXT:
- ship-close: human approval gate, atomic commit, bd close kaya-bov + export, then wai close handoff and authorized push
