---
tags: [pipeline-run:tdd-ro5u-2026-11-25-kaya-9w6-explain-refuses-wrong-typed-attributes, pipeline-step:fix-review]
---

FIX kaya-9w6: F1 fixed - commit refusal now gated behind non_blank(node,'path') with else-if mirroring the build's continue short-circuit (reuse.rs:277-285); regression test tests/explain.rs::commit_refusal_requires_non_blank_path_like_build proves Source{commit:42} without path keeps status unpopulated with no refusals. F3/F4/F5 comment truths fixed (push_refused sorted-dedup wording, REASON_REFUSED_ATTRIBUTE general wording, family-refusal scope note in attribute_refusals doc). F2 deferred by ticket scope. Gates: cargo fmt clean, cargo test 504 passed 0 failed, clippy -D warnings clean.
