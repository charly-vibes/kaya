---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-a4e-kaya-check-index, pipeline-step:refactor]
---

REFACTOR kaya-a4e: no further tidy. The git-index plumbing is one private read_index/list_index_markdown pair behind the Source enum, absence and failure are explicitly distinguished, and no second comparison path was introduced. fmt/clippy clean.

