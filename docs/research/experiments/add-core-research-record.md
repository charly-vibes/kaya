# Experiment report: add-core-research-record

<!-- kaya:authored -->
Experiment: `kaya/Experiment/add-core-research-record`
Question: `kaya/Question/mvp-1-acceptance`

This dossier answers the three MVP-1 acceptance questions from the ledger and
the committed sources it cites. The generated `experiment-report` block below
is compiled by `kaya build`; the answers here are human-owned and survive
every rebuild byte for byte.

### Q1: Why are events not genesis envelopes?

`kaya.event/v0` is the ledger record schema — a closed field set with
provenance, a namespaced type, and no interpretation field — while CLI
command I/O keeps the genesis envelope conventions (`ok`,
`envelope_version`, `envelope_kind`, `data`). Proposal decision 1, "Events
are not genesis envelopes," states the split in
`openspec/changes/add-core-research-record/proposal.md`. The event layer
declares the same seam as an explicit extension point:
`envelope_independence` in
`openspec/changes/add-core-research-record/specs/kaya-event/layer.md` says
records are ledger payloads distinct from CLI envelopes and that the seam is
revisited after the first end-to-end dossier.

### Q2: What evidence supports "adapters translate, never interpret"?

The adapter contract states the rule directly: the requirement "Translate,
never interpret" in
`openspec/changes/add-core-research-record/specs/kaya-adapters/spec.md`
makes adapters emit observations (a decision exists, a run completed, a
bundle was packed) and forbids conclusions, with atomic failure and
per-source provenance. Independent review evidence is recorded in the wai
design artifact
`.wai/projects/kaya/designs/2026-11-16-post-bootstrap-alignment-consumers-mvp-1-accep.md`:
its "Verification evidence anchor" records the TypeSafe-verified Rule-of-5
docs review — 3 HIGH findings, scores 0.92–0.96, false positives 0/3 — first
recorded only in commit `f39a1fd`. Interpretation stays in `kaya.promotion`,
where a human approval is required.

### Q3: What is deliberately unspecified (MVP-2+ seams)?

The section "Deliberately not built" in `docs/src/index.md` excludes another
workflow manager, a general graph database, automatic paper generation, a
universal event bus, automatic conclusions, a proprietary documentation
format, and a complex web application. Design decision 4 in the same file
leaves connect-later semantics — cross-workspace merge and conflict
resolution — deliberately unspecified until a second workspace exists, even
though experiment ids are namespace-qualified from day one. Proposal decision
3 in `openspec/changes/add-core-research-record/proposal.md` keeps epistemic
category changes with `kaya.promotion` alone, so MVP-1 capture remains
observation-only; the milestone list in `docs/src/index.md` assigns continuous
capture, deduplication, and relationship linking to MVP 2 and exploration and
dossier export to MVP 3.
<!-- kaya:end authored -->