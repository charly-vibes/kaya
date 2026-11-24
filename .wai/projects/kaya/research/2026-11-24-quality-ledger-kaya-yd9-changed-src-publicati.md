---
tags: [pipeline-run:tdd-ro5u-2026-11-24-kaya-yd9-implement-add-collection-kinds, pipeline-step:quality-ledger]
---

QUALITY LEDGER kaya-yd9:

Changed:
- src/publication/collection.rs (new): renders the five implemented collections (glossary, topic-map, knowledge-index, example-gallery, api-catalogue) from the source map; row parsing is line-anchored; None = omission.
- src/publication/render.rs: compose_collections renders+omits in one pass; render_pages takes composed collection pages; export collection rows carry no-backing-document reasons derived from the omission summary; heading_anchor shared.
- src/publication/plan.rs: partition admits the five implemented names; plan merges product+collection omissions sorted by name; tier-aware page removals.
- src/publication/decls.rs: DEFAULT_COLLECTIONS declares all five implemented collections.
- src/publication/error.rs: tier_unimplemented refusal names the implemented set.
- src/publication/mod.rs: registers the collection module.
- tests/publications/{common,build,export,declarations}.rs: collection() helper, seven renderer contracts, narrowed refusal pin, omission-row export pin, default-collections pin.
- tests/cli_pipeline.rs: every pages/omitted/export/block pin flipped for the two render-now and two producer-backed collections across six publish scenarios.
- docs/public/*.md + publication.json + docs/research/*.md: dogfood refresh — real tree now carries topic-map + knowledge-index pages; gallery/catalogue omitted with reasons.

Verified:
- cargo test: full suite green (was red first: 10 publication/cli tests failed on the pinned missing behavior before any src change).
- ah check --run-tests: OK — 230 passed, 0 findings.
- openspec validate add-collection-kinds --strict: valid.
- Live dogfood publish (clean rebuild): 12 pages incl. topic-map (per-topic rows + architecture-design cross-ref) and knowledge-index (per-type census); api-catalogue + example-gallery omitted no-backing-document; omission summary sorted by name across tiers.
- cargo fmt --all --check clean; cargo clippy --all-targets --all-features -D warnings clean.

Review:
- RO5U converged stage 4, verdict READY: CRITICAL 0, HIGH 0, MEDIUM 2 (both fixed: sync-direction comment; line-anchored section parsing + regression test), LOW 1 resolved (EDGE-001 = the line-anchor fix).

Risks:
- api-catalogue/gallery row formats are pinned by unit tests but not yet exercised by a real producer block (no TECH doc or experiment-record exists in the dogfood yet); first real producer may need a row-shape follow-up.
- none known beyond that.

Next:
- ship-close: human approval, then atomic commit of src + tests + docs + wai artifacts; closes bd kaya-yd9; never pushes.
