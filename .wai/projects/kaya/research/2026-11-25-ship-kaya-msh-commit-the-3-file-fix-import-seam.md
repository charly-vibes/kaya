---
tags: [pipeline-run:tdd-ro5u-2026-11-25-kaya-msh-wai-adapter-idempotent-re-import, pipeline-step:ship-close]
---

SHIP kaya-msh: commit the 3-file fix (import-seam stable-identity dedup: same identity+same facts -> duplicate skip absorbing mtime drift; same identity+changed facts -> root-superseding generation-salted correction; no id-math/schema change, append-only preserved) + two red-to-green integration tests. Review READY CRITICALS=0. Post-commit: bd close kaya-msh + bd export -o, record-capture absorbs the commit.
