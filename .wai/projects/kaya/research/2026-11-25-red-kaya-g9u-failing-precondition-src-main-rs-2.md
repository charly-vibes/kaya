---
tags: [pipeline-run:tdd-ro5u-2026-11-25-kaya-g9u-split-main-rs-into-command-modules-waves-tighten-pretender-ceiling, pipeline-step:orient]
---

RED kaya-g9u: failing precondition = src/main.rs 2456 lines against a deliberate file_lines_max 3500 (pretender.toml header defers to this ticket). Any tightening attempt today refuses (kaya-g41 measured-max discipline: cap must sit at measured max + slim headroom; with main.rs 2456 the cap floor is ~2600, no tightening possible). Extraction waves are the red->green move; per-wave oracle cargo test green.
