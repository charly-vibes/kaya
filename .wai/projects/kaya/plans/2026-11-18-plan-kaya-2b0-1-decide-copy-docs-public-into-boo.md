---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-2b0-publication-surface, pipeline-step:plan]
---

PLAN kaya-2b0
1. Decide: copy docs/public into book/public as part of `just docs` and the Docs workflow (single Pages artifact), and rewrite the two src links to `public/overview.md`. Record decision + rationale in docs/src/document-taxonomy.md (Pages has one site; a separate upload would replace the book).
2. Create an implementation ticket with metadata.files [justfile, .github/workflows/docs.yml, docs/src/index.md, docs/src/document-taxonomy.md], base_commit HEAD, and a Meter: `just docs` produces book/public/overview.html; `python3 -c` confirms the copied page; the changed links resolve under book/.
3. Green: write the decision and create the ticket; no workflow changes in this ticket.
Commit: docs(publications): decide the publication deployment surface. Closes kaya-2b0.

