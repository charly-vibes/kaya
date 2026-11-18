---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-7hm-authored-staleness, pipeline-step:quality-ledger]
---

QUALITY LEDGER kaya-7hm
Changed: src/projection/report.rs (document labels, authored block names, citation validation, latest-correction map, version-aware acknowledged(), stale narrowing, recorded event count); src/main.rs (build citation validation + ack subtraction; `kaya acknowledge` with reviewed maps, canonical labels, equivalence); src/event.rs (three-segment grammar plus documented acknowledgment exception); src/verification/{mod,assertions}.rs (check citations, ack resolution); tests/staleness.rs (15 tests); tests/event.rs; 13 TOMLs; README + dossier citation.
Verified: cargo test all suites; staleness 15/15; clippy -D warnings; fmt; ah check --changes clarify-authored-staleness --run-tests 49 passed / 0 findings; just ci; openspec validate --strict; spk lint clean; dogfood `kaya check` clean with the dossier citation.
Review: RO5U 0 Critical, 1 High, 2 Medium; all fixed.
Risks: acknowledgment review is per (document, block, id) and the latest correction; `recorded_event_count` keeps verification events out of the research-map summary so replays are byte-identical.
Next: repository gates (kaya-3kg).

