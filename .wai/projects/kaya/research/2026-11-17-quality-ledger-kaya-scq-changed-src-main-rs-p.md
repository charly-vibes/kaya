---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-scq-status-release-architecture, pipeline-step:quality-ledger]
---

QUALITY LEDGER kaya-scq

Changed:
- src/main.rs: PUBLISHED_PRODUCTS gains status-compatibility, release-migration and architecture-design (includes read from publication::admission); doc comment names the eleven products and both packages.
- src/publication/mod.rs: the export id tokenizer keeps '@' so Source references retain their commit suffix.
- tests/publications_export.rs (new: row emission, per-product attribution, repeat byte identity), tests/cli_pipeline.rs (remaining-products publish, admission-based attribution, omitted-page file assertions); contract format-attribution.
- docs/public/{architecture-design,release-migration,status-compatibility}.md generated, reference.md rewritten as a cross-reference, publication.json regenerated; docs/src/document-taxonomy.md and docs/src/status.md updated.

Verified:
- cargo test: every suite green (23 test targets), 0 failures; fmt and clippy clean.
- cargo run -- publish: 9 pages, omitted [concept, example, faq, troubleshooting], second run recomputed []; 24/24 exported Source ids carry @commit.
- ah check --changes add-document-taxonomy --run-tests: 13 passed, 20 structural.

Review:
- rule-of-5-universal, five passes: 0 critical, 0 high, 2 medium, 3 low; all fixed. Sharpest: the export attributed Source-bearing facts to truncated ids, so markdown and export disagreed and neither resolved to a ledger node; and the bound attribution test could not see a product row losing a composed kind.

Risks:
- Fact-once ownership by name order means architecture-design owns the research map and reference is a cross-reference page; deliberate and documented.
- tutorial stays undeclared until exploration-brief/record have producers; collections and the two adapter-dependent kinds remain out of scope (kaya-lh1, cc9).

Next:
- Ship: tick tasks.md 3.8, close kaya-scq, export beads, commit, then run final gates (just ci) and report push authorization.
