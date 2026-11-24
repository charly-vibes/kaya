---
tags: [pipeline-run:tdd-ro5u-2026-11-24-kaya-7r3-incremental-regeneration, pipeline-step:ro5u-review]
---

RO5U-FIXES kaya-7r3: review verdict NEEDS_REVISION (0C/1H/1M/1L, reviewer-verified empirically). CORR-001 HIGH fixed: documents initialize from saved.documents.clone() so an all-skipped run persists the gate it consumed (main.rs:844-851; BuildState fields pub'd at the bin boundary) — kills the measured 0→14 alternating loop. CORR-002 MEDIUM fixed: metered tests now observe the third build (all-skipped → 0) and post-recompute steady state (→ 0). EXCL-001 LOW fixed: store() removes the temp sibling on write/rename failure. Orchestrator-verified: full suite 449 passed / 0 failed with the new assertions (they fail on the pre-fix code by construction — the reviewer measured 14 invocations on build 3).
