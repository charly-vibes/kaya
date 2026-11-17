---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-duv-developer-products, pipeline-step:orient]
---

# ORIENT kaya-duv — developer-guide products

Ticket: kaya-duv, P2, lane:afk, complexity:l, files: src/publication/mod.rs.
Depends on kaya-qp8 (tier machinery, admission table, fact-once, justified
emission) and the document kinds from s5v/i03/3f6/249/1b9.

## Acceptance as testable statements

1. Each of getting-started, how-to, concept, example, troubleshooting, faq
   emits a page only from the kinds in its admission row
   (getting-started: project-overview + reproduction-package; how-to:
   technical-reference + reproduction-package; concept: decision-record +
   findings-and-analysis; example: experiment-record; troubleshooting:
   findings-and-analysis; faq: findings-and-analysis + decision-record).
2. Inadmissible kinds are refused with kaya.publications.inadmissible_refused
   naming product, kind and row (generic machinery already does this).
3. No page without a backing document; omitted products are reported and named
   unavailable in packages.
4. A fact shared by two products renders once; the other page cross-references
   it.
5. CLI publish declares all twelve products from their admission rows (so the
   developer products are exercised) and reports justified omissions.

## Current code seams

- src/publication/mod.rs: PRODUCTS/COLLECTIONS/PACKAGES, admission(),
  reader_question(), build(decls, docs, out), Decl{includes}, fact-once owners,
  package rows, export.
- src/main.rs cmd_publish: declares overview, reference and project-site only.
- Compiled documents available today: orientation.project-overview (Questions),
  orientation.research-map, exploration.experiment-specification, and
  (fixture-only) experiment-record, understanding kinds, reuse kinds.

## Validation

narrow: cargo test --test publications_developer --test publications --test cli_pipeline
full: cargo test; cargo run -- build; cargo run -- publish; ah check --changes add-document-taxonomy --run-tests; just ci
baseline: HEAD 24b4ffc, all suites green, publish emits overview/reference/project-site.

