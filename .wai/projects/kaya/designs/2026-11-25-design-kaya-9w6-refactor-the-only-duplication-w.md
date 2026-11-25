---
tags: [pipeline-run:tdd-ro5u-2026-11-25-kaya-9w6-explain-refuses-wrong-typed-attributes, pipeline-step:refactor]
---

DESIGN kaya-9w6 (refactor): the only duplication worth removing was the kinds->refusals flat_map appearing three times; extracted as kind_refusals so census and product views share one collection seam with attribute_refusals. No data-shape or behavior change; tests stayed green throughout (503 passed, 0 failed).
