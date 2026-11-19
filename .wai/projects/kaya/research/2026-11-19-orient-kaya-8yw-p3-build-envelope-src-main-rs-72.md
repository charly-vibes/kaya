---
tags: [pipeline-run:tdd-ro5u-2026-11-19-kaya-8yw-build-envelope-events, pipeline-step:orient]
---

ORIENT kaya-8yw P3: build envelope (src/main.rs:728-739) exposes out/recomputed/unchanged/omitted/stale/sources/pending_proposals/ledger_digest but no events count, making status/build event-count parity unobservable (check exposes events at src/main.rs:852). Fix: cmd_build envelope gains data.events = effective event count (events.len() of the ledger it compiled — resolve_effective-applied count, consistent with what the model consumed). Red test: tests/cli.rs (or cli_pipeline) asserts build envelope data.events == seeded count. Tests may also now tighten the status_build suite's classifier-domain parity to include events parity (re-scoping the g11 exclusion) — kept minimal: one cli.rs assertion + optionally one parity line in the sweep.
