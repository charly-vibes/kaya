---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-qp8-publications-tracer, pipeline-step:quality-ledger]
---

QUALITY LEDGER kaya-qp8

Changed:
- src/publication/mod.rs (new): closed tier vocabularies (12 products / 9 collections / 8 packages), reader-question and admission tables, tier resolution and pair validation, Documents reader over compiled markdown (vocabulary filtering, first-wins across files, duplicate-block and malformed-markup refusals, empty bodies absent), build with admission/package/tier refusals before any write, justified emission with omission reporting and stale-page removal, fact-once cross-references, package navigation, deterministic page order, two-phase atomic writes, and publication.json export with emitted markers.
- src/lib.rs registration; src/main.rs  reading docs/research and declaring project-site over overview and reference.
- tests/publications.rs (25) and tests/cli_pipeline.rs (22) plus 4 kaya-publications contracts.
- docs/public/{reference.md,project-site.md,publication.json} generated; docs/src/document-taxonomy.md status corrected.

Verified:
- cargo test: 225 passed, 0 failed (16 targets).
- cargo fmt --all --check clean; cargo clippy --all-targets --all-features -- -D warnings clean.
- cargo run -- publish twice: second run recomputed [], unchanged [publication.json, publication.package.project-site, publication.product.reference]; overview omitted no-backing-document; documents [orientation.research-map].
- ah check --changes add-document-taxonomy --run-tests: 10 passed, 23 structural (remaining tickets).

Review:
- rule-of-5-universal, five passes: 0 critical, 0 high, 8 medium, 2 low; all fixed. The sharpest findings: page writes bypassed the managed block (authored bytes would be lost), the export contradicted itself about the omitted product, declarations could duplicate or mismatch tier fields, empty blocks produced stub pages, and page/export writes were not atomic.

Risks:
- Collections are refused until implemented (kaya-duv/kaya-scq); reference's technical-reference include has no producer yet, so the page carries the research map only.
- Publication pages use publication.<tier>.<name> blocks, which are outside the kaya.documents block vocabulary by design; validate_generated_blocks is not applied to docs/public.

Next:
- Ship: tick tasks.md 3.6, close kaya-qp8, export beads, commit, then start kaya-249 (understanding kinds).
