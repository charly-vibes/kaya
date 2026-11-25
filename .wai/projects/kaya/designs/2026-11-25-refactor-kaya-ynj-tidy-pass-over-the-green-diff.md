---
tags: [pipeline-run:tdd-ro5u-2026-11-25-kaya-ynj-research-surfaces-render-information-not-references, pipeline-step:refactor]
---

REFACTOR kaya-ynj: tidy pass over the green diff - collapsed the two nested admission ifs in src/model.rs into let-chains per clippy (collapsible-if x2), ran cargo fmt across the tree (import-order drift in tests/research_map.rs). No structural duplication found: the seam is three small pure helpers + one gated loop block; factor-out candidates would mirror the seam exactly, so tidy applies. Re-verified: clippy 0 warnings, fmt clean, research_map 12/0, projection_index 12/0, acceptance 2/0.
