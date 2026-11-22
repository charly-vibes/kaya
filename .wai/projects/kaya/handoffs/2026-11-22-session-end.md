---
date: 2026-11-22
project: kaya
phase: research
---

# Session Handoff

## What Was Done

Plan v2's implementation queue, four tickets shipped through the
`tdd-ro5u` pipeline (each: orient → plan → red → green → refactor →
Rule-of-5 review → fix → quality ledger → human-approved ship-close), all
committed and pushed to `origin/main` (HEAD `6b7dfd2`):

- **kaya-9b1** (chore, P4): regenerated the stale book spec page
  (`docs/src/specs/kaya-ledger.md`, the deployed `single_writer_lock` row),
  rebuilt the book, closed the ticket.
- **kaya-q5e** (P2, afk/l): content-derived capture attributes — the wai
  adapter now copies a bounded first-heading title (≤ 80 chars, char-boundary
  safe) plus a 16-hex digest into the event's open `data` map; the event id
  still hashes the path-identity payload only, so content never moves the id.
  Deployed as `add-capture-attributes`; the ownership decision ("bounded
  content-derived attribute != text ownership; wai keeps full text") is
  recorded verbatim in the kaya-adapters delta.
- **kaya-q6s** (P3, afk/m): `cites` joined the closed relationship set
  (Decision → Source, exact pair) and `understanding.decision-record` now
  attributes evidence **per decision** — a citing decision scopes its rows to
  the Sources it cites; a decision citing none keeps ledger-global evidence
  with a staleness marker naming it; a record with no complete Source
  renders no Evidence section. Deployed as `add-decision-evidence-link`.
- **kaya-d0n** (P2, design): proposal-only change `add-agent-session-capture`
  deciding the **distilled** agent-session capture surface (turu whisper
  bundles) over raw transcripts, with the target event shape, per-field
  bounds, redaction rules, a per-entry dedup key and the cites-only linking
  hook. No production code.

Gate-driven extras in the same commits: the wai adapter's `capture`/
`capture_file` were split and the capture-attribute tests moved to
`tests/wai_capture.rs` (pretender function/file limits); the over-long
"Source authority retained" requirement was split into two requirements in
the deployed kaya-adapters spec and its archived delta.

## Key Decisions

- The pointers-only ownership rule stays total: no adapter copies its
  owner's text; a bounded content-derived attribute is allowed only where the
  owner keeps the full text. That rule is what makes the distilled
  agent-session surface the only coherent choice today.
- Attribution is decided **per decision**, not per ledger: a mixed ledger
  scopes the cited decisions and marks the uncited ones.
- The dedup key for session capture is **per entry** and excludes the bundle
  digest — folding the bundle digest into the id re-ids every earlier entry
  on append, which defeats incremental import.
- Capture observations stay `ActorType::Tool` (the adapter emitted them);
  `Agent` would change event bytes under a stable id and the ledger would
  refuse the re-import with `kaya.ledger.id_reuse`.
- The user authorized a push (fleet policy otherwise blocks it), which also
  empties the pretender gate's scan scope.

## Gotchas & Surprises

- The `pretender` gate scans files changed relative to `origin/main`, not the
  staged set: touching an already-over-limit file (`src/model.rs`,
  `src/projection/report.rs`, `tests/*`) surfaces pre-existing debt and
  fails the commit with "0 fixed, N still open". Pushing empties the scope;
  the alternative is splitting those files.
- `openspec validate --strict` fails on requirement text over 500
  characters — a warning is fatal under strict.
- A pipeline step needs an artifact added **at** that step, and the
  ro5u-review gate wants a review attached to every artifact registered in
  the run; adding artifacts out of order makes the gate name them one by one.
- Subagent dispatch is quota-limited on this account (429 "Token Plan usage
  limit"): two q6s fix agents and the d0n drafting agent died mid-run. One
  killed agent had also written stray edits into `src/projection/report.rs`
  and the deployed spec — both were reverted to the committed state.
- `openspec archive` needs `skip_specs: true` in the change's
  `.openspec.yaml` for a proposal-only change, and the file also needs
  `schema: spec-driven` or the marker is ignored.

## What Took Longer Than Unexpected

- q6s: the first review round found 2 Critical + 3 High + 6 Medium, all
  about the renderer's regime granularity, the marker gating, and the
  property-coverage mirror — the fix round rewrote the renderer, the delta
  and 20 binding files.
- Binding slugs must equal the scenario heading slug and live under the
  change overlay (`.espectacular/changes/<change>/<cap>/`) before deployment.
- Every pipeline step wants its own artifact; three rounds of
  "artifact has no review" errors before the run advanced.

## Open Questions

- The three P3 tickets from the plan's queue are still open: `kaya-xjp`
  (turu source row in the verification facts census), `kaya-7pj` (test
  report input on the stable toolchain), `kaya-8f4` (publication block
  vocabulary — same validator territory as q5e, different family/kind).
- `kaya-3tm` (release 0.2.0) is now unblocked (9b1+q6s+q5e are closed) but
  tagging v0.2.0 needs explicit per-release human authorization
  (RELEASING.md) — stop before the tag.
- `kaya-aaz` (DocIR) stays parked by decision.
- Two deferred lows from kaya-q5e remain unfixed: an empty `# ` heading
  yields an empty title, and a BOM-prefixed heading is not captured.

## Next Steps

1. `wai pipeline start tdd-ro5u --topic="kaya-xjp verification facts census"`
   (smallest of the three), then `kaya-7pj`, then `kaya-8f4`.
2. After those, `kaya-3tm` — but ask for explicit authorization before
   tagging v0.2.0 (fleet policy: push/deploy/release always ask).
3. Re-run `just ci`, `ah check --run-tests` and `openspec validate
   --specs --strict` before each ship-close; the pretender gate needs either
   a pushed branch or a real reduction of the open violation set.

## Context

### open_issues

```
○ kaya-3tm P2 Release 0.2.0
○ kaya-7pj P3 test report input on the stable toolchain
○ kaya-8f4 P3 Publication pages use blocks outside the document vocabulary
○ kaya-aaz P3 Design thread: DocIR, doctests, documentation commit gate
○ kaya-xjp P3 kaya-verification facts census: add turu source row

--------------------------------------------------------------------------------
Total: 5 issues (5 open, 0 in progress)

Status: ○ open  ◐ in_progress  ● blocked  ✓ closed  ❄ deferred
Priority: P0–P4 (label only; not a status icon)
```

