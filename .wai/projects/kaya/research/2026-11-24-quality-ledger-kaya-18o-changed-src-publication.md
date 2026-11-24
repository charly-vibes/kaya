---
tags: [pipeline-run:tdd-ro5u-2026-11-24-kaya-18o-collections-tier-tracer, pipeline-step:quality-ledger]
---

QUALITY LEDGER kaya-18o — Changed: src/publication/{plan,render,decls,error,source}.rs, src/model.rs (visibility), tests/publications/{build,export}.rs, docs. (cli_pipeline hunks landed with the kaya-y5v commit cbbea6e — shared file.) Verified: publications 32/32, publications_export 3/3, full suite 449/0 after final fixes. Review: RO5U CONVERGED, 0 C/H; both MEDIUMs (stray-fields refusal, false refusal message) fixed + verified. Design notes: glossary renders from compile-time vocabularies (zero IO — publish reads no ledger by design); refusal arm keeps single-constructor for the eight unimplemented kinds; plan_removals stays product-only (collections always emit). Follow-up filed: remaining eight collections = new bd ticket at ship. Next: none in-ticket.
