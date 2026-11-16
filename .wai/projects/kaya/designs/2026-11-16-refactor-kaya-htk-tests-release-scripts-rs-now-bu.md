---
tags: [pipeline-run:tdd-ro5u-2026-11-16-kaya-htk-release-workflow, pipeline-step:refactor]
---

REFACTOR kaya-htk: tests/release_scripts.rs now builds scratch dirs with the existing dev-dependency tempfile::TempDir instead of a hand-rolled pid+nanos path, so fixtures are removed on drop and the helper cannot collide. No production-code change; behavior identical (6/6 pass, clippy clean, fmt clean). Workflow/scripts left as-is: the duplicated retry block mirrors the ddl reference deliberately.
