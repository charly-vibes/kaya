---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-249-understanding-kinds, pipeline-step:quality-ledger]
---

QUALITY LEDGER kaya-249

Changed:
- src/projection/report.rs: FINDINGS_BLOCK/DECISION_BLOCK and build_understanding (project-level document): findings block renders Finding/Observation/Claim rows only; decision block renders Decision rows and Evidence rows (Source path@commit, influences-linked Finding see-decision refs); emptiness decided by the kind's own records; canonical block order; omission/removal with file deletion when nothing remains; validation before writes; write on change.
- src/projection/mod.rs: shared attribute/key/title_or_key/source_reference helpers (source_reference returns None for incomplete Sources).
- src/projection/index.rs: uses the shared helpers (output unchanged).
- src/main.rs: builds the understanding document after the reports and flags stale authored citations.
- src/publication/mod.rs: understanding.md added to the compiled-document reader so the kinds can back concept/faq/troubleshooting.
- tests/understanding.rs (13), tests/cli_pipeline.rs, tests/publications.rs; contract block-content-limited-to-its-kind.
- docs/src/document-taxonomy.md and docs/src/status.md reconciled with the implemented kinds.

Verified:
- cargo test: every suite green (20 test targets, ~240 tests), 0 failures.
- cargo fmt --all --check clean; clippy -D warnings clean.
- cargo run -- build: recomputed [], unchanged [experiment-specification, orientation.research-map, reuse-handoff.reproduction-package], omitted [experiment-record, handoff, decision-record, findings-and-analysis]; no understanding.md for the committed ledger. publish unchanged, documents [orientation.research-map].
- ah check --changes add-document-taxonomy --run-tests: 11 passed, 22 structural (remaining tickets).

Review:
- rule-of-5-universal, five passes: 0 critical, 0 high, 5 medium, 4 low; all fixed. Sharpest: the new document was the only compiled document without stale-authored auditing, its block order depended on build history, incomplete Sources produced unresolvable evidence, removing the last blocks left a zero-byte file, and the publication reader never opened the new file.

Risks:
- Decision Evidence lists every Source node because the graph has no Decision->Source edge; attribution is ledger-global until a relationship exists (recorded for the cutover ticket).
- The taxonomy still lists Run as a findings source while the ticket deliberately excludes run rows; the docs now document the implemented rule.
- No finding/decision records exist in the committed ledger, so the understanding blocks are exercised by fixtures only.

Next:
- Ship: tick tasks.md 3.2, close kaya-249, export beads, commit, then start kaya-1b9 (project-overview).
