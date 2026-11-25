---
tags: [pipeline-run:tdd-ro5u-2026-11-24-kaya-dsv-author-missing-reader-documents, pipeline-step:refactor]
---

REFACTOR kaya-dsv: dropped the duplicate ALL_BLOCKS vocabulary table; all_blocks() now derives from KIND_CONTRACTS so the kind vocabulary has one in-tree spelling. clippy --all-targets --all-features: 0 warnings; cargo fmt --check clean; full cargo test: 499 passed 0 failed.
