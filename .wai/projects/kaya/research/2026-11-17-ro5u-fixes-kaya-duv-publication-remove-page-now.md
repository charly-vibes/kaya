---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-duv-developer-products, pipeline-step:fix-review]
---

RO5U-FIXES kaya-duv: publication::remove_page now leaves a document untouched when it does not carry this page's managed block, removes block-only pages, and strips the block from mixed pages (authored files at omitted product paths survive); cross-reference fragments use heading anchors (alphanumerics/-/_ kept, spaces to hyphens, punctuation dropped) and the committed pages were regenerated; the CLI publish tests now pin the developer-guide package body exactly, seed a Source/commit so getting-started composes two admitted kinds and how-to emits with a cross-reference, and assert authored-page preservation plus block-only removal. Verified: 22 test targets green, fmt/clippy clean, publish idempotent, ah check --run-tests 12 passed.
