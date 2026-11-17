---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-1b9-project-overview, pipeline-step:quality-ledger]
---

QUALITY LEDGER kaya-1b9

Changed:
- src/projection/report.rs: OVERVIEW_BLOCK (test-facing alias) and build_overview; shared entity_rows renderer and shared project-document read/commit policy now used by build_understanding and build_overview (canonical kind order, omission, obsolete-block removal, write-on-change, whitespace deletion, Kind-derived block names).
- src/main.rs: overview build wired after understanding, merged, stale-authored flagged.
- src/publication/mod.rs: overview.md in the compiled-document read list.
- tests/overview.rs (7), tests/cli_pipeline.rs (overview-backed publish), tests/publications.rs; contract known-family-and-kind-admitted.
- docs/research/overview.md (Questions from the committed ledger) and docs/public/{overview.md,project-site.md,publication.json} regenerated; docs/src/status.md, docs/src/document-taxonomy.md, README updated.

Verified:
- cargo test: every suite green (21 test targets), 0 failures; fmt and clippy clean.
- cargo run -- build: recomputed [], unchanged [experiment-specification, orientation.project-overview, orientation.research-map, reuse-handoff.reproduction-package]; omitted [experiment-record, handoff, decision-record, findings-and-analysis].
- cargo run -- publish: omitted [], pages overview/project-site/reference; overview backed by orientation.project-overview; idempotent.
- ah check --changes add-document-taxonomy --run-tests: 12 passed, 21 structural (remaining prose scenarios and other tickets).

Review:
- rule-of-5-universal, five passes: 0 critical, 0 high, 2 medium, 4 low; all fixed. Notable: the two project-level builders had already diverged on whitespace-only commits and duplicated the section renderer; both now share one renderer and one commit policy.

Risks:
- overview currently renders Questions only because the committed ledger has no Project/Finding records; other groups are fixture-tested.
- Two remaining taxonomy kinds have no producer (exploration-brief, technical-reference, knowledge-brief); tracked by kaya-lh1/cc9.

Next:
- Ship: tick tasks.md 3.3, close kaya-1b9, export beads, commit, then run final gates (just ci, ah check, openspec validate) and report push authorization.
