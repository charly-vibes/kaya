---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-8ot-kaya-check-facts, pipeline-step:red]
---

RED kaya-8ot: `cargo test --test verification` → 18 old pass, 4 new fail because `data.events`/facts are absent. New: check_facts_reported, check_capture_frontier_survives_rewrite, check_sources_and_kinds_canonical, check_facts_property; four TOMLs bound (facts-without-a-second-parser, unreachable-capture-commit, sources-and-kinds-are-canonical, facts-reported). ah check --changes add-record-verification: those slugs no longer no-toml. No src changes.

