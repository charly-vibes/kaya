---
tags: [pipeline-run:tdd-ro5u-2026-11-22-kaya-d0n-agent-session-capture-proposal, pipeline-step:refactor]
---

# REFACTOR kaya-d0n — agent-session capture proposal

Run: tdd-ro5u-2026-11-22-kaya-d0n-agent-session-capture-proposal (step 5/9).

## Tidy pass over the proposal
- Section order moved to Decision → Why → What Changes → Impact so the
  decision is read before the rationale, and the rationale before the
  commitments (the acceptance names the decision first).
- The four unlock conditions for the raw surface are stated once, in the
  Decision section, and referenced from Impact instead of being repeated.
- Each of the five coverage areas carries exactly one commitment block, so
  the acceptance can be checked area by area without reading across
  sections.
- Terms that could be read two ways are pinned: "bounded attribute" (the
  80-character title plus the 16-hex digest, and nothing else), "dedup key"
  (sha256 over source bytes digest plus entry identity), "actor typing"
  (`ActorType::Agent` vs today's `ActorType::Tool`).
- The proposal states the falsification test for the choice — three
  questions a reader must be able to answer from a distilled record — so the
  follow-up ticket can measure it instead of assuming.

## No code refactor
Nothing under `src/` changed in this ticket; the tidy is confined to the
proposal text.

## Verification
`openspec validate --changes --strict` → pass; `cargo test` → 31/31 ok;
`git diff --stat src/` → empty.

