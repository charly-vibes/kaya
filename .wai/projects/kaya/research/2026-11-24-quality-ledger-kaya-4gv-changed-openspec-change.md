---
tags: [pipeline-run:tdd-ro5u-2026-11-24-kaya-4gv-finding-to-source-relationship-delta, pipeline-step:quality-ledger]
---

QUALITY LEDGER kaya-4gv — Changed: openspec/changes/add-finding-source-link/ (5 files: proposal, specs/kaya-model delta MODIFIED-only, design, tasks 1.1-1.4+2.x+3.x incl. bindings 2.2 + ah check 3.3, .openspec.yaml). Verified: openspec validate --strict valid; delta-vs-deployed byte-identity checked by reviewer; layer sidecar mirror 20/20. Review: RO5U CONVERGED, 0 CRITICAL; HIGH process finding (bindings convention) fixed and verified. Risks: bindings tomls named in tasks are AUTHORED at consumption time (the consuming AFK ticket creates them alongside its tests) — recorded so archive-time ah check has them; no kaya-model layer.md deployed (pre-existing). Next: none in-ticket.
