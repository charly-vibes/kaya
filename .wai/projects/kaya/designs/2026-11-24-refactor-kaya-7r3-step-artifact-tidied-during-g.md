---
tags: [pipeline-run:tdd-ro5u-2026-11-24-kaya-7r3-incremental-regeneration, pipeline-step:refactor]
---

REFACTOR kaya-7r3 (step artifact): tidied during green — skip logic centralized in BuildState::skippable + cmd_build::restore_skipped/build_single; 12 render-constructor bumps are uniform one-liners; state serialization owned wholly by projection::state; no cross-family duplication. Green suite re-verified post-tidy: full cargo test 449/0.
