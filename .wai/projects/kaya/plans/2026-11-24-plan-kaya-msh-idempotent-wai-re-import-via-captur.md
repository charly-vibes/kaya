---
tags: [pipeline-run:tdd-ro5u-2026-11-24-kaya-msh-wai-adapter-idempotent-re-import, pipeline-step:plan]
---

PLAN kaya-msh: idempotent wai re-import via capture re-observation corrections.

Desired behavior (ledger append policy doc ledger.rs:210-218 + agent-session-capture proposal item 3):
- A1 unchanged-tree replay: second `kaya import wai` appends nothing, exits 0.
- A2 edited record, same declared timestamp: re-import appends one correction event (data.supersedes = stored id, data.title = new heading), exits 0; replay of the corrected state appends nothing.
- A3 ledger stays append-only; no stored line rewritten; kaya.event/v0 schema unchanged.

Out of scope:
- Ledger policy/schema changes; adapter identity scheme changes (event_id stays path+declared-ts based); turu bundle-digest dedup (separate proposal item); incremental per-entry dedup for turu; undated-file mtime re-observation semantics (fresh id = fresh observation by design).

Test cases to write FIRST (tests/cli_pipeline.rs, beside import_wai_appends_each_event_once:287):
1. reimport_of_unchanged_tree_appends_nothing — dated fixture `2026-11-24-design.md` via write_wai_design; import -> appended 1; import again -> appended 0, duplicates 1, exit 0. (Meter test; green today, guards the invariant.)
2. edited_record_yields_correction_never_id_reuse — dated fixture with `# first heading`; import (appended 1, first line id = O); rewrite same file `# second heading`; import -> exit 0, appended 1; last ledger line: data.supersedes == O, data.title == "second heading", id != O; first line bytes unchanged; import again -> appended 0, duplicates 1.
Expected failure: test 2 red today (import exits failure, label kaya.ledger.id_reuse, no correction line).

Narrow command: cargo test --test cli_pipeline reimport_of_unchanged edited_record_yields
Full verification: cargo test && ah check (lefthook gates run at ship commit: record-verify-staged, impact-gate-staged, spk lint, pretender).

Implementation (after red):
- src/adapters/mod.rs: pub(crate) fn reobservation_correction(event: &Event) -> Event — clone event; data gains "supersedes": event.id; new id = "evt-" + sha256_hex(format!("correction\x1f{}\x1f{data}", event.id))[..ID_DIGEST_LEN=26]; all other fields (schema, type, timestamp, actor, subject, provenance, artifacts) unchanged. Deterministic per (artifact identity, content-derived data state) -> replay of a corrected tree is byte-identical and dedupes; distinct title states mint distinct correction ids.
- src/main.rs cmd_import: in the append loop, on Err where err.label == kaya.ledger.id_reuse AND event.r#type starts_with "capture." -> append reobservation_correction(event), count its AppendOutcome normally (Appended/Duplicate); a refusal of the correction itself propagates (genuine anomaly, no loop). All other labels propagate unchanged. model.* collisions still hard-refuse.
- src/ledger.rs, src/adapters/wai.rs: untouched.

Edge cases: correction self-reference (corr id == E.id impossible barring sha collision — validate_supersedes refuses, surfaces); title reverted to original -> E line byte-identical -> plain Duplicate, no correction; git/turu capture.* get the same re-observation semantics for free (collisions only).

Commit shape: single feature commit `fix(import): re-observed captures correct instead of refusing (kaya-msh)` (tests + impl together at green); any tidy afterwards as separate `refactor` commit.

