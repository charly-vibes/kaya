---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-585-kaya-check, pipeline-step:fix-review]
---

RO5U-FIXES kaya-585: HIGH fixed — plan_research now takes Source and passes each existing document's bytes to index/report/reuse planners, so authored prose and foreign blocks are a fixed point and not drift. MEDIUM fixed — stray publication sweep skips plan.removals paths; drift is sorted and deduped. MEDIUM fixed — fixture seeds a foreign managed block and a kaya:authored block before build; added check_reports_publication_drift and check_reports_missing_publication_export. Verified: cargo test --test verification 11/11; full cargo test green; clippy -D warnings; fmt; ah check --changes add-record-verification --run-tests 45 passed, 15 no-toml (later slices); dogfood `kaya check` on this repo returns empty drift.

