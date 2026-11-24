---
tags: [pipeline-run:tdd-ro5u-2026-11-24-kaya-7es-doctest-execution-contracts, pipeline-step:refactor]
---

REFACTOR kaya-7es: extracted line_bounds() helper replacing three inline line-walking computations in doctest::extract; closing-fence scan restructured to Option<(body_end, after_close)> removing the dead after_close initialization. No behavior change; cargo test --lib 24 green, clippy -D warnings clean. Tidy commit will be separate from the feature commit.
