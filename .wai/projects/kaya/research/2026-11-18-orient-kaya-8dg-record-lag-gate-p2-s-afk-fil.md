---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-8dg-record-lag-gate, pipeline-step:orient]
---

ORIENT kaya-8dg: record lag gate (P2, s, afk). Files: scripts/record-lag.py, justfile. Depends kaya-8ot (closed).
Acceptance: a record 52 commits behind exits 1 reporting the count; a current record passes; a PR run prints a warning instead of failing. No re-import, no second ledger parser.
Envelope facts: `kaya check` JSON (last_capture_commit). Count git commits since that sha touching src/, docs/, openspec/, .wai/; bound 25 configurable in the just recipe; --mode warn|fail.

