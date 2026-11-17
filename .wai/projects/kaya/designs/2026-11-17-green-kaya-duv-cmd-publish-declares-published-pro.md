---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-duv-developer-products, pipeline-step:green]
---

GREEN kaya-duv: cmd_publish declares PUBLISHED_PRODUCTS (overview, reference + the six developer products, each includes = publication::admission(name)) and PUBLISHED_PACKAGES (project-site over overview+reference, developer-guide over the six developer products). No publication-module change needed. Regenerated docs/public (getting-started, how-to, developer-guide pages; overview cross-references the shared fact) and corrected the status lines in docs/src/document-taxonomy.md and docs/src/status.md. Verified: 251 tests, fmt/clippy clean, second publish recomputed [] with pages [developer-guide, getting-started, how-to, overview, project-site, reference] and omitted [concept, example, faq, troubleshooting], ah check --run-tests 12 passed.
