---
reviews: 2026-11-25-ro5u-review-kaya-9w6-reviewer-subagent-rule-of-5.md
tags: [pipeline-run:tdd-ro5u-2026-11-25-kaya-9w6-explain-refuses-wrong-typed-attributes, pipeline-step:ro5u-review]
---

REVIEW kaya-9w6 ro5u: CRITICALS: 0. Verdict: incorrect pending one mirror divergence, then fixed. F1 MAJOR: commit refusal not gated behind non-blank path (build short-circuits, reuse.rs:277-285) - fixed with else-if non_blank(node,'path'). F2 minor scope gap: per-name collection view stays omitted while census reads refused-attribute (ticket scopes collection rendering out) - deferred follow-up. F3 minor: push_refused doc claimed graph order while sorting - comment fixed. F4 minor: REASON_REFUSED_ATTRIBUTE doc narrowed to products - comment fixed. F5 minor: attribute_refusals doc overclaimed family-wide refusal reflection - scope note added. Lenses: precedence honest (refused>emitted/omitted), census lossless, O(K*N) bounded, mirror duplication stated, test readable.
