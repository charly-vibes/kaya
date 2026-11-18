---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-8dg-record-lag-gate, pipeline-step:plan]
---

PLAN kaya-8dg
- scripts/record-lag.py: run `cargo run --quiet -- check`, parse the single envelope; require ok and a string last_capture_commit (else exit 1 in fail mode / warn in warn mode); `git rev-list --count <sha>..HEAD -- src docs openspec .wai`; compare to --bound (default 25); print `record lag: N commits since <sha12> (bound B)`; exit 1 in fail mode above bound, 0 otherwise; --mode warn always exits 0. No ledger parsing.
- justfile: `record-lag mode="fail" bound="25": python3 scripts/record-lag.py --mode {{mode}} --bound {{bound}}`.
- Verification: repo run reports its actual lag and exits 1 (bound 25); `--mode warn` exits 0; py_compile clean.
Commit: chore(gates): add record lag gate. Closes kaya-8dg.

