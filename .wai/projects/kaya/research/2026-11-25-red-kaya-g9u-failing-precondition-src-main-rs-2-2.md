---
tags: [pipeline-run:tdd-ro5u-2026-11-25-kaya-g9u-split-main-rs-into-command-modules-waves-tighten-pretender-ceiling, pipeline-step:red]
---

RED kaya-g9u: failing precondition = src/main.rs 2456 lines against deliberate file_lines_max 3500 (pretender.toml header defers to this ticket). Tightening today refuses: kaya-g41 discipline puts the cap at measured max + slim headroom, so with main.rs 2456 the floor is ~2600 - no tightening possible until extraction lands. Extraction waves are the red->green move; per-wave oracle cargo test green, 501+ tests.
