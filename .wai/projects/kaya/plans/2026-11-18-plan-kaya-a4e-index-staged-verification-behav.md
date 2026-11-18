---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-a4e-kaya-check-index, pipeline-step:plan]
---

PLAN kaya-a4e — --index staged verification

Behavior: staged_content (partial staging honored, worktree noise ignored, first commit verifiable),
index_read_failures, property index_reads. Out of scope: facts, assertions, stale, hooks.

Design:
1. src/verification/mod.rs: turn Source into an enum { Worktree, Index } with the same read_text,
   read_bytes, list_markdown, relative. Worktree reads fs; Index resolves root-relative paths with
   `git -C <root> show :<rel>` and lists with `git -C <root> ls-files -- <rel-dir>` filtered to
   direct Markdown children.
2. Index absence vs failure: `git ls-files --stage -- <rel>` empty => absent (None). A non-empty
   unmerged entry (`git ls-files -u -- <rel>` non-empty) or a failing `git show :<rel>` => exit 2
   `kaya.verification.check_io` naming the path; never fall back to the worktree.
3. Ledger: in index mode read `:.kaya/ledger.jsonl`; committed bytes from `HEAD:.kaya/ledger.jsonl`.
   When HEAD carries no ledger copy, accept the staged ledger with empty history (first commit).
   Otherwise the same prefix + newline-terminated-line rule as worktree. --expect-digest still
   pins the verified bytes; no --allow-uncommitted.
4. src/main.rs: `--index` flag on Check, passed to verification::check. Envelope/exit unchanged.

Tests first (tests/verification.rs):
- index_only_drift_fails: stage a drifted index.md, keep worktree clean => --index exits 1 with
  changed block; a worktree-only drift with clean index passes (two tests or one with both halves).
- first_commit_is_verifiable: no ledger in HEAD, staged ledger => --index verifies instead of refusing.
- unreadable_index_path_fails_loudly: unmerged index entry for index.md (git update-index
  --index-info with two stages) => exit 2, kaya.verification.check_io naming docs/research/index.md,
  no drift, worktree copy ignored.
TOMLs: partial-staging-is-honored, worktree-noise-is-ignored, first-commit-is-verifiable,
unreadable-index-path-fails-loudly, index-reads.

Commands: cargo test --test verification; cargo test; clippy; fmt; ah check --changes
add-record-verification --run-tests; just ci.
Commit: feat(check): verify staged record with --index. Closes kaya-a4e.

