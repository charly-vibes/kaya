---
tags: [pipeline-run:tdd-ro5u-2026-11-25-kaya-dar-pretender-target-exclude-glob-fix, pipeline-step:refactor]
---

REFACTOR kaya-dar: no refactor step applies - the change is a one-line exclude glob in pretender.toml; there is no code motion, no new module surface, and no threshold semantic to tidy. Post-green verification is the tidy evidence: full scan free of ./target findings, cargo test 504/0 unchanged, record-verify ok.
