---
tags: [pipeline-run:tdd-ro5u-2026-11-24-kaya-yd9-implement-add-collection-kinds, pipeline-step:fix-review]
---

RO5U-FIXES kaya-yd9: EDGE-002 fixed — IMPLEMENTED doc comment now names both sync directions (name without arm renders as omission; arm without name refused at partition). EDGE-001 fixed cheap — section() anchors heading finds to line starts, plus regression test gallery_sections_match_at_line_starts proving a mid-line heading mention cannot shadow a real section. CORR-001 resolved by live verification: real dogfood publish (clean rebuild) emits topic-map + knowledge-index, omits api-catalogue/example-gallery with no-backing-document. Full suite green, fmt + clippy -D warnings clean.
