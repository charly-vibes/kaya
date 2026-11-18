---
reviews: 2026-11-18-green-kaya-n50-overview-cutover-docs-public-ove.md
verdict: pass
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-n50-overview-cutover, pipeline-step:ro5u-review]
---

RO5U kaya-n50 — Rule-of-5 review of the overview cutover

Artifact under review: docs/public/overview.md (authored block), README.md, docs/src/index.md, docs/src/document-taxonomy.md, docs/src/status.md.
TYPESAFE_API_KEY is not set: findings are UNVERIFIED (self-reported).

STAGE 1 — DRAFT
The cutover is editorial: one canonical overview page (compiled, authored interpretation plus the managed publication block) and three repository documents reduced to pointers and their unique material. README keeps a usable landing page; the book keeps its long-form rationale. Shape Quality: GOOD.

STAGE 2 — CORRECTNESS
CORR-001 LOW — README.md opening
The one-line purpose ("A living record of exploration — kaya turns the artifacts a project already produces into continuously generated, human-owned documentation") restates the first sentence of the overview page in different punctuation, so a reader can see the same claim twice.
Evidence: normalized sentence comparison finds no verbatim duplicate (the check counts 0), but the semantic copy is visible side by side.
Impact: acceptable for a GitHub landing page; the full explanation exists only once.
No broken links: every relative link in the five files resolves (checked after the edits).

CORR-002 LOW — docs/src/status.md
The taxonomy row was corrected from "eight record kinds" to the eleven implemented kinds and now links the deployed specs. This is a factual correction outside the ticket's file list, made because the row's link broke when the change was archived; the new text matches src/projection/report.rs Kind::ALL.
Correctness Quality: GOOD.

STAGE 3 — CLARITY
CLAR-001 LOW — docs/public/overview.md
The authored block ends with a status line linking the book's motivation page; a reader of the published tree may also want the getting-started page. The generated block already links getting-started, so adding a second pointer would duplicate navigation.
No change recommended.
Clarity Quality: GOOD.

STAGE 4 — EDGE CASES
EDGE-001 LOW — the duplication check is a throwaway script
The meter is editorial; no repo gate enforces "one copy". A future edit can reintroduce duplication silently.
Evidence: the check lives in the run notes, not in just/CI.
Fix deferred: a durable link/duplication gate would need an allowlist of legitimate overlaps and is a different ticket; the review records the check output instead.

EDGE-002 LOW — authored bytes on a generated page
If kaya publish ever rewrites the whole page instead of the managed block, the authored overview would vanish. Evidence: coverage exists in tests/publications.rs::build_writes_one_managed_block_and_preserves_surrounding_bytes and was reconfirmed by rebuilding twice with byte-identical output.
No action.

Edge Case Coverage: GOOD.

STAGE 5 — EXCELLENCE
EXCL-001 LOW — README's FAQ still points at "Sources and authority" in index.md, which remains and is still the design-owned content; no duplication introduced.
Convergence (after stage 4): new CRITICAL 0; new issues all LOW and falling; false positives 0. CONVERGED.

Final: CRITICAL 0, HIGH 0, MEDIUM 0, LOW 4.
Verdict: READY.

