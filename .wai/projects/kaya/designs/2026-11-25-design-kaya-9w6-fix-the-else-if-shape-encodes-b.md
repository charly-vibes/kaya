---
tags: [pipeline-run:tdd-ro5u-2026-11-25-kaya-9w6-explain-refuses-wrong-typed-attributes, pipeline-step:fix-review]
---

DESIGN kaya-9w6 (fix): the else-if shape encodes build's control flow directly (path refusal -> else path-reachable commit check) rather than a reachability flag, keeping the mirror readable line-by-line against reuse.rs; regression test asserts both absence of refused status and absence of the refusals field, so any future mirror drift on this arm fails loudly.
