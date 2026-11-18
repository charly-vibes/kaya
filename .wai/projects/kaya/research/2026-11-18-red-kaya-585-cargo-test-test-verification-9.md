---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-585-kaya-check, pipeline-step:red]
---

RED kaya-585: `cargo test --test verification` → 9/9 fail as expected (unknown subcommand `check`, exit 1 with usage label). Tests compile with `cargo check --test verification`. Nine spectular TOMLs added under .espectacular/changes/add-record-verification/kaya-verification/, each bound to its test fn; `ah check --changes add-record-verification` reports no finding for these nine (remaining no-toml are the other slices). No src/ changes.

