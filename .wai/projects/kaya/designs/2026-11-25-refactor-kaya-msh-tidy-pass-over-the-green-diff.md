---
tags: [pipeline-run:tdd-ro5u-2026-11-25-kaya-msh-wai-adapter-idempotent-re-import, pipeline-step:refactor]
---

REFACTOR kaya-msh: tidy pass over the green diff - collapsed the nested capture-gate if/if-let into the repo's let-chain style (src/commands/import.rs:61-88) and hoisted the kind serialization out of format! args (src/adapters/mod.rs stable_identity) clearing two new-code clippy warnings. Gates after tidy: clippy 0 warnings, fmt clean, full suite 506 passed / 0 failed. No structural refactor applies: the seam is four small pure helpers + one gated loop block beside the existing correction machinery it mirrors.
