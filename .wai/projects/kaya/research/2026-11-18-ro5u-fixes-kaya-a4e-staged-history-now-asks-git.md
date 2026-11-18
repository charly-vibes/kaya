---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-a4e-kaya-check-index, pipeline-step:fix-review]
---

RO5U-FIXES kaya-a4e: staged_history now asks `git ls-tree HEAD -- .kaya/ledger.jsonl` whether HEAD records the ledger; absence (or unborn HEAD) is the only empty baseline, a failed `git show HEAD:./...` after a present entry is kaya.verification.check_io, so blobless/nested-root reads fail closed. read_index uses one `ls-files --stage -z` call for presence and unmerged (stage != 0). Added check_index_reads_the_ledger_from_the_index (worktree ledger refuses, staged ledger clean) and check_index_reports_stray_managed_document. Verified: verification suite 18/18; full cargo test green; clippy -D warnings clean.

