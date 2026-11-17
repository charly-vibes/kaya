---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-1b9-project-overview, pipeline-step:fix-review]
---

RO5U-FIXES kaya-1b9: build_overview derives its block name from Kind::ProjectOverview; one shared entity_rows renderer drives the overview groups and the understanding findings/decision groups; one shared read/commit policy (read_project_document + commit_project_document) now applies replace/append/unchanged, canonical kind order, omission, obsolete-block removal, write-on-change and whitespace-only deletion for both project-level builders, so they cannot diverge. Tests: partial-overview exact block (no empty sections), authored prose surviving an omitted build with a stable third build, and a vocabulary kind read from a non-canonical file with the backed product emitted. Docs: status.md taxonomy row and build-output lists, README build outputs, taxonomy section-order wording. Verified: all suites green, fmt/clippy clean, build/publish idempotent, ah check --run-tests 12 passed.
