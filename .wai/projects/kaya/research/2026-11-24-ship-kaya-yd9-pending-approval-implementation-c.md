---
tags: [pipeline-run:tdd-ro5u-2026-11-24-kaya-yd9-implement-add-collection-kinds, pipeline-step:ship-close]
---

SHIP kaya-yd9 (pending approval): implementation complete — five implemented collections (glossary, topic-map, knowledge-index, example-gallery, api-catalogue) with render-or-omit semantics, export omission reasons, tier-aware removals, read-list parity (technical-reference.md in both lists). Gates: full cargo suite green (red first), fmt+clippy clean, ah check 230 passed, openspec validate --strict valid, dogfood published (topic-map + knowledge-index live, gallery/catalogue omitted with reasons), kaya check ok on committed dogfood. RO5U READY, 0 critical, 2 medium fixed. Docs: CHANGELOG unreleased entries, tasks.md 1.x/3.x ticked, follow-ups already filed (kaya-34g, kaya-quh, kaya-w68). Commit plan: one atomic feat(publication) commit (src + tests + docs/public + CHANGELOG + openspec tasks + wai artifacts + .beads after bd close kaya-yd9); no push.
