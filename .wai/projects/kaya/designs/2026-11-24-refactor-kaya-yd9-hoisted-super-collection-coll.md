---
tags: [pipeline-run:tdd-ro5u-2026-11-24-kaya-yd9-implement-add-collection-kinds, pipeline-step:refactor]
---

REFACTOR kaya-yd9: hoisted super::collection::collection_page to a top-level use in render.rs matching the module's import style; scanned the new renderers for duplication — none (section/first_id/heading_anchor each have one home). Tests stay green (38 cli + 42 publications), fmt clean, clippy -D warnings zero.
