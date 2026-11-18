---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-n50-overview-cutover, pipeline-step:orient]
---

ORIENT kaya-n50 — overview cutover (approved scope: overview half only)

Ticket: kaya-n50, P2, complexity:m, lane:hitl, files ["README.md","docs/src/index.md","docs/src/document-taxonomy.md"]. User-approved scope: cut over the overview half now; leave concept hand-written and record the blocker.

Acceptance restated as checkable statements:
1. The compiled overview product answers "what is this and why does it matter" from generated content plus authored interpretation: docs/public/overview.md carries a kaya:authored block (what/why/when) and its generated publication block.
2. The hand-written overview prose duplicated elsewhere is removed or reduced: README stops restating the idea section; docs/src/index.md keeps only its long-form rationale and points at the compiled overview; no sentence about what kaya is appears in two files.
3. `kaya publish` preserves the authored block across rebuilds and stays idempotent (second run recomputed []); the mdbook still builds.
4. Concept remains hand-written; the blocker (no Finding/Decision records in the committed ledger) is recorded on the ticket.

State found:
- docs/public/overview.md is 10 lines: the managed publication block only (reader question + a cross-reference to getting-started, the fact-once owner of orientation.project-overview). No authored interpretation.
- The generated orientation body lives in getting-started and holds only the Question row; the ledger has no Finding/Decision, so concept cannot be generated (confirmed by ledger census: Question 1, Hypothesis 1, Experiment 1, Model 1, Source 6).
- Duplication measured by phrase: "grows out of the work" appears in README.md and docs/src/index.md; "append-only" in README (4), index.md (2), status.md (2); README carries the full what/why and an event-format example that index.md also explains under "Documentation compilation".
- README, index.md and document-taxonomy.md are not compiled by kaya, so kaya:authored markers only take effect in the compiled overview page.

Risks: README is the GitHub landing page, so the cutover must keep install/status/FAQ usable while removing the duplicated idea section; links from README/index to the compiled page must resolve in the repo; mdbook must still build with a link from docs/src to docs/public.
