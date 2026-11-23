---
tags: [pipeline-run:tdd-ro5u-2026-11-22-kaya-xjp-verification-facts-census, pipeline-step:orient]
---

ORIENT kaya-xjp: P3 task, lane:afk, complexity:s, files [src/verification/facts.rs, docs/src/specs/kaya-verification.md], base_commit fbec965. Acceptance as testable statements: (1) capture.entry.observed events classify to canonical source 'turu' in events_by_source; (2) --require-source turu passes on a ledger holding one turu event and fails (exit 1, reason no-events) on a ledger without one; (3) --require-source turu is accepted as vocabulary (exit 0/1 path, never usage exit 2) since turu joins SOURCES; (4) facts_not_policy constraint row and the canonical source enumeration in the deployed verification spec name turu; (5) ah check clean, openspec validate --strict clean.

Surface map (read, no edits yet):
- src/verification/facts.rs:27 'pub const SOURCES: [&str; 4] = ["wai", "git", "test-runner", "model"]' -> must become 5 entries incl. turu.
- src/verification/facts.rs:100-110 source_of(): match arms CAPTURE_COMMIT->git, RUN_COMPLETED->test-runner, CAPTURE_ARTIFACT+provenance.tool==wai->wai, type starts_with model.->model, else None. turu's capture.entry.observed (src/model.rs:31 ENTRY_OBSERVED_EVENT, src/adapters/turu.rs:30 EVENT_TYPE) falls to None today: no arm matches, so turu events are invisible.
- src/verification/assertions.rs:79 validate_request checks SOURCES.contains -> adding turu to SOURCES makes --require-source turu valid automatically; error text at :84 uses SOURCES.join(', ').
- src/main.rs:566-571 cmd_import already names the source 'turu' for the import envelope; no change needed there.
- Spec rows hard-coding the four-source list: openspec/specs/kaya-verification/spec.md:26 (constraint facts_not_policy), :195-196 (Canonical source and kind vocabulary requirement), :205-206 (scenario Sources and kinds are canonical), :212-213 (Assertions on request requirement). docs/src/specs/kaya-verification.md:28,197-198,207-208,214-215 are the generated mirror (scripts/sync-specs.py) - regenerate via 'just specs-docs', never hand-edit.
- Spec delta format: openspec change dir with proposal.md + tasks.md + specs/kaya-verification/spec.md (MODIFIED Requirements) and optionally specs/kaya-verification/layer.md; archived under openspec/changes/archive/<date>-<name>/. Prior precedent: 2026-11-21-add-capture-attributes.
- Tests: tests/verification.rs:610 local 'const SOURCES: [&str; 4]' mirror (must become 5), :869-875 source_counts() asserts the key set equals SOURCES exactly, :1652 check_facts_test_runner_source (the pattern to mirror for turu), :1998 check_required_source_absent, :2163 check_required_items.
- .espectacular bindings: deployed .espectacular/kaya-verification/*.toml bind scenario headings to cargo test flags; a new scenario needs a change overlay binding + deployed binding (precedent: 2026-11-21-add-capture-attributes mirrored into .espectacular/kaya-adapters/).

Constraint noted: openspec validate --strict is fatal on requirement text over 500 chars.
