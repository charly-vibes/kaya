---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-96w-publication-deployment, pipeline-step:plan]
---

PLAN kaya-96w
- justfile docs: specs-docs -> mdbook build -> rm -rf book/public; mkdir -p book/public; cp -R docs/public/. book/public/.
- docs.yml Build docs step mirrors the copy.
- docs/src/index.md and docs/src/document-taxonomy.md: `../public/overview.md` -> `public/overview.md`.
- Verify: just docs; test -f book/public/overview.html; grep the compiled overview marker in it; `python3` resolves both rewritten links as book/public/overview.html (or .md source mapping via mdbook).
Commit: chore(docs): deploy the publication tree inside the book. Closes kaya-96w.

