---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-2b0-publication-surface, pipeline-step:orient]
---

ORIENT kaya-2b0: publication deployment surface (P2, s, design). Files: book.toml, docs/src/SUMMARY.md, .github/workflows/docs.yml.
Current: docs/public (11 pages + publication.json) is committed but no workflow uploads it; docs/src/index.md and document-taxonomy.md link `../public/overview.md`, which mdBook renders as book/../public/... — outside the uploaded book/ artifact.
Decision required + implementation ticket with files, base_commit, Meter. Recommendation in ticket: deploy docs/public inside the book artifact because GitHub Pages serves one site per repository; a second Pages job would overwrite the book.
Meter: Pages artifact contains book/public/overview.html; the built book's public links resolve under book/.

