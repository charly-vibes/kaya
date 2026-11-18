---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-cc9-archive-and-deploy-document-taxonomy, pipeline-step:quality-ledger]
---

QUALITY LEDGER kaya-cc9 — document taxonomy archived and deployed

Purpose: make the taxonomy enforceable and published — every kaya.documents and kaya.publications scenario now has an executing contract, both capability layers live in openspec/specs with their sidecars, the book renders them, and the change is archived.

Changed:
- .espectacular/kaya-documents/ (18 contracts) and .espectacular/kaya-publications/ (17 contracts): all 35 deployed scenarios bound to executing tests; the change contract directory was removed by `ah archive`.
- openspec/specs/{kaya-documents,kaya-publications}/{spec.md,layer.md}: deployed deltas plus copied layer sidecars, real Purpose sections, one slug collision resolved by renaming the prose scenario to "Kind outside the closed vocabulary refused", two over-long requirements reduced (tier vocabularies moved into a dedicated scenario; the reader-question/admission binding moved into the existing scenario body).
- openspec/changes/archive/2026-11-17-add-document-taxonomy/: archived change with the archive checklist ticked (3.10 n50, 3.11 rdr, 3.12 aaz stay unchecked as open tickets).
- docs/src/SUMMARY.md Design specs lists kaya.documents and kaya.publications; scripts/sync-specs.py + `just specs-docs` generate docs/src/specs/*.md; `just docs` depends on it.

Verified: ah check --run-tests 36 passed / 0 findings; ah check 0 structural; ah report 35/35 covered, 0 missing, 0 failing; spk lint openspec/specs ok (2 files, 0 issues); openspec validate --specs --strict 2 passed; openspec validate --changes --strict 1 passed; cargo test 24 targets green; just ci passed; mdbook build green; build and publish idempotent (second run recomputed []).

Review: RO5U verdict READY WITH NOTES — Critical 0, High 0, Medium 2 fixed (stray generated file; generator ownership/pruning), Low 3 (one fixed, two documented). Convergence reached at stage 4; false positives 0.

Risks: committed generated book pages can lag the deployed specs between builds (no CI freshness gate, by choice); the archived delta and deployed spec differ where deployment required prose edits, documented in GREEN; adapter ingestion for the two reuse kinds remains deferred to kaya-syx/kaya-24c.

Next: ship-close approval, then commit bindings+archive+book+evidence and close kaya-cc9; push stays unauthorized.
