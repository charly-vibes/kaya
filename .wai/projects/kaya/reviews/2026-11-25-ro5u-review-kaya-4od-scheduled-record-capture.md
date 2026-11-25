---
reviews: 2026-11-25-fix-review-kaya-4od-all-four-actionable-findings.md
verdict: pass
tags: [pipeline-run:tdd-ro5u-2026-11-25-kaya-4od-scheduled-record-capture-races-in-flight-work, pipeline-step:ro5u-review]
---

# RO5U Review — kaya-4od (scheduled record-capture races in-flight work)

Verdict: READY (after fix-review) · FINAL: CRITICALS=0 · Converged stage 5 · Reviewer subagent, uncommitted working tree. Initial verdict NEEDS_REVISION; all findings fixed.

## Findings and dispositions

| ID | Severity | Finding | Disposition |
|---|---|---|---|
| CORR-001 | HIGH | `.kaya/capture.lock` untracked → after first run porcelain non-empty → permanent silent SKIP; CI would commit the lock and wedge local pulls | **FIXED** — `.gitignore` gains `.kaya/capture.lock`; verified `git check-ignore` |
| CORR-002 | MEDIUM | dirty check ran before lock acquisition; queued capture re-ran past a stale check over another capture's uncommitted output | **FIXED** — check moved under `flock` |
| CLAR-001 | MEDIUM | conflict doc's stage labels inverted for rebase (`:2:`=remote/base, `:3:`=replayed local; `--theirs`=local side under rebase) — reviewer verified experimentally in a scratch repo | **FIXED** — doc relabeled, `checkout --ours` for the remote-superset case, step-4 dedup note corrected |
| EDGE-001 | MEDIUM | nightly-only capture desyncs `record-lag` bound 25 (measured 42 record-relevant commits on 2026-11-24) → CI red on active days | **FIXED** — bound raised to 60 in `record-lag` + `record-check` with measured-day comment |
| EXCL-001 | LOW | no automated regression coverage for guard/trigger (a future revert of the push-trigger removal would be silent) | **ACCEPTED** — CI-only ops glue; the in-repo record (workflow comment + this review + ticket) documents the constraint; recorded here per quality-ledger convention |

## Reviewer-verified safe (no change needed)

- `github.actor != 'github-actions[bot]'` stays live for bot dispatches; nightly actor is the workflow editor (community discussion #47855).
- `concurrency: record` still meaningful (nightly ∥ queued dispatch).
- SKIP exit-0 safe in Actions (clean checkout; commit step no-ops via `git diff --cached --quiet`).
- `docs.yml` unaffected; nightly snapshot push still triggers it.
- flock fd-9 mechanics correct (released on exit, present on runners); stale-sha note consistent with kaya-verification spec :184-191.
- No docs/spec claim ties capture cadence to push.

## Fix-review verification (orchestrator)

Dirty main tree: `record-capture` → visible SKIP, exit 0, zero mutations · `git check-ignore -q .kaya/capture.lock` passes · `just record-check fail` passes at bound 60 (lag 3) · `cargo test -q` 510 passed / 0 failed · fmt clean.

FINAL: CRITICALS=0

