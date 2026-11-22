---
tags: [pipeline-run:tdd-ro5u-2026-11-22-kaya-d0n-agent-session-capture-proposal, pipeline-step:green]
---

# GREEN kaya-d0n — agent-session capture proposal

Run: tdd-ro5u-2026-11-22-kaya-d0n-agent-session-capture-proposal (step 4/9 green).

## Delivered
`openspec/changes/add-agent-session-capture/` — a proposal-only openspec
change:
- `proposal.md` with exactly four sections: Decision, Why, What Changes,
  Impact.
- `tasks.md` — the research steps done here and the follow-up tickets to
  file (left as "to be filed"; no beads issues created).
- `.openspec.yaml` — `schema: spec-driven`, `skip_specs: true` (no spec
  deltas: this ticket decides a surface, it does not change behavior).

## The decision (design)
Distilled. Agent sessions enter kaya as turu whisper bundles observed through
the shipped turu adapter (`capture.entry.observed`), carrying bounded derived
attributes only; raw transcripts are rejected with four explicit unlock
conditions (a spec delta naming a session-store owner, a proven redaction
layer, size bounds and dedup over transcript bytes, a new bounded-excerpt
event kind).

## The five coverage areas as committed design
1. Secret redaction — a concrete starting pattern list (OpenAI/Anthropic/
   GitHub/AWS keys, Slack tokens, JWTs, PEM private keys, and a
   `key/secret/password/token = value` shape), applied to every bounded
   attribute before copying; redaction replaces with a placeholder, never
   truncates silently; one positive and one negative fixture per pattern.
2. Transcript size bounds — 80-char derived titles cut on a character
   boundary, 16-hex digests, no transcript body read into an event, and an
   atomic refusal beyond 500 entries per import.
3. Incremental import dedup by content hash — sha256 over the source bytes
   digest plus entry identity (id, topic, ts); the event id is derived from
   that content, so re-importing an unchanged session yields the same ids
   and no new events.
4. Event kind and actor typing — kind `capture.entry.observed` (already in
   the closed capture vocabulary), subject `<project>/Source/entry-<entry id>`,
   actor type `Agent` for agent sessions (today's turu adapter emits `Tool`),
   actor name naming the session tool; the switch is a follow-up ticket
   because it changes shipped event bytes.
5. MVP-2 relationship linking hooks — entry id, topic, ts and scope are the
   join key; with kaya-q6s's `cites` relationship a Decision can cite the
   session Source, and Findings attach through the influence edge.

## Verification (orchestrator)
- `openspec validate --changes --strict` → pass (skip_specs accepted, zero
  deltas).
- `git diff --stat src/` → empty: no production code.
- `cargo test` → 31/31 suites ok; `cargo clippy --all-targets -- -D warnings`
  → clean; deployed `ah check` → 0 findings.

