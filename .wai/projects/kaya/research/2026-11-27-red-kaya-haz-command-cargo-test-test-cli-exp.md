---
tags: [pipeline-run:tdd-ro5u-2026-11-27-kaya-haz-adopt-genesis-envelope-and-cli, pipeline-step:red]
---

RED kaya-haz: command='cargo test --test cli'; expected failure=version_json_prints_version_envelope panics in parse_envelope (stdout is clap's plain --version text, not a JSON envelope — genesis::cli::maybe_print_version_json not yet wired); observed: 1 failed / 9 passed, failure is the missing version-json behavior, not setup breakage.
