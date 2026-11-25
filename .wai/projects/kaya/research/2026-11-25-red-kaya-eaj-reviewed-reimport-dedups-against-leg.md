---
tags: [pipeline-run:tdd-ro5u-2026-11-25-kaya-eaj-session-review-remediation-four-findings, pipeline-step:red]
---

RED kaya-eaj: reviewed_reimport_dedups_against_legacy_digestless_event fails appended true vs false - legacy digest-less reviewed event never matches the whole-data dedup (EDGE-002 isolated: only the legacy event present, reshaped valid seed envelope, subject Document casing matched). CLAR-001/EDGE-001 are artifact-only (docs/spec/yaml) - no local runtime oracle; EDGE-001 verified by execution only at nightly, documented like 4od.
