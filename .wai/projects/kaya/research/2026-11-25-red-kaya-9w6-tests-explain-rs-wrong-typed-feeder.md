---
tags: [pipeline-run:tdd-ro5u-2026-11-25-kaya-9w6-explain-refuses-wrong-typed-attributes, pipeline-step:red]
---

RED kaya-9w6: tests/explain.rs::wrong_typed_feeder_attribute_reads_refused fails at HEAD exactly as planned - kind view status 'unpopulated' != expected 'refused' (assertion at explain.rs:250), proving explain silently reports wrong-typed feeder attributes as unpopulated. Red observed via cargo test --test explain wrong_typed_feeder_attribute_reads_refused.
