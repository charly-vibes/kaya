---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-1b9-project-overview, pipeline-step:green]
---

GREEN kaya-1b9: report.rs gains OVERVIEW_BLOCK (Kind-derived) and build_overview: one block with ### Project / ### Questions / ### Findings rows limited to Project, Question and Finding entities (no run, artifact, sha or capture rows); empty sections omitted; all-empty omits the kind, removes a stale block and deletes a whitespace-only document; validation before writes; write on change; sources keyed by the block name. main.rs merges the result and flags stale authored citations; publication::read_documents reads overview.md. Regenerated docs/research/overview.md (one Question row) and docs/public (overview page, project-site link, export emitted). docs/src/document-taxonomy.md status updated. Verified: 242 tests, fmt/clippy clean, build/publish idempotent, ah check --run-tests 12 passed.
