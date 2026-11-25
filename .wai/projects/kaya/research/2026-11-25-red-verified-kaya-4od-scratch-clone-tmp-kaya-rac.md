---
tags: [pipeline-run:tdd-ro5u-2026-11-25-kaya-4od-scheduled-record-capture-races-in-flight-work, pipeline-step:green]
---

RED-VERIFIED kaya-4od: scratch clone /tmp/kaya-race, dirty marker RACE-PROBE.md uncommitted, just record-capture with CARGO_TARGET_DIR reuse: recipe exit 0, NO skip signal, ledger mutated in place (capture.artifact.observed x774 appended over dirty tree), docs/public + ledger modified. Demonstrates both hazards: local capture builds over half-finished work, and (workflow layer) push-triggered Actions capture colliding with in-flight commits. Green follows: remove push trigger from record.yml, flock+dirty-guard in justfile, scripts/record-conflicts.md.
