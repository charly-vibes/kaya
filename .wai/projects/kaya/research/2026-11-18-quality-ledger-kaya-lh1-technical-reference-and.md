---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-lh1-reuse-families-technical-reference-knowledge-brief, pipeline-step:quality-ledger]
---

QUALITY LEDGER kaya-lh1 — technical-reference and knowledge-brief at the projection layer

Purpose: kaya.documents reuse-family kinds technical-reference (specodelic spec ids) and knowledge-brief (turu/whisper entry ids) now compile from reference-only Source attributes, with no ledger write and no payload copy. Scope decision: projection layer only; adapter ingestion stays deferred with kaya-syx/kaya-24c.

Changed:
- src/projection/reuse.rs: two new block/file constants, Inputs.specs/entries, four-document documents() in canonical kind order, build() validating all targets before any write (read_validated), independent spec/entry/revision selection, render_flat shared by the three single-section kinds, blank-reference filtering for the source map.
- tests/reuse_adapters.rs (new, 8 tests): input selection incl. independence and refusals, exact rows, ledger byte-identity plus markdown-only writes, empty-kind omission once each with no directory, stale removal for both adapter kinds, canonical order/vocabulary names, authored-byte preservation for both kinds.
- Expectation updates where the four-kind reuse family now reports omissions: tests/reuse.rs, tests/cli_pipeline.rs (five lists), tests/acceptance.rs.
- .espectacular/changes/add-document-taxonomy/kaya-documents/brief-adds-no-knowledge.toml: contract binding the scenario to two cargo selectors.

Verified:
- cargo test: 264 passed / 0 failed, 24 targets.
- cargo clippy --all-targets -- -D warnings: clean.
- cargo fmt --check: clean.
- ah check --changes add-document-taxonomy --run-tests: 15 passed, 19 structural (was 13/20 before this ticket).
- openspec validate --changes --strict: add-core-research-record passes; add-document-taxonomy fails on two pre-existing long-requirement warnings in kaya-publications/spec.md (no spec file modified here).

Review: RO5U verdict READY WITH NOTES. Critical 0, High 0, Medium 1 fixed, Low 4 (3 fixed, 1 tracked as comments on kaya-24c/kaya-syx). Convergence reached before stage 5; false positives 0.

Risks: `spec`/`entry` become meaningful Source attribute names, so a Source carrying a non-string value under either key now refuses the build — acceptable because no adapter emits them yet and the refusal names node and attribute; the deferred adapters must emit exactly these keys (recorded on both tickets). No end-to-end ingestion proof until kaya-syx/kaya-24c ship.

Next: `wai pipeline approve`, then ship-close commits the feature (separately from any tidy) and closes kaya-lh1; push stays unauthorized.
