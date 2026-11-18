---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-ntv-kaya-check-assertions-stale, pipeline-step:green]
---

GREEN kaya-ntv: new verification/assertions.rs (Requested validation, Required and Stale types, required() from facts, stale() from changed_event_ids + report::stale_authored); CheckOptions {index, require_kinds, require_sources, fail_on_stale_authored}; Outcome gains required/stale and enforcement-aware exit_code; main adds repeatable --require-kind/--require-source and --fail-on-stale-authored and emits required + computed stale. Verified: verification suite 29/29; full cargo test green; clippy -D warnings; fmt.

