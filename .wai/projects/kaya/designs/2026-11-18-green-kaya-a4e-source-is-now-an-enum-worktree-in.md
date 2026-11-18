---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-a4e-kaya-check-index, pipeline-step:green]
---

GREEN kaya-a4e: Source is now an enum (Worktree/Index); index reads use git ls-files --stage for absence, ls-files -u for unmerged check_io, and git show :<rel> for bytes, with literal pathspecs resolved from the check root; list_markdown uses ls-files -z; ledger reads staged bytes and accepts staged-only history when HEAD lacks a ledger copy; check() gains index mode; main --index. Verified: cargo test --test verification 16/16; full cargo test green; clippy -D warnings clean; fmt clean; ah check --changes add-record-verification --run-tests 50 passed, 10 no-toml (facts/assertions/stale slices); dogfood `kaya check --index` empty drift.

