---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-3kg-record-hooks, pipeline-step:quality-ledger]
---

QUALITY LEDGER kaya-3kg
Changed: lefthook.yml (pre-commit staged check with stale enforcement; pre-push just record-verify, both outside ddl-managed blocks); justfile (record-verify recipe).
Verified: temp record clean staged => exit 0; staged research-map drift => exit 1 naming the block; `just record-verify` on this repo => exit 0; cargo test/clippy unaffected (config only).
Review: self-review 0 findings.
Risks: local hooks depend on the global core.hooksPath shim; CI runs the same command only after kaya-tm8.
Next: kaya-8dg lag gate.

