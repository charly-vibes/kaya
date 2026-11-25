---
tags: [pipeline-run:tdd-ro5u-2026-11-25-kaya-l46-human-verification-trail-attributed-and-content-bearing, pipeline-step:quality-ledger]
---

QUALITY LEDGER kaya-l46: changed src/commands/impact.rs (reviewed dispositions bind dependency digest), src/main.rs (Ir --walk flag + dispatch + IrCollector pub(crate)), src/commands/publication.rs (walk branch: provenance resolution via one-pass index, latest-event semantics, per-record trail with digest freshness, owned-set records section), tests/acceptance.rs (4 tests: digest binding + walk resolution/staleness + plain-ir guard + acknowledged trail; one approved needle-escape edit). Oracles: red first (digest Null, --walk unknown arg), then acceptance 6/0, suite 514/0, fmt clean, clippy -D warnings clean. Residual: reviewed-digest freshness is surfaced only in the walk (impact gate deliberately unchanged - waivers lapse, reviews do not; recorded as design decision). Review READY CRITICALS=0 after one fix round (4 fixed).
