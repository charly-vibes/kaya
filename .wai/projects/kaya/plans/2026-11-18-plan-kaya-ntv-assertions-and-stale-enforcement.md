---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-ntv-kaya-check-assertions-stale, pipeline-step:plan]
---

PLAN kaya-ntv — assertions and stale enforcement

Design:
- New src/verification/assertions.rs (Purpose/Responsibilities/Rationale): validate requested kinds/sources,
  compute required[] from facts, compute stale[] from changed_event_ids + expected documents + the empty
  acknowledgment map (7hm later supplies acknowledgments).
- verification::check takes Options { index, require_kinds, require_sources, fail_on_stale }; Outcome gains
  required and stale; exit_code = 1 when drift or required is non-empty, or fail_on_stale and stale is non-empty.
- Envelope: data.required entries {area,name,reason}; data.stale entries {block,evidence}.
- main.rs adds repeatable --require-kind/--require-source and --fail-on-stale-authored.
Out of scope: cites validation, acknowledgment events/command (kaya-7hm); build behavior unchanged.

Tests first:
- check_required_kind_omitted: seed-only record, --require-kind understanding.decision-record => exit 1,
  required[{area:kind,name,reason:no-records}]; without flag exit 0.
- check_required_source_absent: --require-source test-runner => exit 1, reason no-events; unknown source => exit 2.
- check_stale_advisory_by_default: fixture authored block cites a seed event id; append a valid correction event
  (data.supersedes target, changed attributes) => check exits 0, stale names authored-1 and the target id.
- check_stale_enforced: same with --fail-on-stale-authored => exit 1, same naming; bytes unchanged; build exits 0.
- check_required_items (property) and check_stale_violation (property) mirror the invariants.
TOMLs: required-kind-omitted, required-source-absent, advisory-by-default, stale-authored-block-fails-the-check,
required-items, stale-violation.

Commands: cargo test --test verification; cargo test; clippy; fmt; ah check --changes add-record-verification
--run-tests; just ci.
Commit: feat(check): enforce required kinds/sources and stale authored blocks. Closes kaya-ntv.

