---
tags: [pipeline-run:tdd-ro5u-2026-11-27-kaya-bov-adopt-genesis-feedback-subcommand, pipeline-step:red]
---

RED kaya-bov: command='cargo test --test cli'; expected failure=the 3 new feedback tests fail because kaya has no feedback subcommand (clap emits a usage error envelope, exit 1); observed: 3 failed / 10 passed, failures are the missing subcommand, not setup breakage.
