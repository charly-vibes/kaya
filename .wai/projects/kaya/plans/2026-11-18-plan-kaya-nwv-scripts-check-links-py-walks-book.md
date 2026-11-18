---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-nwv-link-checker, pipeline-step:plan]
---

PLAN kaya-nwv: scripts/check-links.py walks book/**/*.html, extracts href/src, skips schemes/#, resolves relative targets against the page, and fails naming each unresolved target. justfile `links: docs` builds then checks. Verify: current book passes; inject a broken link in a temp copy -> exit 1 naming it.

