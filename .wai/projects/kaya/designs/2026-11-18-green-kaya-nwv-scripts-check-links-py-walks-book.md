---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-nwv-link-checker, pipeline-step:green]
---

GREEN kaya-nwv: scripts/check-links.py walks book/**/*.html and resolves relative href/src; just links runs it. Book sources fixed: specs links point to rendered specs/, active-change links are GitHub URLs, research is rendered into book/research, publication backlink points to ../index.html. Renderer generalized for public+research trees. Verified 28 pages, 0 unresolved; injected missing.html exits 1 naming it.

