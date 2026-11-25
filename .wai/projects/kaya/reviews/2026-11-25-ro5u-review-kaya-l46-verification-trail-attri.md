---
reviews: 2026-11-25-fix-review-kaya-l46-verdict-ready-criticals-0-aft.md
verdict: pass
tags: [pipeline-run:tdd-ro5u-2026-11-25-kaya-l46-human-verification-trail-attributed-and-content-bearing, pipeline-step:ro5u-review]
---

# RO5U Review — kaya-l46 (verification trail attributed and content-bearing)

Verdict: READY (after fix round) · FINAL: CRITICALS=0 · Converged stage 5 · Reviewer subagent (TypeSafe-verified CORR-001 at 0.97 confidence). Initial verdict NEEDS_REVISION; all four findings fixed.

## Findings and dispositions

| ID | Severity | Finding | Disposition |
|---|---|---|---|
| CORR-001 | HIGH | `record_trail` matched `subject.id == record_id` — acknowledgements (subject = document label, block in `data.block`) could never appear in any walk trail; the feature's core deliverable silently under-reported for authored records | **FIXED** — record id split at last `#`: reviewed/waived exact-match; acknowledged matches document label + `data.block`, only when the id carries `#`. Plus reviewer-missed ledger/IR spelling bridge (`authored-1` vs `authored` for the first block, ordinal naming vs `authored_name`) so first-block acknowledgements match. New test `ir_walk_includes_acknowledged_blocks` pins the exact trail entry |
| CORR-002 | MEDIUM | walk `records[]` re-emitted trails of block-owned records, contradicting its own comment; "composed pages" claim false (composed records never exist in this IR) | **FIXED** — `owned` set of `{path}#{block}` ids excluded; comment now states exactly what the section is |
| EDGE-001 | LOW | O(events) scans per provenance id / per record; duplicate `record_trail` call in records filter+map | **FIXED** — one-pass `EventIndex` (by_subject + by_id, ledger-order preserved, last = latest); single `filter_map` |
| EDGE-002 | LOW | provenance resolved to origin event, not the supersede-chain successor | **FIXED** — latest event wins via index tail; event-id fallback; unresolved stays `{id, unresolved: true}` |

## Reviewer-verified safe (no change needed)

- Digest hoist invisible to the impact gate (waiver lapse loop filters `WAIVED_EVENT`; reviewed set reads only `data.record`).
- `fresh` tri-state honest: null ⇔ event carries no digest; record-never-materializes path unreachable.
- Provenance fallback cannot misattribute (single ULID space).
- Whole-`data` dedup: unchanged re-review still dedups; content change → distinct event.
- `IrCollector` `pub(crate)` justified (binary-crate-internal signature need).
- Walk key naming consistent with `DocumentIr`/event-data serde conventions.

## Fix-round verification (orchestrator)

`cargo test -q --test acceptance` 6 passed / 0 failed · full suite 514 passed / 0 failed · fmt clean · clippy `-D warnings` clean.

FINAL: CRITICALS=0

