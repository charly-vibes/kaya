---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-duv-developer-products, pipeline-step:quality-ledger]
---

QUALITY LEDGER kaya-duv

Changed:
- src/main.rs: cmd_publish declares PUBLISHED_PRODUCTS (overview, reference + the six developer products, includes read from publication::admission) and PUBLISHED_PACKAGES (project-site, developer-guide).
- src/publication/mod.rs: block-aware remove_page (authored pages at omitted paths survive; block-only pages removed; mixed pages stripped) and slugged cross-reference anchors.
- tests/publications_developer.rs (5), tests/cli_pipeline.rs (publish cases incl. package body, two-kind composition, authored-page preservation), tests/publications.rs anchor updates.
- docs/public/{developer-guide,getting-started,how-to}.md generated; overview/how-to cross-reference; docs/src/document-taxonomy.md and docs/src/status.md status corrected.

Verified:
- cargo test: every suite green (22 test targets), 0 failures; fmt and clippy clean.
- cargo run -- publish: pages [developer-guide, getting-started, how-to, overview, project-site, reference], omitted [concept, example, faq, troubleshooting], second run recomputed [].
- ah check --changes add-document-taxonomy --run-tests: 12 passed, 21 structural.

Review:
- rule-of-5-universal, five passes: 0 critical, 0 high, 2 medium, 2 low; all fixed except the documented fact-once ownership design. Sharpest: publish could delete a hand-written page that happened to sit at an unbacked product's path (fixed), and cross-reference fragments did not match heading anchors (fixed).

Risks:
- Fact-once ownership follows name order, so getting-started owns the project-overview body and the overview page is a cross-reference; deliberate and documented.
- tutorial and kaya-scq's three products remain undeclared until that ticket.

Next:
- Ship: tick tasks.md 3.7, close kaya-duv, export beads, commit, then start kaya-scq (remaining products + format attribution).
