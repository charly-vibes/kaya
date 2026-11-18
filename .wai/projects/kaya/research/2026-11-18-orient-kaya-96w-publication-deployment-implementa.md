---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-96w-publication-deployment, pipeline-step:orient]
---

ORIENT kaya-96w: publication deployment implementation from kaya-2b0. Files: justfile, .github/workflows/docs.yml, docs/src/index.md, docs/src/document-taxonomy.md. Base 3095266.
Acceptance: `just docs` produces book/public/overview.html containing the compiled overview; the rewritten links resolve under book/; Docs workflow keeps artifact path book.
Approach: in `docs` recipe after mdbook build, replace book/public with docs/public; mirror in docs.yml; rewrite the two `../public/overview.md` links to `public/overview.md`.

