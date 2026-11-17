---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-s5v-experiment-dossier-slice, pipeline-step:refactor]
---

REFACTOR kaya-s5v: no behavior change. (1) build() no longer clones the existing document for change detection: it reads once into the working buffer and tracks a changed flag on replace/remove, removing an O(n) copy plus whole-document compare. (2) anchors are built once per build and shared by render_specification/render_record instead of two identical BTreeSet constructions; linked() takes the shared set. (3) rustfmt normalization across ticket files, clippy -D warnings clean. cargo test: 16 targets ok, 164 tests, 0 failures.
