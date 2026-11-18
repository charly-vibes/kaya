---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-a4e-kaya-check-index, pipeline-step:orient]
---

ORIENT kaya-a4e: kaya check --index staged mode (P1, complexity m, lane afk). Files: src/main.rs, src/verification/index.rs. Depends kaya-585 (closed, commit ff8a2f6).

Spec: add-record-verification — Staged-content verification (3 scenarios) and Index read failures (1), plus property mirror index_reads.
Testable:
1. --index reads .kaya/ledger.jsonl and every compared document through `git show :<path>`; worktree bytes are never consulted for those paths.
2. A document drifted in the index only fails; drifted in the worktree only passes.
3. When HEAD has no ledger, the staged ledger is verified with empty committed history (first commit).
4. An unmerged/unreadable index entry fails exit 2 with kaya.verification.check_io naming the path; no worktree fallback.
5. Reported paths stay relative to --root; envelope/exit contract unchanged (0/1/2).

Meter: cargo test --test verification && ah check --changes add-record-verification --run-tests; index scenarios execute, 0 no-toml for them. Anti-goals: no worktree fallback, no writes.

