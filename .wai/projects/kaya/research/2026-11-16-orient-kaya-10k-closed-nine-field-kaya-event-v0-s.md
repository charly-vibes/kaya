---
tags: [pipeline-run:tdd-ro5u-2026-11-16-kaya-10k-event-ledger, pipeline-step:orient]
---

ORIENT kaya-10k: closed nine-field kaya.event/v0 schema + append-only JSONL ledger. Acceptance: schema/type/timestamp/provenance/closed-set refusals, append-only prefix, duplicate replay no-op, id-reuse refusal, supersede unchanged bytes, digest stability, crash-tail recovery. Files src/event.rs src/ledger.rs.
