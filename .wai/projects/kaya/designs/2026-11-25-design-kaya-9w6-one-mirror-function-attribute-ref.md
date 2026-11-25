---
tags: [pipeline-run:tdd-ro5u-2026-11-25-kaya-9w6-explain-refuses-wrong-typed-attributes, pipeline-step:green]
---

DESIGN kaya-9w6: one mirror function attribute_refusals(graph, block) -> Vec<(String, attr)> in src/explain.rs, arms keyed on (block, entity) matching reuse::inputs refusal surfaces verbatim so the two can never disagree without a failing contract test. Shape: kind view data.refusals [{node, attribute}]; product view status refused wins over emitted (build would fail, so nothing can emit); census rows land on the omitted side (no page is built) with reason refused-attribute and sorted-dedup nodes array. Wrong-typed Artifact.kind stays a skip mirroring build. New vocabulary const REASON_REFUSED_ATTRIBUTE re-exported beside REASON_NO_BACKING_DOCUMENT.
