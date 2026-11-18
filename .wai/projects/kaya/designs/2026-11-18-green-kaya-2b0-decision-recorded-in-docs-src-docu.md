---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-2b0-publication-surface, pipeline-step:red]
---

GREEN kaya-2b0: decision recorded in docs/src/document-taxonomy.md — the publication tree ships inside the book artifact (GitHub Pages serves one site per repository; a second upload would replace the book), with `just docs` copying docs/public to book/public and docs.yml uploading book/. Created implementation ticket kaya-96w (files justfile, .github/workflows/docs.yml, docs/src/index.md, docs/src/document-taxonomy.md; base 3095266; Meter: book/public/overview.html exists and rewritten links resolve under book/). Added kaya-96w as a blocker of kaya-nwv.

