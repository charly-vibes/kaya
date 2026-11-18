---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-585-kaya-check, pipeline-step:orient]
---

ORIENT kaya-585

Ticket: kaya check: read-only record verification (P1, feature, complexity m, lane afk).
Files: src/main.rs, src/verification/mod.rs. Base commit e1221f9 (5 commits behind HEAD e464639; LOW).
Depends: none. Blocks: kaya-a4e, kaya-ntv, kaya-3kg.

Spec: openspec/changes/add-record-verification, capability kaya.verification.
Scenarios owned by this slice (9):
- Read-only verification / Drift reported without writing
- No relaxed verification / Unverifiable ledger refused
- Drift report / Drifted block
- Drift report / Clean record
- Drift report / Missing generated document
- Exit contract / Usage error exits 2
- Reported paths and projection roots / Paths follow the root
- Property mirror / no_side_effects
- Property mirror / drift_exit

Testable statements:
1. check computes exactly the bytes build (docs/research) and publish (docs/public) would write, and writes/appends nothing.
2. check refuses a ledger whose worktree bytes do not extend HEAD bytes with kaya.ledger.uncommitted_history and no drift findings; no relax option.
3. Envelope kind check, ok:true when executed; drift entries {path, block?, kind: missing|extra|changed}; exit 0 clean, 1 drift, 2 usage/IO.
4. Reported paths are relative to --root; --out/--publish-out move roots; --root also locates .kaya/ledger.jsonl.
5. Usage errors produce a kaya.verification.* error envelope and exit 2.

Acceptance meter: cargo test --test verification; ah check --changes add-record-verification --run-tests (slice scenarios execute, 0 no-toml); git diff --exit-code after check over a drifted fixture.

