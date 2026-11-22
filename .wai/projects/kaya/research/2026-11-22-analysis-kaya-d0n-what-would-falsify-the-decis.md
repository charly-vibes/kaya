---
tags: [pipeline-run:tdd-ro5u-2026-11-22-kaya-d0n-agent-session-capture-proposal, pipeline-step:red]
---

# Analysis kaya-d0n — what would falsify the decision

Run: tdd-ro5u-2026-11-22-kaya-d0n-agent-session-capture-proposal.

## The decision under test
Distilled: agent sessions enter kaya as turu whisper bundles observed through
the existing turu adapter, with bounded derived attributes; raw transcripts
are rejected.

## Evidence that would falsify "distilled is enough"
1. A reader with only the distilled record (entry id, ts, topic, scope,
   bundle digest) cannot answer a question that a raw transcript would
   answer. Test: pick three questions the external evaluation implied
   (what was decided, why, what was tried) and check the distilled record
   against them. If two of three fail, the distilled surface is too thin and
   the raw surface needs a design decision after all.
2. The bounded attributes cannot be made safe: if real session transcripts
   cannot be redacted reliably (a pattern list that misses secrets in real
   data), then the distilled surface is not merely thin but unsafe, and the
   raw surface is worse. Test: run the proposed pattern list over a corpus of
   real transcripts and count misses.
3. The turu bundle surface cannot express agent-session content at all
   (entries carry only id/ts/topic): if session material does not flow
   through whisper bundles, "distilled" has no carrier.

## Risks carried by the chosen surface
- Thin evidence: a distilled session may record that something happened
  without why — the kaya-q6s `cites` relationship helps attribution but not
  content.
- The redaction list is a starting set; a false negative leaks a secret into
  a bounded attribute.
- The actor typing change (Tool → Agent) touches a shipped adapter's event
  bytes; it must be a deliberate, spec-backed ticket, not a drive-by.

## Conditions that would unlock the raw surface
1. A spec delta naming the session store (pi/terapia) as a text owner,
   parallel to wai and turu — i.e. the ownership rule gains a third owner
   whose text kaya may never copy.
2. A redaction layer proven by tests over real transcripts, with the pattern
   list as its specification.
3. Size bounds and dedup defined over transcript bytes, not over distilled
   entries.
4. A new event kind in the closed capture vocabulary (for example
   `capture.session.transcript`) carrying a bounded excerpt, never the whole.

## Acceptance checks the orchestrator runs
- `openspec validate --changes --strict` → pass.
- `git diff --stat src/` → empty.
- `cargo test` → green (no behavior change).
- Rule-of-5 review → CRITICAL 0.

