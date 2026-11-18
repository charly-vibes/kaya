---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-cc9-archive-and-deploy-document-taxonomy, pipeline-step:green]
---

GREEN kaya-cc9 — taxonomy bound, archived, deployed

Bindings: 19 contract files authored under .espectacular/changes/add-document-taxonomy/{kaya-documents,kaya-publications}/, one per unbound scenario, each cargo selector naming an existing executing test (mapping in ORIENT). One extra contract added after deployment for the renamed scenario and one for the new vocabulary scenario.

Archive: `openspec archive add-document-taxonomy --yes` moved both deltas into openspec/specs/{kaya-documents,kaya-publications}/spec.md and archived the change as 2026-11-17-add-document-taxonomy; `ah archive add-document-taxonomy` then deployed all contracts to .espectacular/{kaya-documents,kaya-publications}/ and removed the change contract directory.

Deployment fixes found and made:
- Layer sidecars are not copied by openspec archive, so kaya-documents/layer.md and kaya-publications/layer.md were copied beside their specs (spk now lints 2 files, 0 issues).
- Two archive Placeholder Purposes replaced with real capability purposes.
- Slug collision: the prose scenario "Unknown kind refused" and the property scenario unknown_kind_refused collide; the prose one is now "Kind outside the closed vocabulary refused" (deployed spec and archived delta) with its own contract bound to unknown_kind_is_refused_with_a_label_and_vocabulary.
- Strict validation: "Publication tiers" and "Reader-question binding" exceeded 500 chars. Tiers became one short requirement plus a "Tier vocabularies are closed" scenario pinning all three vocabularies (contract bound to tier_resolved_for_every_name_and_unknown_refused); the reader-question requirement became short with the full question/admission binding moved into the existing "Every product declares a product question" scenario body. openspec validate --specs --strict now passes both capabilities.
- Book: scripts/sync-specs.py renders spec.md + layer body into docs/src/specs/<capability>.md; `just docs` depends on the new `just specs-docs`; SUMMARY.md Design specs lists kaya.documents and kaya.publications. Generated pages committed like the other compiled docs.
- Archived tasks.md archive checklist ticked with evidence; 3.10 (n50), 3.11 (rdr) and 3.12 (aaz) stay unchecked because they are separate open tickets.

Verification: ah check --run-tests 36 passed / 0 findings; ah check 0 structural; spk lint openspec/specs ok (2 files, 0 issues); openspec validate --specs --strict 2 passed; openspec validate --changes --strict 1 passed; cargo test 24 targets green; just ci passed; cargo run build and publish both idempotent (second run recomputed []).
