---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-lh1-reuse-families-technical-reference-knowledge-brief, pipeline-step:orient]
---

ORIENT kaya-lh1 — Reuse families: technical-reference and knowledge-brief

Ticket metadata: kaya-lh1, P3, complexity:m, lane:afk, base_commit df816560, files ["src/projection/reuse.rs"]. Claimed this session.

Acceptance restated as testable statements:
1. `reuse-handoff.technical-reference` renders one row per specodelic spec id, shaped `- <node id> — <spec id>`, so the spec id is present in the document.
2. `reuse-handoff.knowledge-brief` renders one row per turu/whisper entry id, shaped `- <node id> — <entry id>`, so every rendered item names its entry id.
3. Building either kind writes markdown only: the ledger file is byte-identical and no ledger entity is created.
4. A kind with no source rows is omitted exactly once with reason no-records, writes no file and no placeholder; a source disappearing removes its block (and deletes a document left blank).
5. Block names are the closed vocabulary names (`reuse-handoff.technical-reference`, `reuse-handoff.knowledge-brief`), and each document carries exactly one family-scoped block.
6. Meter: `cargo test --test reuse_adapters` green and `openspec validate --changes --strict` (2 changes) passes.

Scope decision (user-approved 2026-11-17): deliver both kinds at the projection layer from fixture Source references; kaya-syx and kaya-24c stay deferred for ledger ingestion, so their dependency edges were removed from kaya-lh1. End-to-end adapter ingestion is unchanged and still owned by those tickets.

State found:
- `Kind::TechnicalReference` and `Kind::KnowledgeBrief` already exist in the closed vocabulary (src/projection/report.rs:123-124, block names at 169-171); publication admission already lists both (src/publication/mod.rs:80-82). No vocabulary work is needed.
- src/projection/reuse.rs implements only reproduction-package and handoff: `Inputs` carries revisions/commands/handoffs, `documents()` returns two files, `build()` destructures exactly two paths, `inputs()` selects Source.path+commit, Run.command(s), Artifact.kind=handoffs.
- Callers: src/main.rs:533-539 loops `reuse::documents(out)` for stale-authored flagging, so document ordering must stay canonical.

Attribute convention chosen (no prior vocabulary pinned it): a `Source` node with a non-blank `spec` attribute is a specodelic reference; a `Source` node with a non-blank `entry` attribute is a turu/whisper entry reference. Absent/blank contributes no row; a present non-string is refused with `kaya.projection.reuse_attribute` naming node and attribute, matching the existing reuse rules. Rows carry ids only — no payload copy, no entity creation.

Risks: deterministic document and row order (canonical kind order technical-reference, reproduction-package, knowledge-brief, handoff); omission reporting once per kind; existing tests/reuse.rs must keep passing unchanged.
