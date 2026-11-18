---
reviews: 2026-11-18-ro5u-kaya-a4e-0-critical-0-high-2-medium-2-low.md
verdict: pass
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-a4e-kaya-check-index, pipeline-step:ro5u-review]
---

RO5U kaya-a4e: 0 Critical, 0 High, 2 Medium, 2 Low. Verdict correct with one fail-open to fix.
MEDIUM 1: staged_history returns empty baseline on any `git show HEAD:` failure; a blobless clone or nested root would accept an unverified staged ledger. Fix: ask `git ls-tree HEAD -- .kaya/ledger.jsonl` whether HEAD records it; only empty/absent tree means first commit, a failed show after a present entry is check_io; use HEAD:./ path syntax.
MEDIUM 2: no test proves the ledger is read from the index. Add a test where the worktree ledger is corrupted and the staged ledger equals HEAD; worktree check exits 2, --index exits 0.
LOW 3: collapse presence+unmerged into one `ls-files --stage -z` call (stage != 0 means unmerged).
LOW 4: add an index stray managed document test to exercise list_index_markdown.

