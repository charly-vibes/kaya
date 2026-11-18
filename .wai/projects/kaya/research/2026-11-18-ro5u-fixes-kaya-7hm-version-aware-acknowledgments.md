---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-7hm-authored-staleness, pipeline-step:fix-review]
---

RO5U-FIXES kaya-7hm: version-aware acknowledgments — latest_corrections maps targets to the newest superseding event; ack events record `reviewed` (target→correction id); coverage applies only while the reviewed correction is still the latest, so a later correction re-raises the block and a re-acknowledge appends a new event. Type grammar restored to >=3 dotted lowercase segments with one explicit, documented `verification.acknowledged` exception; refusal message updated and two-segment refusal restored. Labels canonicalized via document_label_for (canonical path relative to cwd; outside-root documents refused); build and acknowledge use it. Added re-correction and absolute-path tests. Verified: staleness 15/15; full cargo test green; clippy -D warnings; fmt; ah check --changes clarify-authored-staleness --run-tests 49 passed 0 findings; just ci green.

