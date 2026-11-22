---
tags: [pipeline-run:tdd-ro5u-2026-11-21-kaya-q6s-decision-evidence-link, pipeline-step:fix-review]
---

# RO5U-FIXES kaya-q6s — Decision-to-Source evidence link

Run: tdd-ro5u-2026-11-21-kaya-q6s-decision-evidence-link (step 7/9 fix-review).
Review: /tmp/kaya-q6s/ro5u-review.md — CRITICAL 0, verdict NEEDS_REVISION
(HIGH 3, MEDIUM 7, LOW 4).

Execution note: two dispatched fix subagents died on provider quota
(429 Token Plan / rate limit) before writing anything; the fixes below were
applied by the orchestrator, then verified with the same gates.

## HIGH-1 (DRAFT-001) — attribution decided per decision, not per ledger
`src/projection/report.rs` `render_decision`: citations are collected per
decision. A decision with at least one `cites` edge renders only the
Sources it cites (`- {source} — {reference} — see {decision}`); a decision
with none keeps the ledger-global rows and gains one marker naming it:
`- attribution-stale — decision {decision_id}: no decision-source links in
this ledger; source evidence is ledger-global` (`attribution_stale_row`,
exported). Mixed ledgers now behave per decision.
New tests: `mixed_ledger_scopes_the_cited_decision_and_marks_the_uncited_one`.

## HIGH-2 (CORR-001) — the marker never creates an Evidence section
A decision record with no complete Source renders no `### Evidence` section
and no marker (pre-change behaviour): the marker is emitted only when the
decision actually has ledger-global rows to mark.
New test: `decisions_only_ledger_renders_no_evidence_section`.

## HIGH-3 (CLAR-001) — property coverage mirror stays true
`openspec/changes/add-decision-evidence-link/specs/kaya-model/spec.md`
MODIFIES the deployed "Property coverage mirror" requirement: the six
deployed scenarios are carried over and ten mirror scenarios are added, one
per new layer property row, each with a `**VERIFIES**` line.

## MEDIUM — closed-set rows, bindings, marker wording, unpinned states
- Closed-set and direction rows (spec.md + layer.md exprs) name the set
  including `cites`; the "ten admitted relationship kinds" cardinality is
  gone from `src/model.rs`.
- Marker wording narrowed to source attribution ("source evidence is
  ledger-global"); finding rows stay decision-attributed.
- `source_cited_by_two_decisions_renders_one_row_per_reference` has its own
  test fn and its own binding; `unknown_relationship_refusal_names_cites_in_the_closed_set`
  got a binding and a layer property row.
- Tests import `attribution_stale_row` from the crate instead of copying the
  literal; the stale RED-phase `cites()` helper is gone.
- Previously unpinned states now have tests: duplicate citation, citation to
  an incomplete-only Source
  (`duplicate_citation_renders_one_row_and_incomplete_citation_renders_none`).
- Bindings: 20 tomls in the change overlay plus 20 deployed copies under
  `.espectacular/kaya-model/`; id == slug == filename.

## Verification (orchestrator, after fixes)
- `cargo test`: 30/30 suites ok, 0 failures (understanding suite 15/15).
- `cargo clippy --all-targets -- -D warnings`: clean.
- `openspec validate --changes --strict`: pass.
- `ah check --changes add-decision-evidence-link`: 0 findings.
- `ah check --changes add-decision-evidence-link --run-tests`: 177 passed,
  0 findings.

## Deferred
None of the review findings were deferred. L-1 (empty `# ` heading) and
L-2 (BOM heading) belong to kaya-q5e and stay deferred there.

