---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-8ot-kaya-check-facts, pipeline-step:orient]
---

ORIENT kaya-8ot: facts and canonical vocabulary (P1, complexity m, lane afk). Files: src/main.rs, src/verification/facts.rs. Depends kaya-585 (closed). Blocks kaya-8dg.

Spec: add-record-verification — Record facts for policy, Capture frontier fact, Canonical source and kind vocabulary, property facts_reported.
Testable:
1. check envelope reports events, ledger_digest, last_capture_commit, newest_event_at, events_by_source, emitted_kinds, omitted_kinds; kaya evaluates no thresholds.
2. last_capture_commit = newest capture.commit.observed data.sha, remains reported after the commit is rewritten/unreachable; null when none.
3. events_by_source keys are exactly wai/git/test-runner/model derived from capture family and provenance (seed human actor never appears); zeros included.
4. emitted_kinds and omitted_kinds are dotted family.kind names, deduplicated; omitted entries join family and kind with '.'.

Meter: facts scenarios execute; model=13 and never 'charly'; kind lists dotted. Anti-goals: no thresholds, no ledger writes.

