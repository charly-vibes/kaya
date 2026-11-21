---
tags: [pipeline-run:tdd-ro5u-2026-11-21-kaya-q5e-capture-attributes, pipeline-step:quality-ledger]
---

# QUALITY LEDGER kaya-q5e — capture attributes in wai events

Run: tdd-ro5u-2026-11-21-kaya-q5e-capture-attributes. Date: 2026-11-21.

## Gates and results (all run by orchestrator this session)
- RED pre-green: `cargo test --test wai` — 4 failed as mapped (3 derivation
  assertions + rename pin on missing-attribute precondition), 11 passed.
- GREEN: `cargo test` full — 30/30 suites ok, 0 failures.
- REFACTOR: `cargo test` full — 30/30 ok; `cargo clippy --all-targets --
  -D warnings` clean.
- FIX-REVIEW: `cargo test` full — 30/30 ok (0 failures, no panics);
  `cargo clippy --all-targets -- -D warnings` clean.
- ro5u-review: CRITICAL 0, verdict pass (conditional); 2 High + 1 Medium + 5 Low
  found; H-1, H-2, M-1, L-3, L-4, L-5 fixed and re-verified.

## Remaining known risks
- L-1 (empty `# ` heading yields empty title + digest-of-empty) and L-2
  (BOM-prefixed heading not captured) accepted spec-literal this ticket;
  behavior-level changes requiring delta rewording — to be ticketed.
- `.espectacular` bindings written at deployed-spec paths; `ah check` /
  `ah check --changes add-capture-attributes` still to be run at ship step;
  if the change overlay prefers `.espectacular/changes/<change>/` paths the
  tomls will be moved before commit.

## Next action
Ship-close: human approval, then `openspec` deployment of
add-capture-attributes, `just specs-docs`, `ah check`, atomic commit,
`bd close kaya-q5e` + beads export.

