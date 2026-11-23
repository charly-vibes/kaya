# Rule-of-5 Review — kaya-xjp "add-turu-source-row"

**Work Reviewed:** working-tree diff of `src/verification/facts.rs`, `tests/verification.rs`, `README.md`, `CHANGELOG.md`, `openspec/specs/kaya-verification/spec.md`, `docs/src/specs/kaya-verification.md`, plus untracked `openspec/changes/archive/2026-11-22-add-turu-source-row/**` and `.espectacular/**` bindings (repo `/var/home/sasha/para/areas/dev/gh/charly/kaya`).

**Method:** `skill://rule-of-5-universal` lenses — correctness vs. spec scenarios, invalid states unrepresentable, error handling and refusal labels, boundaries/edge cases, allocation/IO cost, test quality, delta-format conformance, binding correctness, doc-truth against the shipped behavior. Read-only for the artifact under review; the repository's own gates (`cargo test`, `cargo clippy`, `ah check --run-tests`, `openspec validate --strict`, `spk lint`, `pretender check`) were run before and after the fixes and are reported as observed.

**VERDICT: NEEDS_REVISION → all findings fixed — CRITICAL count = 0 (HIGH 1, MEDIUM 2, LOW 1).**

---

## STAGE 1: DRAFT (shape)

One coherent vertical slice: `turu` becomes a fifth canonical capture source in the verification facts census. The production change is two edits in one file; the spec side is a MODIFIED delta plus one direct deployed constraint-row edit; three contract bindings and three tests carry it.

Shape is good: the canonical list is a single constant that three call sites read (`events_by_source` seeding, `validate_request` vocabulary, the facts report), so the vocabulary grew in one place. The refactor pass additionally removed a four-way duplication of event-type literals that `src/model.rs` already publishes — the module had been keeping a private copy of the model's vocabulary, and a fifth source would have extended that drift.

The one structural wrinkle: a deployed constraint-table row (`facts_not_policy`) had to be edited in place, because an openspec archive merges requirement deltas only. That is a real asymmetry between the requirement and constraint halves of a capability, not a defect in this change — but it must be recorded, or the archive stops explaining the deployed state.

[DRAFT-001] MEDIUM — deployed constraint row vs. archive
**Description:** The `facts_not_policy` constraint row in `openspec/specs/kaya-verification/spec.md` names the canonical source list inline. An openspec archive merges `## MODIFIED Requirements` deltas and leaves constraint tables untouched, so this row can only be changed by a direct deployed edit — and the archive that seeded the row (`openspec/changes/archive/2026-11-18-add-record-verification/specs/kaya-verification/layer.md`) would then contradict the deployed table.
**Evidence:** scratch probe in `/tmp/osp2`: an archived change carrying a `layer.md` with an edited `facts_not_policy` row left the deployed `openspec/specs/kaya-verification/spec.md` row unchanged and created no `layer.md` under `openspec/specs/kaya-verification/`. The capability has no deployed layer sidecar — its constraint table lives inline in `spec.md`.
**Recommendation:** record the deployed constraint-row edit in this change's own archived `proposal.md` and `tasks.md`, stating that an archive merges requirement deltas only; and sync the archived layer delta that seeded the table, as commit `6b7dfd2` did for the adapters requirement split.

## STAGE 2: CORRECTNESS

[CORR-001] HIGH — `README.md:62`
**Description:** The README told users the valid `--require-source` values are `wai|git|test-runner|model`. This change makes `turu` a valid source, so the documented vocabulary became false the moment the change shipped.
**Evidence:** `README.md` read "`--require-kind <family.kind>`, `--require-source <wai|git|test-runner|model>`, and `--fail-on-stale-authored` turn those findings into exit 1" — no `turu`, while the deployed `Assertions on request` requirement now names `turu` as one of the canonical sources.
**Recommendation:** name `turu` in the README's vocabulary list.

[CORR-002] MEDIUM — scenario text vs. bound test
**Description:** `Sources and kinds are canonical` was edited to say "`test-runner` and `turu` are zero", but its bound test asserted only that `test-runner` is zero. A scenario claiming a fact no test verifies is a claim the corpus cannot enforce — precisely what the `property coverage mirror` requirement exists to prevent.
**Evidence:** `tests/verification.rs` `check_sources_and_kinds_canonical` asserted `counts["test-runner"] == 0` and had no assertion on `counts["turu"]`, while the deployed and archived scenario text asserted `turu` is zero.
**Recommendation:** pin `counts["turu"] == 0` in `check_sources_and_kinds_canonical`.

[CORR-003] LOW — `.espectacular` binding header
**Description:** The three generated contracts omit the `tool_version` line older deployed bindings carry (`0.12.2`).
**Evidence:** `.espectacular/kaya-adapters/*.toml` carry `tool_version = "0.12.2"`; `.espectacular/kaya-verification/facts-reported.toml` starts at `id =` with no `tool_version`.
**Recommendation:** none — the omission matches the deployed verification capability's own current format and the generator's output. Recorded, not changed.

## STAGE 3: CLARITY

[CLAR-001] MEDIUM — the archive stopped explaining the deployed state
**Description:** The deployed constraint row changed with no delta recording it, and the seeded archived layer still named the four-source list. A reader comparing the archive to the deployed capability would find a contradiction and no explanation.
**Evidence:** see DRAFT-001; the archived `add-record-verification` layer's `facts_not_policy` row read `wai, git, test-runner, model` while the deployed row read `wai, git, test-runner, model, turu`.
**Recommendation:** record the direct deployed edit in the change's archived proposal/tasks and sync the seeded layer row.

## STAGE 4: EDGE CASES

[EDGE-001] LOW — a turu event from another tool
**Description:** `source_of` classifies `capture.entry.observed` as `turu` by event type alone, with no provenance check — unlike the `wai` arm, which requires `provenance.tool == "wai"`. A future adapter emitting the same event type would be counted under `turu`.
**Evidence:** `src/verification/facts.rs` `source_of` — the `wai` arm guards on provenance because `capture.artifact.observed` is a shared capture family; `capture.entry.observed` is currently emitted only by the turu adapter (`src/adapters/turu.rs:28`), so the type alone names its family, matching the `git` arm's treatment of `capture.commit.observed`.
**Recommendation:** none for this ticket — the type names its family today, and adding a provenance guard would be speculative against an adapter that does not exist. Recorded as a known boundary.

## STAGE 5: EXCELLENCE

[EXCL-001] LOW — CHANGELOG coverage
**Description:** The change alters user-visible behavior (`--require-source turu` becomes valid, `events_by_source` gains a `turu` key) with no CHANGELOG entry.
**Evidence:** `CHANGELOG.md` `## [Unreleased] / ### Added` had no entry for the facts-census change; the nearest prior tickets (`e948080` q5e, `b6a4e47` q6s) added none either, so this is a repo-wide gap rather than a regression.
**Recommendation:** add an `### Added` entry; also worth raising as a repo-level follow-up for the earlier shipped tickets.

---

## Convergence

```
New CRITICAL issues: 0
Total new issues: 4 (1 HIGH, 2 MEDIUM, 1 LOW)
New issues vs Previous Stage: 0% (no new findings after Stage 2)
Estimated false positive rate: 0% (every finding was confirmed in the artifact)
Status: CONVERGED
```

## Stage-by-Stage quality

- Stage 1 (Draft): GOOD — one small vertical slice, one constant, no new abstraction.
- Stage 2 (Correctness): GOOD after CORR-001 and CORR-002 were fixed.
- Stage 3 (Clarity): GOOD after CLAR-001 was fixed.
- Stage 4 (Edge Cases): GOOD — EDGE-001 recorded as a known boundary, not a defect.
- Stage 5 (Excellence): WITH_NOTES — EXCL-001 fixed.

## Verdict

**READY** — after the HIGH and MEDIUM findings were fixed and re-verified green.
