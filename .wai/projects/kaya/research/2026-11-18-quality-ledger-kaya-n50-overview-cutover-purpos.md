---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-n50-overview-cutover, pipeline-step:quality-ledger]
---

QUALITY LEDGER kaya-n50 — overview cutover

Purpose: make the compiled overview product the single owner of the overview answer, so README and the book point at it instead of restating it. Concept half stays hand-written and blocked.

Changed:
- docs/public/overview.md: kaya:authored interpretation (what/why/when/status) ahead of the generated publication block.
- README.md: removed the duplicated why blockquote body, "When to use" and the 28-line "The idea" section; now a one-line purpose plus links to the compiled overview and the book; taxonomy links updated to the deployed specs; the CLI envelope sentence now points at status.md.
- docs/src/index.md: opens with a pointer to the compiled overview; keeps the long-form rationale.
- docs/src/document-taxonomy.md: contract links point at deployed kaya-documents/kaya-publications specs; names the compiled overview as the reader-facing summary.
- docs/src/status.md: taxonomy row links the deployed specs and lists eleven implemented kinds.
- bd kaya-n50 comment records the concept blocker (no Finding/Decision/Observation/Claim/Run entities in the ledger).

Verified: normalized sentence-duplication sweep = 0 duplicate sentences across the five files (was 1 plus the discarded idea section); kaya:authored count = 1; all relative links resolve; kaya publish twice recomputed [] and the overview page bytes are identical; kaya build recomputed []; mdbook build green; just ci green; cargo test 24 targets green.

Review: RO5U verdict READY — Critical 0, High 0, Medium 0, Low 4 (all accepted or deferred with reason); convergence at stage 4; false positives 0.

Risks: the duplication meter is editorial, not a CI gate (deferred); README keeps a one-line semantic echo of the overview's opening (landing-page minimum); concept remains hand-written until understanding records exist.

Next: ship-close approval, commit the cutover, close kaya-n50; push stays unauthorized. Then kaya-rdr publish wiring preparation.

