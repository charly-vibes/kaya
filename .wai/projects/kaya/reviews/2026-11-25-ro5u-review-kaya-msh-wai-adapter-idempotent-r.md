---
reviews: 2026-11-24-green-kaya-msh-command-cargo-test-result-full-s.md
verdict: pass
tags: [pipeline-run:tdd-ro5u-2026-11-25-kaya-msh-wai-adapter-idempotent-re-import, pipeline-step:fix-review]
---

# RO5U Review - kaya-msh (wai adapter idempotent re-import)

Verdict: READY · FINAL: CRITICALS=0 · Converged stage 5 · Reviewer subagent on uncommitted working tree.

## Scope reviewed

3-file diff (+196/-2): src/adapters/mod.rs four pub helpers (stable_identity, same_observation, latest_stored_same_identity, root_observation); src/commands/import.rs capture.*-gated dedup block before ledger.append; tests/cli_pipeline.rs two new integration tests (drifted date-less replay appends nothing; edited date-less supersedes root).

## Checklist

| Stage | Result |
|---|---|
| 1 Inventory | all hunks in full-file context; helpers called only from cmd_import |
| 2 Correctness | salt-inputs claim proven: dated edits re-mint root's id, so target.id = root.id makes the new arm byte-identical to the id_reuse arm; pinned dated tests traced step-by-step unchanged; ledger append/Duplicate/id_reuse semantics preserved |
| 3 Performance/scope | no new asymptotic class (dominated by append's full reload); no id-math change, no schema change, append-only preserved; gating verified: git/testrunner/turu/model events structurally cannot enter the block |
| 4 Adversarial | kind missing/null, cross-project identity, same-identity-per-run reachability, reverted date-less content generation salt, interleaved dated+date-less, heading-less→titled transitions - all converge to defined replay-stable behavior |
| 5 Converge | verdict below |

## Findings

| Dimension | Finding | Severity |
|---|---|---|
| Correctness | skip arm fires on parsed-data equality; a hand-edited stored line with same parsed content is counted as duplicate where the ledger docstring defines byte-refusal semantics. Exotic (manual ledger edit); consistent with "same facts" model; narrows a documented refusal path | P3 (conf 0.55) |
| Performance | per-event identity rescan allocates; worst case O(events × ledger lines), dominated by ledger.append's existing full-file reload | P3 (informational) |
| Maintainability | correctness leans on "no two same-identity capture events per import run" (true for all current adapters) - invariant unstated in code; one-line comment would localize it | P3 |
| Tests | drift-skip and edit-correct-replay arcs covered end to end; dated byte-identity pinned; no merge-blocking gaps | — |
| Security/Scope | no injection surface; separator collisions cannot mis-dedup (content equality gates both arms); scope claims verified | — |

FINAL: CRITICALS=0

