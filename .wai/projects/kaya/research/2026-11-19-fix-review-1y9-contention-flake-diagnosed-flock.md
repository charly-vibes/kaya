---
tags: [pipeline-run:tdd-ro5u-2026-11-19-kaya-1y9-ledger-lock-recovery, pipeline-step:red]
---

FIX-REVIEW 1y9: contention flake diagnosed (flock release visibility transient under oversubscription, not a product bug — interposer trace shows unlock completed yet retry saw EWOULDBLOCK for under a millisecond); test fixed with 2s-bounded retry on the post-release append, 5/5 stable under reproducing load; instrumentation kept out of the repo.
