---
tags: [pipeline-run:tdd-ro5u-2026-11-21-kaya-q6s-decision-evidence-link, pipeline-step:ship-close]
---

# SHIP kaya-q6s — Decision-to-Source evidence link

Run: tdd-ro5u-2026-11-21-kaya-q6s-decision-evidence-link — ship-close evidence.

## Approval
Human checkpoint covered by the session's batch approval (q5e + pre-approved
q6s and d0n), 2026-11-21.

## Deployed state
- openspec change add-decision-evidence-link archived as
  2026-11-21-add-decision-evidence-link; deltas applied to the deployed
  kaya-model spec (+2 requirements, ~3 modified).
- Spectacular bindings: 20 scenario tomls in the change overlay, archived to
  .espectacular/changes/archive/2026-11-21-add-decision-evidence-link/, with
  deployed copies under .espectacular/kaya-model/.
- Book specs regenerated (just specs-docs, 9 pages); docs/src/specs/kaya-model.md
  now carries the new closed set and mirror scenarios.

## Final verification (this session)
- cargo test: 30/30 suites ok, 0 failures (understanding suite 15/15).
- cargo clippy --all-targets -- -D warnings: clean.
- ah check (deployed): no issues, 0 findings.
- ah check --run-tests (deployed): 177 passed, 0 findings.
- openspec validate --changes --strict: pass.
- ro5u review CRITICAL 0; all High and Medium findings fixed and re-verified
  (per-decision attribution, marker gating, property-coverage mirror,
  closed-set rows, bindings, unpinned states).

## Commit/ticket state
- Atomic feature commit follows: src/model.rs, src/projection/report.rs,
  tests/model.rs, tests/understanding.rs, openspec (deployed + archive),
  docs/src/specs regen, .espectacular bindings, .wai run state, beads export
  with kaya-q6s closed.
- No push (fleet policy: push needs explicit authorization).

