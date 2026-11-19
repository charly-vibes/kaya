---
tags: [pipeline-run:tdd-ro5u-2026-11-19-kaya-1y9-ledger-lock-recovery, pipeline-step:refactor]
---

REFACTOR 1y9: LockGuard RAII over the ledger file; load and append share one guard spanning read-validate-write; unused load_optional absorbed; new labels kaya.ledger.locked and kaya.ledger.io follow house style; no-op otherwise.
