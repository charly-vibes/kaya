---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-8ot-kaya-check-facts, pipeline-step:quality-ledger]
---

QUALITY LEDGER kaya-8ot
Changed: src/verification/facts.rs (new Facts: canonical sources by type+provenance, sha-bearing capture frontier by parsed time, dotted emitted/omitted kinds); src/verification/mod.rs (ledger digest, omitted-kind collection, Outcome.facts); src/main.rs (seven fact fields in the check envelope); tests/verification.rs (5 facts tests + helpers); 4 TOMLs.
Verified: verification suite 23/23; full cargo test; clippy -D warnings; fmt; ah check --changes add-record-verification --run-tests 54 passed / 6 no-toml (assertions, stale); just ci green; dogfood facts events=80, sources git20/model13/wai47/test-runner0, frontier 645a6b5, digest 005f99…, dotted kinds.
Review: RO5U 0 Critical, 0 High, 1 Medium, 3 Low; all fixed.
Risks: no thresholds by design; assertions and stale enforcement next.
Next: kaya-ntv assertions and --fail-on-stale-authored.

