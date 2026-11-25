---
tags: [pipeline-run:tdd-ro5u-2026-11-25-kaya-9w6-explain-refuses-wrong-typed-attributes, pipeline-step:plan]
---

PLAN kaya-9w6 (step plan) — RED first: tests/explain.rs::wrong_typed_feeder_attribute_reads_refused; ledger seeded with Source{spec:42} (model.entity.created event); kaya explain technical-reference asserts status refused + refusals [{node: kaya/Source/spec-bad, attribute: spec}]; census asserts technical-reference omitted with reason refused-attribute and nodes naming the id; both fail at HEAD (status unpopulated / no-backing-document). GREEN: src/explain.rs attribute_refusals(graph, block) mirrors build refusal surfaces (Source.spec/entry/path/commit, Run.command/commands, Artifact.path under kind=='handoffs'; Artifact.kind wrong-type remains skip); kind view status refused; product view reason refused-attribute; census nodes array. Full gates: cargo test, just record-check, scripts/ah-check check. Files: src/explain.rs, tests/explain.rs. One feature commit.
