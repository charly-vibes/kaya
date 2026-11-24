---
tags: [pipeline-run:tdd-ro5u-2026-11-24-kaya-7es-doctest-execution-contracts, pipeline-step:red]
---

RED kaya-7es: command='cargo test doctest' result=50 compile errors, all E0425/E0433/E0422 naming the planned API only: extract/run/Outcome/Runner/Failure/contract_hash/supersede_event/capture_with_kind in new src/doctest.rs tests, RecordKind::Doctest + crate::doctest::contract_hash in src/projection/ir.rs tests, Actor/ActorType imports. No setup breakage: package kaya-vibes compiles clean without cfg(test); bindings declared (.espectacular/changes/add-document-ir-commit-gate/kaya-documents/*.toml x4). Expected failures = missing doctest module surface + IR variant.
