---
tags: [pipeline-run:tdd-ro5u-2026-11-19-kaya-1y9-ledger-lock-recovery, pipeline-step:green]
---

REFACTOR 1y9: LockGuard RAII (File, try_lock/unlock at drop); load_inner and append share one held guard spanning dedupe-validate-write; load_optional deleted (absorbed); labels kaya.ledger.locked / kaya.ledger.io follow house style. No duplication; deliberate no-op otherwise.
