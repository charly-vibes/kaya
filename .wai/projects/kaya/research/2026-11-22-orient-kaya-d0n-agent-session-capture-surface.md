---
tags: [pipeline-run:tdd-ro5u-2026-11-22-kaya-d0n-agent-session-capture-proposal, pipeline-step:orient]
---

# ORIENT kaya-d0n — Agent-session capture surface: proposal

Run: tdd-ro5u-2026-11-22-kaya-d0n-agent-session-capture-proposal.

## Ticket
- id kaya-d0n, P2, lane design, complexity m, files `openspec/changes`.
- Type spike; no production code. Blocks nothing (kaya-3tm is blocked by
  9b1+q6s+q5e only).
- Base commit fbec965.

## Acceptance restated as testable statements
1. An openspec proposal exists that states the distilled-vs-raw decision
   plainly, with reasoning.
2. The proposal carries exactly four sections: Decision, Why, What Changes,
   Impact.
3. The proposal covers all five required areas concretely: secret redaction
   (pattern list + the tests it needs), transcript size bounds, incremental
   import dedup by content hash, event kind + actor typing, and hooks for
   MVP-2 relationship linking.
4. `openspec validate --changes --strict` passes.
5. No production code changes: no file under `src/` is touched.

## Current capture surface facts (quoted from the tree)
- `src/adapters/turu.rs:41` — "Entry text is never copied into an event."
  The turu adapter reads one bundle once, emits one `capture.entry.observed`
  event per bundle entry (`src/adapters/turu.rs:28`), subject
  `<project>/Source/entry-<entry id>` (`src/adapters/turu.rs:140`), actor
  `ActorType::Tool` / name `turu` (`src/adapters/turu.rs:136`), and data
  `{entry, topic, scope, repo_key, bundle_digest}` (`src/adapters/turu.rs:127`).
- `src/adapters/wai.rs:1-8` — ownership rationale: "bounded content-derived
  attribute != text ownership; wai keeps full text". The wai adapter copies
  only a bounded title (80 chars) plus a 16-hex digest, and the event id
  hashes the path-identity payload only, so content never moves the id.
- `src/model.rs:31` — `ENTRY_OBSERVED_EVENT = "capture.entry.observed"` is
  part of the closed capture event vocabulary (`src/model.rs:25`,
  `src/model.rs:417-418`).
- `src/event.rs:108-115` — `ActorType` is a closed set: `Tool`, `Human`,
  `Agent`.
- `src/model.rs` — the entity vocabulary is closed (13 kinds) and the
  relationship set is closed (`cites` added by kaya-q6s); a Decision can
  already cite a Source, which is the linking hook a session capture needs.

## The constraint that decides the surface
A Rule-of-5 review (recorded in kaya-q5e's orientation) confirmed the
pointers-only rule is total: no adapter carries content, and the only
content kaya ever copies is a bounded, content-derived attribute whose
owner keeps the full text. A raw session transcript would be a
DESIGN-BOUNDARY change, not a tweak — exactly the classification the
external evaluation gave kaya-q5e.

## Uncertainties
- Whether a future session store (pi/terapia) can be named as a text owner in
  a spec delta without changing the wai/turu rationale (follow-up ticket,
  not this one).
- The redaction pattern list is a starting set to be validated against real
  transcripts by the follow-up implementation ticket.
- The external evaluation's "agent sessions are the richest available source"
  is an unverified claim; this proposal treats it as motivation, not evidence.

