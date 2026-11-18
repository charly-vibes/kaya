---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-a4e-kaya-check-index, pipeline-step:red]
---

RED kaya-a4e: `cargo test --test verification` → 11 old pass, 5 new fail: the binary answers `--index` with `kaya.verification.usage` exit 2 (unexpected argument). New: check_index_only_drift_fails, check_worktree_noise_ignored, check_first_commit_verifiable, check_unreadable_index_path_fails, check_index_reads; five TOMLs bound (partial-staging-is-honored, worktree-noise-is-ignored, first-commit-is-verifiable, unreadable-index-path-fails-loudly, index-reads). ah check --changes add-record-verification: those slugs no longer no-toml. No src changes.

