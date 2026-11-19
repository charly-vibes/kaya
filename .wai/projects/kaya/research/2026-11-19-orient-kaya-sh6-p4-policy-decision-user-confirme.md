---
tags: [pipeline-run:tdd-ro5u-2026-11-19-kaya-sh6-replay-policy, pipeline-step:orient]
---

ORIENT kaya-sh6 P4: policy decision (user-confirmed: document-refuse). Dedupe stays BYTE-EXACT (src/ledger.rs byte-compare before id check); a semantically-equal replay whose stored line was hand-edited (e.g. duplicate-key last-wins, whitespace, key order) refuses with kaya.ledger.id_reuse rather than deduping — protective: the log never silently absorbs divergent bytes. Fix: document the policy on Ledger::append's doc comment + the dedupe section; pin it with a test (hand-edited stored line, then append canonical event with same id -> id_reuse refusal, bytes unchanged). No behavior change.
