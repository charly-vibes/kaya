---
tags: [pipeline-run:tdd-ro5u-2026-11-24-kaya-msh-wai-adapter-idempotent-re-import, pipeline-step:orient]
---

ORIENT kaya-msh: wai adapter re-import idempotency (kaya.ledger.id_reuse on re-observed records).

Mechanics confirmed (scout brief, file:line):
- src/adapters/wai.rs event_id() hashes (TOOL, SOURCE_VERSION, project, declared-ts, path, {kind,path}) — content-derived title/title_digest are added AFTER id derivation (add_capture_attributes), so an edited record keeping its declared timestamp (dated filename) derives the SAME id with DIFFERENT bytes.
- src/ledger.rs append(): byte-exact replay = AppendOutcome::Duplicate; same id + different bytes = refuse LABEL_ID_REUSE "kaya.ledger.id_reuse" with guidance to append a correction event naming the id in data.supersedes. validate_supersedes: target must exist, no self-ref, no cycles. Ledger is append-only, one write_all per line; nothing is ever edited in place.
- cmd_import (src/main.rs:673-716) appends per event; a mid-loop refusal leaves earlier events of the run appended (the observed 417->419 then permanently failing state: the colliding record refuses on every replay, the run never completes).
- No adapter reads the ledger today; the only dedup is the ledger's byte-exact Duplicate.
- Corrections on the wire: ordinary kaya.event/v0 line, data carries supersedes; resolve_effective patches the target's data, keeping id/type/subject/timestamp.

Acceptance restated as testable statements:
A1. `kaya import wai` twice on an unchanged tree: second run exits 0, appended == 0 (byte-exact duplicates).
A2. Same dated record with a changed heading: re-import exits 0 (never kaya.ledger.id_reuse), appends exactly one correction event whose data.supersedes names the original stored id and whose data.title carries the new heading; a third import appends nothing.
A3. Stored lines are never rewritten (append-only preserved); first line bytes identical after corrections.
A4. No kaya.event/v0 schema change; no ledger policy change; model.* and other non-capture collisions still refuse.
A5. Anti-goals held: no edit-in-place, ledger append-only, no new data fields on freshly observed events (historical lines must stay byte-replayable — adding any attribute would break byte-exact dedupe for every stored wai line).

Anti-goal discovered during orient: the correction path must NOT add attributes to observed events (would change every stored line's bytes on re-import and mass-refuse). Corrections carry the new capture attributes only in the correction line itself.

