# Post-bootstrap alignment: consumers, MVP-1 acceptance, fleet-standard adoption, promotion budget

Decisions from the grill-me alignment interview (2026-11-16, post-bootstrap,
pre-MVP-1). Resolves the "what do we actually want from kaya" layer that the
origin conversation and add-core-research-record left implicit.

1. Primary consumer — agents are the everyday consumers of kaya's output;
   humans are the authority and the terminal audience. Tie-break rule when a
   projection format must choose: agent parseability, never at the cost of
   human readability. Rationale: the fleet's session loop (recall / orient /
   resume) is agent-driven; human reads are occasional trust-and-approve
   moments. All artifacts stay plain Markdown + JSONL so one artifact serves
   both.

2. MVP-1 acceptance is a three-question dossier test on kaya itself
   (dogfood). Running MVP-1 kaya over this repo must produce a research
   index plus one experiment report (add-core-research-record) in which each
   of these is answerable from the dossier alone, with provenance, without
   reading raw sources:
   (1) Why are events not genesis envelopes?
   (2) What evidence supports "adapters translate, never interpret"?
   (3) What is deliberately unspecified (MVP-2+ seams)?
   The dulce-de-leche tools (wai, dont, whisper/turu, specodelic, ddl) are
   the second-example validation targets as their adapters land.

3. Provenance contract for MVP-1 — repo files are cited as Source entities
   pinned to commit (path@commit); no layer-level adapter is required in
   MVP-1. The Source schema reserves a namespaced id field (e.g.
   specodelic:kaya-promotion.spec#R2) so file-path citations upgrade to
   layer-level resolution without re-migration.

4. Adoption — kaya is fleet-standard from day one: ddl init wires the
   `.kaya/` ledger plus the `kaya status` replay-integrity gate (the
   kaya-lb2 skeleton), and nothing else until MVP-2 adapters exist.
   Consequence: MVP-2's acceptance is "capture stays alive under the
   mandate", not "earn standard status". kaya-dgz (ddl registry PR) is
   load-bearing for the MVP-1 ship path and was bumped P3 → P2.

5. Promotion budget — pull-based approval at point of use. Proposals never
   interrupt; they surface on dossier build/read and as a count in
   `kaya status`. Approval is one cheap command (`kaya promote <id>`, under
   10 seconds), executed at existing session-close moments (wai close /
   wrap-up) or when a load-bearing answer cites a `proposed` assertion.
   Unapproved proposals stay visibly `proposed`; they never expire and never
   masquerade as settled. Design constraint on kaya.promotion: pending
   proposals must fit one session-close; if they routinely exceed it,
   proposal generation gets ranked/capped — the human is never pushed to
   approve more. Spec scenarios should assert this rate cap.

Verification evidence anchor (closes a capture gap found during the
interview): the TypeSafe-verified Rule-of-5 docs review — 3 HIGH findings,
scores 0.92–0.96, false positives 0/3 — previously existed only as commit
f39a1fd's message; recorded here as wai-side evidence supporting acceptance
question (2).

