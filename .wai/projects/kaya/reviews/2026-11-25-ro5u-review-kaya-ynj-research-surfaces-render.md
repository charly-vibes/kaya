---
reviews: 2026-11-25-green-kaya-ynj-subagent-landed-three-seams-1.md
verdict: pass
tags: [pipeline-run:tdd-ro5u-2026-11-25-kaya-ynj-research-surfaces-render-information-not-references, pipeline-step:ro5u-review]
---

# RO5U Review — kaya-ynj (research surfaces render information not references)

Verdict: READY (after fix-review) · FINAL: CRITICALS=0 · Converged stage 5 · Reviewer subagent on uncommitted working tree.

## Scope reviewed

6-file diff: src/model.rs (entity_from_event flat text/title admission), src/projection/mod.rs (title_or_key title→text→key), src/projection/index.rs (relationship endpoint labels), tests/research_map.rs (3 new red→green tests + 6 regenerated edge fixtures), tests/acceptance.rs (COMMITTED_EDGES re-pins from marker), tests/projection_index.rs (2 labeled-row pins).

## Per-dimension findings

| Dimension | Finding | Severity | Disposition |
|---|---|---|---|
| Correctness/spec | scenarios semantic (kaya-documents spec.md:95-115); endpoints+type on row, isolated listed once, no invented edges; graph.node miss keeps bare id; attribute() flattens newlines | — | none |
| Invalid states | non-string/blank ignored; explicit data.attributes wins (contains_key guard); admission only in ENTITY_CREATED arm — adapter whitelist untouched, "translate, never interpret" holds | — | none |
| Error handling | no new error paths; lenient admission matches codebase convention | — | none |
| Boundaries | **A (P1):** topic-map back-parser (collection.rs:113-130 split_once) kept label prose in topic ids — confirmed live in docs/public/topic-map.md; ` -[` inside a label could cut mid-label. **B (P2):** node_ids tokenizer injected `kaya/…` tokens from label prose into event_ids (phantom provenance in knowledge-index + export) | P1, P2 | **both FIXED in fix-review:** collection.rs label-stripping + labeled fixture; source.rs MAP_KIND-gated strip_paren_groups + census regression test |
| Allocation/IO | two BTreeMap lookups + one format! per endpoint, O(edges); labels pure functions of graph, byte-identical rebuilds preserved | — | none |
| Test quality | 3 new tests drive the real path (model::build → index::build → bytes), pin behavior not implementation; precedence pinned by negative assertion; key-fallback labels pinned via regenerated fixtures; gap (no e2e labeled map through composer) closed by Finding A/B tests | — | none |

## Fix-review verification (orchestrator)

Full suite 510 passed / 0 failed · clippy 0 warnings · fmt clean · real-tree `kaya build` + `publish` exit 0 · docs/public/topic-map.md rows bare-id (`- kaya/Model/M-001 — Model — realizes`), zero label prose in published ids.

FINAL: CRITICALS=0

