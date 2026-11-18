---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-ntv-kaya-check-assertions-stale, pipeline-step:orient]
---

ORIENT kaya-ntv: assertions and stale enforcement (P2, complexity m, lane afk). Files: src/main.rs, src/verification/assertions.rs. Depends kaya-585/8ot (closed).

Spec: add-record-verification — Assertions on request (2 scenarios), Staleness enforcement (2), property required_items and stale_violation.
Testable:
1. --require-kind <dotted family.kind> repeatable: an omitted kind is a violation naming it and reason no-records; without the flag plain check exits 0.
2. --require-source <wai|git|test-runner|model> repeatable: a zero-event source is a violation naming it; unknown values are usage errors (exit 2, kaya.verification.*).
3. --fail-on-stale-authored turns a non-empty stale report into exit 1 naming block and evidence ids; without the flag check exits 0 and reports stale; build keeps reporting stale without failing; authored bytes unchanged.
4. Envelope data carries required[] and stale[]; drift/exit contract unchanged.

Envelope shape: required entries {area:"kind"|"source", name, reason:"no-records"|"no-events"|"absent"}; stale entries {block, evidence:[ids]}.
Meter: assertions and stale scenarios execute, 0 no-toml. Anti-goals: plain check stays exit 0 on stale/omitted; build behavior unchanged.

