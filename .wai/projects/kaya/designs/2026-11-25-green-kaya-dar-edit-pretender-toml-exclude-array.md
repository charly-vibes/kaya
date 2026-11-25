---
tags: [pipeline-run:tdd-ro5u-2026-11-25-kaya-dar-pretender-target-exclude-glob-fix, pipeline-step:green]
---

GREEN kaya-dar: edit pretender.toml exclude array - add './target/**' after 'target/**'. Rationale documented in orient; no other keys change. Then full 'pretender check': expect zero ./target findings in output, delta resolution flips (fixed>=55, still_open<=5, new 0), remaining exit-1 driven only by the 5 documented live yellow units (g41 steady state). Then gates: cargo test -q 504/0, just record-verify ok, pretender check --diff-only exit 0.
