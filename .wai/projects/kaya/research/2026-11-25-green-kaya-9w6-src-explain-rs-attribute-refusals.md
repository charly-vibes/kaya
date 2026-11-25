---
tags: [pipeline-run:tdd-ro5u-2026-11-25-kaya-9w6-explain-refuses-wrong-typed-attributes, pipeline-step:green]
---

GREEN kaya-9w6: src/explain.rs attribute_refusals(graph, block) mirrors build refusal surfaces exactly (Source.spec/entry/path/commit, Run.command/commands incl. non-array, Artifact.path under kind=='handoffs' via match guard); kind view status 'refused' + refusals [{node,attribute}]; product view refused precedence over emitted with reason refused-attribute; census push_refused adds omitted row reason refused-attribute + sorted dedup nodes; REASON_REFUSED_ATTRIBUTE const exported from publication::vocabulary. Full cargo test: 503 passed 0 failed; cargo fmt clean; clippy -D warnings clean.
