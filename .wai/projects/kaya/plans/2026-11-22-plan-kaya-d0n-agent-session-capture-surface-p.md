---
tags: [pipeline-run:tdd-ro5u-2026-11-22-kaya-d0n-agent-session-capture-proposal, pipeline-step:plan]
---

# PLAN kaya-d0n — Agent-session capture surface: proposal

Run: tdd-ro5u-2026-11-22-kaya-d0n-agent-session-capture-proposal.

## Decision
**Distilled.** Agent sessions enter kaya as distilled turu whisper bundles
observed through the existing turu adapter (`capture.entry.observed`), with
bounded derived attributes only. Raw transcripts (pi/terapia session
transcripts) are rejected for now and recorded with the conditions that
would unlock them.

## Rationale
- The ownership rule is total: `src/adapters/turu.rs:41` and the wai adapter
  header — no adapter carries content. A raw surface would move the boundary,
  which needs its own spec decision (as kaya-q5e did) and its own redaction
  guarantee before any text could be copied.
- The distilled bundle already carries the session's structure: entry id, ts,
  topic, scope, plus a bundle digest — everything a reader needs to know
  WHICH session said WHAT, without the transcript.
- The external evaluation called agent sessions the richest available source;
  the distilled surface is the only one that can capture that richness while
  the ownership rule holds.

## Alternatives considered
1. Raw transcripts as `capture.entry` events — rejected: breaks the
   pointers-only rule; needs a new ownership decision, a proven redaction
   layer, size bounds over transcript bytes, and dedup by transcript hash.
2. Neither (defer agent sessions entirely) — rejected: the distilled surface
   is already shipped by the turu adapter; deferring only postpones the
   decision the ticket asks for.
3. Distilled with content-bearing attributes (title + digest) — chosen, with
   the redaction rule that makes bounded attributes safe.

## The five required coverage areas → design commitments
1. **Secret redaction** — a concrete pattern list (in the What Changes
   section) applies to every bounded attribute before it is copied; each
   pattern needs a positive and a negative fixture.
2. **Transcript size bounds** — bounded attributes only: title ≤ 80 chars,
   digest = 16 hex; no transcript body is read into an event; per-import
   entry cap with an atomic refusal beyond it.
3. **Incremental import dedup by content hash** — the dedup key is the
   sha256 over (bundle bytes digest, entry id, topic, ts); because the event
   id is derived from that content, re-importing an unchanged session yields
   the same ids and no new events.
4. **Event kind + actor typing** — kind `capture.entry.observed` (already in
   the closed capture vocabulary), subject `<project>/Source/entry-<entry id>`,
   actor type `Agent` for agent-session captures (today's turu adapter emits
   `Tool`; the switch is a follow-up ticket), actor name = the session tool.
5. **Hooks for MVP-2 relationship linking** — the entry id + topic + ts are
   the join key; the `cites` relationship (shipped by kaya-q6s) lets a
   Decision cite the session Source, and Findings can be attached through
   the influence edge.

## Change shape
Proposal-only openspec change `add-agent-session-capture` with
`.openspec.yaml` (`schema: spec-driven`, `skip_specs: true`) — no spec
deltas, because this ticket decides a surface, it does not change behavior.
Validated with `openspec validate --changes --strict`.

## Verification the orchestrator runs
- `openspec validate --changes --strict` → pass.
- `cargo test` (unchanged code, must stay green), `cargo clippy
  --all-targets -- -D warnings` → clean.
- `git diff --stat src/` → empty (no production code).
- Rule-of-5 review of the proposal: CRITICAL 0.

## Risks
- The distilled surface may disappoint the evaluation's "richest source"
  expectation; the follow-up ticket must measure what a stranger can learn
  from a distilled session record before MVP-2 generation work starts.
- The redaction pattern list is unproven against real transcripts; the
  follow-up implementation ticket must validate it with fixtures drawn from
  real session data before it can be trusted.

