---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-n50-overview-cutover, pipeline-step:green]
---

GREEN kaya-n50 — overview cutover

docs/public/overview.md now opens with a kaya:authored block that answers the overview reader question: what kaya is (a living record of exploration compiled from tool artifacts), why it exists (documentation drifts when written away from the work), when to use it, what it is not (consumer/integrator of wai, dont, whisper/turu, specodelic), and its status with a link to the long-form rationale. The managed publication block follows unchanged, and `kaya publish` preserves the authored bytes (verified byte-identical across rebuilds).

README: the duplicated what/why blockquote, the "When to use" section and the 28-line "The idea" section are gone. The landing page keeps the quote, badges, a one-line purpose, status/CLI/install/FAQ/development, and points at the compiled overview and the book. The duplicated CLI envelope sentence now lives only in status.md.

docs/src/index.md: opens with a pointer to the compiled overview and keeps the long-form motivation, architecture, decisions, milestones and provenance.

docs/src/document-taxonomy.md: the contract link now points at the deployed kaya-documents and kaya-publications specs (the change proposal moved to the archive), and the page names the compiled overview as the reader-facing summary.

docs/src/status.md: the taxonomy row links the deployed specs and no longer claims eight record kinds — it lists the eleven implemented kinds including technical-reference and knowledge-brief.

Concept stays hand-written: recorded on the ticket that the ledger has no Finding/Decision/Observation/Claim/Run entities, so understanding kinds cannot back the concept product.

Verification: normalized sentence-duplication sweep across README, index, document-taxonomy, status and the overview page = 0 duplicate sentences (was 1 plus the discarded idea section); grep -c kaya:authored docs/public/overview.md = 1; relative links in all five files resolve; kaya publish twice recomputed [] with byte-identical overview pages; kaya build recomputed []; mdbook build green; just ci green.

