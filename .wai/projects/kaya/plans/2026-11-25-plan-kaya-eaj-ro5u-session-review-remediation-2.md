---
tags: [pipeline-run:tdd-ro5u-2026-11-25-kaya-eaj-session-review-remediation-four-findings, pipeline-step:plan]
---

# Plan — kaya-eaj: ro5u session-review remediation (four findings)

Source: repo-scope ro5u review (Ro5uSessionReview, verdict READY CRITICALS=0,
4 findings). One remediation run, four targeted changes.

## CLAR-001 — walk reader doc + spec delta (MEDIUM)

- Reader doc: `docs/src/` mdBook source (SUMMARY.md). `docs/public/reference.md`
  is a GENERATED managed block — never hand-edit it. Add one reader page
  `docs/src/verification-walk.md` (run the walk, read `trail[].fresh`:
  `true` = digest matches current content, `false` = re-review needed, `null` =
  event carries no digest — legacy or acknowledge), wire into SUMMARY.md under
  a Verification heading.
- Spec delta: `openspec/specs/kaya-impact/spec.md` gains the reviewed-events-
  bind-digest requirement (mirrors the waiver language already at :44,:61);
  `openspec/specs/kaya-projection/spec.md` gains the walk-surface requirement
  with Properties mapped to the existing l46 test generators
  (`ir_walk_resolves_provenance_and_trail`, `ir_walk_includes_acknowledged_blocks`,
  `reviewed_disposition_binds_evidence_digest`). Gate: `just validate` must
  stay green.

## EDGE-001 — workflow failure attribution (LOW)

`.github/workflows/record.yml` commit/push block: after a failed
`git pull --rebase`, test `-d .git/rebase-merge`; when set, emit
"snapshot push abandoned: rebase conflict — resolve per scripts/record-conflicts.md"
and exit 1 immediately (no retry can fix it); drop the final 25s sleep after
the last attempt.

## EDGE-002 — dedup vs legacy digest-less reviews (LOW)

`cmd_disposition` (src/commands/impact.rs:162-176): the whole-`data` `existing`
compare never matches a legacy `verification.record.reviewed` event that lacks
`digest`. Fix: when kind is `reviewed` and the stored event's data has no
`digest` key, compare with the candidate's digest dropped. Waived unchanged
(always carried digest). Red test: fixture records reviewed once (current
shape), clones its ledger line, strips `data.digest`, mints a distinct id,
appends it as the legacy event, re-runs the identical review → `appended: false`.
(Pre-fix: second event appended.)

## EXCL-001 — shared authored-name bridge (LOW)

`src/impact.rs:805-813` owns `ir_block_name` under a "one seam, kept in one
place" comment; `record_trail` (src/commands/publication.rs:426-434)
re-implements the bridge. Export one helper from `kaya::impact`
(e.g. `pub fn ledger_block_names(ir_name: &str) -> Vec<String>` returning every
ledger spelling for an IR block name — `[ir_name]` or `[ir_name, "authored-1"]`),
use it in both consumers, and the comment becomes true again.

## Non-goals

- EDGE-003 (dogfood an acknowledgment) is a next-session act, not code.
- No behavior change beyond EDGE-002's dedup; EXCL-001 is behavior-preserving.

