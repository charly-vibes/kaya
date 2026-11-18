---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-ntv-kaya-check-assertions-stale, pipeline-step:red]
---

RED kaya-ntv: `cargo test --test verification` → 23 old pass, 6 new fail (flags rejected with kaya.verification.usage; stale stays []). New tests: check_required_kind_omitted, check_required_source_absent (incl. unknown source/malformed kind usage), check_stale_advisory_by_default, check_stale_enforced, check_required_items, check_stale_violation; six TOMLs bound. ah check --changes add-record-verification reports no issues. No src changes. (A red-agent shell accident committed and was reset to 879b4d2; docs/research/index.md verified byte-clean, only tests/verification.rs and the six TOMLs changed.)

