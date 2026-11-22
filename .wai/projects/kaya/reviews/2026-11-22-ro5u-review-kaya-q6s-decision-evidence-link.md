# Rule-of-5 Review — kaya-q6s "add-decision-evidence-link"

**Work Reviewed:** working-tree diff of `src/model.rs`, `src/projection/report.rs`, `tests/model.rs`, `tests/understanding.rs`, plus untracked `openspec/changes/add-decision-evidence-link/**`, `.espectacular/changes/add-decision-evidence-link/**`, `.espectacular/kaya-model/*.toml` (repo `/var/home/sasha/para/areas/dev/gh/charly/kaya`).

**Method:** `skill://rule-of-5-universal` lenses — correctness vs. spec scenarios, invalid states unrepresentable, error handling and refusal label bytes, boundaries/edge cases, allocation/IO cost, test quality, delta-format conformance, binding correctness, renderer doc-comment accuracy. Read-only: no file in the repo was edited; only `git diff`, `git log`, `grep`, `glob`, and file reads were used. No `cargo`/`ah`/`openspec` command was run, so every compile/validate claim below is **UNVERIFIED** (self-reported validation; `TYPESAFE_API_KEY` not checked).

**VERDICT: NEEDS_REVISION — CRITICAL count = 0 (HIGH 3, MEDIUM 7, LOW 4).**

---

## STAGE 1: DRAFT (shape)

The change is one coherent vertical slice: a tenth relationship `cites` (Decision → Source) enters the closed set in `src/model.rs`, and `render_decision` in `src/projection/report.rs` grows a second attribution regime (scoped-per-citation vs. ledger-global-with-staleness-marker). Delta spec + layer sidecar + `.espectacular` bindings are all present, and the change follows the `add-capture-attributes` archive layout exactly.

Shape is good: the regime switch is one boolean (`scoped`), the marker is a single exported `const`, and the fallback path is byte-for-byte the old rendering plus one line. The weakness is not shape but **state modelling**: the regime is chosen per *ledger*, while attribution is per *decision*, and the states that fall between the two (mixed ledger; decision with zero cited sources; ledger with no evidence at all) are neither specified nor pinned by a test.

[DRAFT-001] HIGH — `src/projection/report.rs:878-901`
**Description:** The regime switch is ledger-global while the attribution it produces is per-decision. Three ledger states are representable but unspecified: (a) a *mixed* ledger (some decisions cite, some do not, plus uncited Sources) — every uncited Source silently disappears from rows **and** from the source map, with no staleness marker; (b) a decision that cites nothing in a scoped ledger renders no evidence row and no marker, indistinguishable from "has no evidence"; (c) a decision whose only cited Source is incomplete (`path` without `commit`) renders no evidence row either. In all three, the published document silently under-reports.
**Evidence:**
```rust
let scoped = graph
    .edges()
    .any(|edge| edge.relationship == Relationship::Cites);
```
and, for the scoped branch:
```rust
let Some(reference) = source_reference(to) else {
    continue;
};
```
**Recommendation:** Make the fallback per-decision rather than per-ledger, or make the marker per-decision: render the staleness marker for any decision with zero cited rows in a scoped ledger (and for a decision whose only citations are incomplete), so the document states "this decision has no attributable evidence" instead of silently omitting rows. Add the three states to the delta spec as scenarios and pin them in `tests/understanding.rs`.

---

## STAGE 2: CORRECTNESS (vs. the delta's own scenarios)

The four spec scenarios are implemented as written and their tests are honest:

- "Cites joins the closed set" — `Relationship::Cites` in `ALL` (`src/model.rs:159,174`), `as_str() == "cites"` (`:189`), `RELATIONSHIP_SET` gains `cites` (`:34`). ✓
- "Cites to an unknown node refused" — dangling resolution runs before direction in `build()` (`src/model.rs:385-400`), so `kaya.model.references_resolve` wins. ✓
- "Cites direction follows the grammar" — `fixed_direction` = `(Decision, Source)` (`:202`), `requires_exact_pair` includes `Cites` (`:217`); the refusal message embeds `Decision -> Source` via `required_from.as_str()`/`required_to.as_str()` (`:576-582`), so the `direction_fixed` label and rule text both hold. ✓
- "Evidence lists only cited sources" / "one row per citing decision" — the scoped branch pushes `(to.id, "{reference} — see {from.id}", line)`; sorting by `(id, key)` then yields source-id-then-decision-id order, and `dedup()` collapses identical `(source, decision)` tuples while keeping distinct decisions. ✓

No defect in these paths. The defects are in the states *around* them.

[CORR-001] HIGH — `src/projection/report.rs:941-948`
**Description:** The unscoped path appends the marker unconditionally, which changes the document shape for ledgers that have **no** evidence rows at all. `push_heading` skips an empty section, so before this change a decisions-only ledger rendered `### Decisions` and nothing else; now it renders an `### Evidence` section whose sole content is `- attribution-stale — evidence is ledger-global: this ledger records no decision-source links` — a claim about evidence that does not exist. This contradicts the change's own stated impact ("pre-relationship ledgers change by exactly the marker line"): here the delta is a whole new section, and it lands in published `understanding.decision-record` bytes.
**Evidence:**
```rust
let mut evidence_lines: Vec<String> = evidence.into_iter().map(|(_, _, line)| line).collect();
if !scoped {
    evidence_lines.push(ATTRIBUTION_STALE_ROW.to_string());
}
...
push_heading(&mut body, 3, "Evidence", &evidence_lines);
```
**Recommendation:** Emit the marker only when the Evidence section has content (i.e. `if !scoped && !evidence_lines.is_empty()`), or emit it as a per-decision note under `### Decisions` when a decision has no attributable rows. Add a decisions-without-sources fixture and pin the expected bytes either way.

[CORR-002] MEDIUM — `src/projection/report.rs:857-866` (doc comment) and `:945`
**Description:** The staleness marker asserts "evidence is ledger-global", but the unscoped regime still renders **influenced findings with per-decision attribution**: `- p/Finding/f1 — see p/Decision/d1` (`src/projection/report.rs:930-934`). So even in the "ledger-global" regime, part of the Evidence section is decision-scoped. The marker is therefore a false statement about the section it terminates, and the renderer doc comment repeats the claim ("sources render ledger-globally exactly as before the relationship existed" — true of Sources, but the section as a whole is not ledger-global).
**Evidence:**
```rust
pub const ATTRIBUTION_STALE_ROW: &str =
    "- attribution-stale — evidence is ledger-global: this ledger records no decision-source links";
```
**Recommendation:** Narrow the claim to what is true, e.g. `- attribution-stale — source attribution is ledger-global: this ledger records no decision-source links`, and state in the renderer doc comment that influenced-finding rows remain decision-attributed in both regimes.

[CORR-003] MEDIUM — `src/projection/report.rs:907` vs `:932` (row-key semantics)
**Description:** The two evidence row kinds carry incompatible sort keys: Source rows use `reference` (fallback) or `"{reference} — see {decision}"` (scoped) as the second tuple element, while finding rows use the bare `decision.id`. Within one source id the ordering still reduces to decision id (identical reference prefix), so the spec's "sorted by source id then decision id" holds — but only because the reference happens to be equal. Any future change that makes a source's reference vary per citation (e.g. per-revision pinning) silently reorders rows relative to the decision id.
**Evidence:**
```rust
evidence.push((
    finding.id.clone(),
    decision.id.clone(),
    format!("- {} — see {}", finding.id, decision.id),
));
```
**Recommendation:** Give both row kinds the same key shape — e.g. `(node_id, decision_id_or_empty, line)` — so sorting never depends on the reference text.

---

## STAGE 3: CLARITY (spec/delta conformance and reader legibility)

[CLAR-001] HIGH — `openspec/changes/add-decision-evidence-link/specs/kaya-model/spec.md` (whole file) vs `openspec/specs/kaya-model/spec.md:114-117`
**Description:** The deployed capability spec states a hard invariant the change breaks:
> "Every property row in the layer sidecar SHALL be verified by exactly one dedicated scenario."

The change's layer sidecar adds six property rows (`cites_joins_the_closed_set`, `cites_to_an_unknown_node_refused`, `cites_direction_follows_the_grammar`, `evidence_lists_only_cited_sources`, `source_cited_by_two_decisions_renders_one_row_per_reference`, `pre_relationship_ledger_falls_back_to_global_attribution_with_staleness_marker`), but the delta never MODIFIEDs the "Property coverage mirror" requirement, and the deployed spec's mirror section still carries only six `**VERIFIES**` scenarios. Only three of the six new properties have a corresponding scenario at all (the two `cites` model scenarios and the direction scenario ride inside the MODIFIED requirements and carry no `**VERIFIES**` line); the three projection-side properties (`evidence_lists_only_cited_sources`, `source_cited_by_two_decisions_renders_one_row_per_reference`, `pre_relationship_ledger_falls_back_to_global_attribution_with_staleness_marker`) have **no** scenario and no `**VERIFIES**` entry anywhere in the delta. After deploy, the deployed spec's own mirror requirement is false.
**Evidence:**
```
| evidence_lists_only_cited_sources | unit | [[kaya.model.layer.decision_evidence_scoping]] | ...
| source_cited_by_two_decisions_renders_one_row_per_reference | unit | ...
| pre_relationship_ledger_falls_back_to_global_attribution_with_staleness_marker | unit | ...
```
and `grep -n "VERIFIES" openspec/changes/add-decision-evidence-link/specs/kaya-model/spec.md` → no match.
**Recommendation:** Add `## MODIFIED Requirements` → `### Requirement: Property coverage mirror` to the delta spec.md with one scenario per new property row, each carrying `- **VERIFIES** [[kaya.model.layer.<row>]]`, matching the deployed mirror's shape.

[CLAR-002] HIGH — `openspec/specs/kaya-model/spec.md:21` and `:24` vs `openspec/changes/add-decision-evidence-link/specs/kaya-model/layer.md:24,27`
**Description:** The acceptance criterion "closed-set + direction rows updated consistently in spec.md AND the layer.md sidecar exprs" is only half-met. The layer sidecar is updated:
> `relationships come from the closed set {investigates, predicts, executes, observes, supports, challenges, influences, realizes, derives, cites} — growth only by a schema revision`
> `... Model realizes Hypothesis, Decision cites Source — reversed edges are refusals, never coercions`

but the **deployed** spec's own constraint rows still enumerate nine relationships and omit "Decision cites Source", and `docs/src/specs/kaya-model.md:23,26` (generated from it) still shows the nine-name set. Post-deploy, a reader of the deployed capability or the book sees a closed set of nine while the code admits ten and refuses `depends_on` with a message naming ten.
**Evidence:**
```
| relationship_set_closed  | invariant | `relationships come from the closed set {investigates, predicts, executes, observes, supports, challenges, influences, realizes, derives} — growth only by a schema revision` | [[kaya.model]] |
```
**Recommendation:** Carry the two updated constraint rows into the deploy step for `openspec/specs/kaya-model/spec.md` (and regenerate `docs/src/specs/kaya-model.md` via `just specs-docs`), or fix the deploy pipeline so the sidecar's constraint table is the single source. Note the same drift already exists for `capture_attribute_bound` (present in `openspec/changes/archive/2026-11-21-add-capture-attributes/specs/kaya-model/layer.md`, absent from the deployed spec) — worth fixing together.

[CLAR-003] MEDIUM — `src/model.rs:145-146`, `:209`
**Description:** The change removes a stale cardinality from the refusal message ("the closed nine-relationship set is …" → "the closed relationship set is …", `src/model.rs:245`) but reintroduces cardinality in the doc comments: "The ten admitted relationship kinds" and "The five structural relationships". These will go stale on the next growth exactly like the message did.
**Recommendation:** Use cardinality-free wording — "The admitted relationship kinds, including the Decision-to-Source attribution edge `cites`" and "The structural relationships name a specific source and target entity".

[CLAR-004] MEDIUM — `openspec/changes/add-decision-evidence-link/proposal.md` (Impact)
**Description:** The impact statement is inaccurate in the zero-evidence case (see CORR-001): "pre-relationship ledgers change by exactly the marker line" holds only when the ledger already had evidence rows; a decisions-only ledger gains a whole `### Evidence` section. The Impact section also does not mention that the deployed `openspec/specs/kaya-model/spec.md` constraint rows and the book page will still show the nine-name closed set until the deploy step runs.
**Recommendation:** Restate the impact precisely ("ledgers that already render Evidence gain exactly one marker line; ledgers with decisions but no evidence gain a new Evidence section") and add the deploy-time doc/spec obligation to the tasks list.

[CLAR-005] LOW — `src/model.rs:8-9` (module doc) and `openspec/changes/add-decision-evidence-link/specs/kaya-model/spec.md:57`
**Description:** The module doc says "direction follows a fixed grammar — including Decision-to-Source attribution via `cites`" and the delta's Direction-discipline requirement says "an Implementation Model realizes a Hypothesis" — `Implementation` is not an entity type in the closed thirteen-type vocabulary (`Model` is). The phrase is pre-existing (copied from the deployed spec), but the change re-writes that requirement text and could have corrected it.
**Recommendation:** Write "a Model realizes a Hypothesis" when the requirement is next touched.

---

## STAGE 4: EDGE CASES

[EDGE-001] MEDIUM — `src/projection/report.rs:881-901`
**Description:** Unpinned boundary states in the scoped branch:
- a **duplicate** `cites` edge (same decision → same source asserted twice) is silently collapsed by `evidence.dedup()` — correct, but unpinned;
- a decision whose **only** citation points at an incomplete Source (`path` without `commit`) renders **no** evidence row and no marker, so the decision looks evidence-free;
- a **self-cite** (`Decision → Decision`) is refused by `requires_exact_pair`, but that refusal is only reachable when both endpoint nodes already exist at edge-assertion time (`src/model.rs:370-374` skips the check when an endpoint is missing, deferring to `build()`), so the timing of the refusal differs between the two paths — a reader of the refusal message cannot tell which rule fired.
**Recommendation:** Pin the duplicate-edge and incomplete-only-citation states with fixtures; and, for the self-cite case, assert the label and message once through `build()` so the byte contract is fixed.

[EDGE-002] MEDIUM — `src/projection/report.rs:878-880`
**Description:** The regime predicate is computed by a full edge scan and then the cites branch scans all edges again (`graph.edges().any(...)` followed by a second `for edge in graph.edges()` loop, plus a third for influences). Each cites edge additionally performs two `BTreeMap` lookups (`graph.node(&edge.from)`, `graph.node(&edge.to)`) to re-derive facts the model already guarantees (`cites` is exact-pair Decision → Source). Allocation-wise this is trivial, but the redundancy is the shape that lets the two regimes drift apart.
**Recommendation:** Collect the cites edges once (single pass) into a `Vec`, derive `scoped` from `!cites_edges.is_empty()`, and drop the entity re-checks that duplicate `fixed_direction`.

[EDGE-003] LOW — `src/projection/report.rs:870-871`
**Description:** `render_decision` returns `None` only when there are no decisions; a ledger with decisions but no sources still produces a block. Combined with CORR-001 this is where the new `### Evidence` section first appears.
**Recommendation:** Covered by the CORR-001 fix; add a fixture named for the case (`graph_with_decisions_only`) so the shape is explicit.

[EDGE-004] LOW — `tests/understanding.rs:44-46`
**Description:** The `cites()` helper is written in RED-phase shape ("parsed through the existing closed-set API so this suite compiles before the model admits it; the expect is the RED failure until `cites` joins the closed set"). The model now admits `cites`, so the helper and its doc comment are stale; `Relationship::Cites` is the direct expression.
**Recommendation:** Replace with `Relationship::Cites` and delete the RED-phase comment (and the now-unneeded `use std::str::FromStr`).

---

## STAGE 5: EXCELLENCE (test quality, binding correctness, allocation)

[EXCL-001] MEDIUM — `tests/understanding.rs:48-51` vs `src/projection/report.rs:52-53`
**Description:** The staleness marker is asserted through a **second, hand-copied literal** in the test file rather than through the exported `report::ATTRIBUTION_STALE_ROW`. The bytes are pinned today (the render uses the const, the test compares to its own copy), but the exported constant — the thing downstream consumers would import — is never itself asserted to equal the rendered line, and the two copies can drift in a way the suite will not notice (edit both, the suite stays green while published bytes change). The ticket asks for a "byte-stable" marker; the byte pin belongs on the shipped constant.
**Recommendation:** Import `ATTRIBUTION_STALE_ROW` in the test and assert both (a) the rendered line equals the const and (b) the const equals one literal byte string, so there is a single literal in the repo.

[EXCL-002] MEDIUM — `.espectacular/kaya-model/source-cited-by-two-decisions-renders-one-row-per-reference.toml:11`
**Description:** Two `.espectacular` scenario ids bind to the **same** cargo test function:
```
cargo = [{flags = "decision_evidence_lists_only_sources_the_decision_cites"}]
```
in both `source-cited-by-two-decisions-renders-one-row-per-reference.toml` and `evidence-lists-only-cited-sources.toml`. The layer sidecar declares these as two distinct properties, and the deployed spec's mirror requires each property row to be verified by "exactly one **dedicated** scenario" — a shared test is not dedicated. (The shared-source behaviour *is* incidentally asserted inside that test, since `p/Source/src3` appears twice in the expected lines, so behaviour is covered; only the binding is wrong.)
**Recommendation:** Give the two-decisions property its own test function (e.g. `source_cited_by_two_decisions_renders_one_row_per_reference`) that asserts the two `src3` rows and their order, and bind the overlay to it.

[EXCL-003] MEDIUM — `.espectacular/kaya-model/` (no overlay for the closed-set naming test)
**Description:** The change adds `tests/model.rs:unknown_relationship_refusal_names_cites_in_the_closed_set` — the test that pins the acceptance item "closed-set refusal naming cites" — but ships no `.espectacular` overlay for it. The deployed `unknown-relationship-refused.toml` still binds the older `unknown_relationship_refused_naming_the_relationship_and_closed_set`, which asserts the nine legacy names and **not** `cites`. Consequently `ah check` will not fail if `cites` is dropped from `RELATIONSHIP_SET`, and the layer sidecar has no property row for that behaviour either (its `unknown_relationship_refused` property points at the legacy scenario).
**Recommendation:** Add a deployed overlay (id == slug == filename, e.g. `unknown-relationship-refusal-names-cites.toml`) bound to the new test, and add a matching layer property row.

[EXCL-004] MEDIUM — `tests/understanding.rs:158-224` (fixture) and `:617-670` (test)
**Description:** The scoped fixture gives every decision at least one citation (`d1` → `src1`, `src3`, `src5`; `d2` → `src2`, `src3`), so the two states that the scoped regime actually gets wrong — a decision with **zero** cited sources, and a mixed ledger where uncited legacy Sources still exist alongside citations — are never constructed. The forbidden-substring assertions (`"src4"`, `"src5"`, `"attribution-stale"`) are the right shape for what they check, but they cannot detect the silent-omission failure mode (a decision with no evidence row renders nothing to forbid).
**Recommendation:** Extend the fixture with a third decision that cites nothing, and with a legacy uncited complete Source alongside the cited ones; assert the expected marker/absence per decision.

[EXCL-005] LOW — `tests/model.rs:543-668`
**Description:** The four model tests are well targeted (edge admitted; closed set names `cites`; wrong pairs refused with `direction_fixed` and `Decision -> Source`; dangling refused with `references_resolve`), and the closed-set test deliberately asserts containment rather than the old cardinality phrase — good. Two gaps remain: no test asserts that the *legacy* nine-name assertion in `unknown_relationship_refused_naming_the_relationship_and_closed_set` (`tests/model.rs:115-129`) still holds together with the new name (it does, but the suite does not prove the set grew rather than being replaced), and no test pins the refusal-message byte contract for `cites` beyond substring containment.
**Recommendation:** Extend the legacy closed-set test's name list to include `cites` so growth is proven, and add one exact-message assertion for the direction refusal.

[EXCL-006] LOW — `src/model.rs:34`, `:164-175`
**Description:** `RELATIONSHIP_SET` and `Relationship::ALL` now duplicate the same ten names in two places that must be edited together (`ALL` is the parsing source, the const is the message source). The change edits both consistently — good — but the duplication is a standing drift hazard, and `Relationship::ALL` is `[Self; 10]`, a second cardinality.
**Recommendation:** Derive the message's set text from `ALL` (e.g. `Self::ALL.iter().map(|r| r.as_str()).collect::<Vec<_>>().join(", ")`) or accept the duplication with a test that asserts the const lists exactly the `ALL` names.

[EXCL-007] LOW — `src/projection/report.rs:848-866`
**Description:** The rewritten renderer doc comment is accurate about the two regimes but omits the two behaviours a reader of the code would need: that the regime is chosen per ledger (not per decision) and that influenced-finding rows stay decision-attributed in both regimes (CORR-002). It also no longer states the pre-existing rule that "emptiness is the decision records only" for the scoped case (a scoped ledger whose only decision cites nothing still renders a block).
**Recommendation:** Add one sentence per omitted rule.

---

## Convergence check (after Stage 5)

```
New CRITICAL issues: 0
Total new issues: 17 (3 HIGH, 7 MEDIUM, 4 LOW + 3 further HIGH/MEDIUM from Stages 1-2)
New issues vs Previous Stage: +29% (Stage 5 added 7)
Estimated false positive rate: ~12% (CORR-002 and EXCL-001 are wording-level judgments)
Status: CONTINUE → escalate to HUMAN only for the mixed-ledger / zero-cited-decision design decision (DRAFT-001), which is a product call, not a code call.
```

---

# Final Report

**Work Reviewed:** code + spec delta + contract bindings — kaya-q6s `add-decision-evidence-link` working tree.
**Convergence:** Stage 5 (escalated item: the mixed-ledger design decision).

## Summary

Total Issues by Severity:
- CRITICAL: 0 — must fix before proceeding
- HIGH: 3 — should fix before proceeding
- MEDIUM: 7 — consider addressing
- LOW: 4 — nice to have

## What's Working Well

1. `src/model.rs` — the relationship joins the closed set cleanly and consistently: enum variant, `ALL` (now 10), `as_str`, `fixed_direction` → `(Decision, Source)`, `requires_exact_pair`, `RELATIONSHIP_SET`, and the refusal message all moved together — `src/model.rs:34,159,174,189,202,217,245`.
2. `src/model.rs:245` — dropping the hard-coded "nine" from the refusal message is exactly the right fix for a set that must grow; the old phrasing was a standing staleness bug.
3. `src/projection/report.rs:878-901` — the scoped branch keeps the existing `(id, key, line)` + `sort` + `dedup` discipline, so "one row per citing decision, sorted by source id then decision id" falls out of the existing machinery rather than a new sort.
4. `tests/understanding.rs:617-670` — the scoped test asserts the whole block byte-for-byte *and* asserts forbidden substrings (`src4`, `src5`, `attribution-stale`); that pairing is what makes the scoped/fallback distinction checkable.
5. Delta format matches the `add-capture-attributes` archive layout exactly (`proposal.md`, `tasks.md`, `specs/kaya-model/{spec.md,layer.md}`), and all six `.espectacular` overlays exist in both the change dir and the deployed dir with byte-identical content.

## Top 3 Critical Findings (highest severity — no CRITICAL-severity defects found)

1. [DRAFT-001] HIGH — `src/projection/report.rs:878-901` — the attribution regime is chosen per ledger but the evidence it produces is per decision, so a mixed ledger, a decision with zero cited sources, and a decision whose only citation is incomplete all silently under-report (rows *and* source-map entries vanish, with no marker). Fix: per-decision fallback/marker, plus three scenarios and three fixtures.

2. [CORR-001] HIGH — `src/projection/report.rs:941-948` — the marker is appended unconditionally, so a decisions-only ledger gains a brand-new `### Evidence` section whose only content claims "evidence is ledger-global" about evidence that does not exist, contradicting the change's own "exactly the marker line" impact claim. Fix: emit the marker only when the Evidence section has content (or move it under `### Decisions`), and pin the bytes with a fixture.

3. [CLAR-001] HIGH — delta `specs/kaya-model/spec.md` vs deployed `openspec/specs/kaya-model/spec.md:114-117` — the deployed spec's "Every property row in the layer sidecar SHALL be verified by exactly one dedicated scenario" is false after deploy: the sidecar gains six property rows and the delta adds no mirror scenarios and no `**VERIFIES**` lines for the three projection-side properties. Fix: MODIFIED the mirror requirement with one scenario + `**VERIFIES**` per new property row.

## Stage-by-Stage Quality

- Stage 1 (Draft): GOOD — one coherent slice, one switch, one marker; the flaw is state granularity, not architecture.
- Stage 2 (Correctness): GOOD — every stated scenario is implemented correctly and pinned; the surrounding states are the problem.
- Stage 3 (Clarity): FAIR — the delta is well written and under 500 chars per requirement (375 / 329 / 429 / 482), but the deployed constraint rows and the property-coverage mirror are left behind.
- Stage 4 (Edge Cases): FAIR — duplicate edges, incomplete-only citations, self-cite timing, and the decisions-only ledger are all unpinned.
- Stage 5 (Excellence): GOOD — bindings are present and deployed, but two scenarios share one test, the new closed-set test has no overlay, and the marker literal is duplicated instead of imported.

## Recommended Actions

1. Decide the mixed-ledger semantics (per-decision fallback vs. ledger-global with a per-decision marker), write it into the delta as scenarios, and extend `graph_with_scoped_decision_evidence` with a zero-citation decision and a legacy uncited source.
2. Gate the staleness marker on a non-empty Evidence section (or re-home it under `### Decisions`), and pin the decisions-only ledger's bytes with a fixture.
3. MODIFIED the "Property coverage mirror" requirement in the delta with one `**VERIFIES**` scenario per new layer property row.
4. Update the deployed `openspec/specs/kaya-model/spec.md` constraint rows (`relationship_set_closed`, `direction_fixed`) — and `docs/src/specs/kaya-model.md` via `just specs-docs` — so the deployed capability and book stop advertising a nine-name closed set.
5. Give `source_cited_by_two_decisions_renders_one_row_per_reference` its own test and overlay; add an overlay + layer property for `unknown_relationship_refusal_names_cites_in_the_closed_set`; import `ATTRIBUTION_STALE_ROW` in the test instead of hand-copying the literal.
6. Narrow the marker wording to source attribution (finding rows stay decision-attributed in both regimes), and drop the reintroduced cardinalities ("ten", "five") from the model doc comments.

## Verdict

**NEEDS_REVISION**

**critical_count: 0** (HIGH 3, MEDIUM 7, LOW 4; 3 further HIGH/MEDIUM items found in Stages 1-2 and 5 are folded into the counts above)

**Rationale:** The core mechanism is correct and every stated scenario is honestly tested, so nothing here is fatal; but the change ships a ledger-global regime whose uncovered states (mixed ledger, zero-cited decision, decisions-only ledger) silently under-report evidence in published documents, and it leaves the deployed constraint rows and the property-coverage mirror behind. Fixing the three HIGH items — per-decision attribution semantics, marker gating, and the mirror/constraint-row update — plus the binding corrections in items 5-6 is enough to ship this slice.
