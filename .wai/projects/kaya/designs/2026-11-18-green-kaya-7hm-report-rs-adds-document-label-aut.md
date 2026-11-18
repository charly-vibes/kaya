---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-7hm-authored-staleness, pipeline-step:green]
---

GREEN kaya-7hm: report.rs adds document_label, authored_block_names, validate_citations (kaya.projection.citation_marker naming document/marker/id), acknowledged(events, document), unacknowledged narrowing, and a verification-excluding recorded event count so replays stay byte-identical; build validates citations on every projection document and subtracts per-document acknowledgments; check validates citations before comparison and assertions::stale resolves acknowledgments; new `kaya acknowledge` (human-only, unknown document/block/event refusals, equivalent ack returns appended:false, no rebuild). Event type grammar relaxed to two dotted lowercase segments because the deployed kaya-event spec says "namespaced dotted path" and the acknowledgment spec names `verification.acknowledged`; tests/event.rs updated accordingly. Verified: staleness 13/13; full cargo test green; clippy -D warnings; fmt; ah check --changes clarify-authored-staleness --run-tests 49 passed 0 findings; dogfood check clean.

