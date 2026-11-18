---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-ntv-kaya-check-assertions-stale, pipeline-step:fix-review]
---

RO5U-FIXES kaya-ntv: Stale entries now carry the root-relative document path, sorted and deduped by (path, block, evidence). Added satisfied+repeated-flag coverage, the dotted-but-unknown `absent` kind path, and index parity: staged correction/citing document enforces under --index while worktree-only staleness is ignored. Verified: verification suite 32/32; full cargo test green; clippy -D warnings; fmt.

