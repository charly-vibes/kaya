---
tags: [pipeline-run:tdd-ro5u-2026-11-23-kaya-8f4-publication-block-vocabulary, pipeline-step:refactor]
---

REFACTOR kaya-8f4: the tidy pass reviewed the green diff and found nothing to change - the validator is already the smallest correct shape (one vocabulary predicate, one scan-based refusal, two call sites at the plan and removal paths), the tests reuse the file's existing fixtures and helpers rather than adding parallel ones, and the docs edits are single-purpose. No duplication to remove, no names to simplify, no behavior to adjust.

Verification unchanged and re-confirmed: cargo test --test publications 30 passed 0 failed; cargo fmt --all --check clean; cargo clippy --all-targets --all-features -- -D warnings clean.
