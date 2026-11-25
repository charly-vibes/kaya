---
tags: [pipeline-run:tdd-ro5u-2026-11-25-kaya-ynj-research-surfaces-render-information-not-references, pipeline-step:ro5u-review]
---

FIX-REVIEW kaya-ynj: both review findings fixed by subagent, verified by orchestrator. A (P1): collection.rs render_topic_map strips trailing ' (label)' from from/to endpoint ids after split_once (node ids have no whitespace/parens); fixture extended in tests/publications/build.rs with a labeled row asserting bare-id topic rows + bare-id embedded edge. B (P2): source.rs MAP_KIND-gated strip_paren_groups (depth-counted, nesting-safe) before node_ids on the research-map block only; new census regression test proves label-referenced kaya/Decision/old-gate yields no phantom id. Orchestrator verified: FULL SUITE 510 passed 0 failed (509+1 new), clippy 0, fmt clean. Real-tree proof: kaya build + publish exit 0; docs/public/topic-map.md rows now bare-id (- kaya/Model/M-001 — Model — realizes), no label prose in any published id.
