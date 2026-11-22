---
tags: [pipeline-run:tdd-ro5u-2026-11-22-kaya-d0n-agent-session-capture-proposal, pipeline-step:fix-review]
---

# RO5U-FIXES kaya-d0n — agent-session capture proposal

Run: tdd-ro5u-2026-11-22-kaya-d0n-agent-session-capture-proposal (step 7/9).
Review: /tmp/kaya-d0n/ro5u-review.md — FAIL, 2 Critical, 3 High, 6 Medium,
4 Low. All findings closed.

## Critical
- **C1 (bounds and redaction described the wrong adapter).** The proposal
  now states the ownership rule as the code states it ("no adapter copies
  the owner's text"; a bounded content-derived attribute is allowed where
  the owner keeps the full text and the attribute has a declared bound —
  `src/adapters/turu.rs:41`, `src/adapters/wai.rs:7`) and adds a **Target
  event shape** table: type `capture.entry.observed`, subject
  `<project>/Source/entry-<entry id>`, actor `Tool`, and every data key with
  its bound (`entry` ≤ 64, `topic` ≤ 80 on a character boundary, `scope`
  ≤ 64, `bundle_digest` 64 hex as shipped) and its redaction obligation
  (`topic` and `scope` are session-derived free text and must be redacted).
- **C2 (dedup key undefined and re-ids every entry).** The dedup key is now
  defined per entry and explicitly excludes the bundle digest: sha256 over
  `tool \x1f source_version \x1f project \x1f <normalised ts> \x1f <entry id>
  \x1f <topic> \x1f <scope>`. The bundle digest travels as provenance and as
  `data.bundle_digest` only. The append case is now a required test
  (import A,B → append C → re-import → one new event, A and B ids
  unchanged), and the schema-visible re-id of existing turu events is
  declared in Impact (a `SOURCE_VERSION` bump plus a migration note).

## High
- **H1 (actor typing).** `ActorType::Tool` stays — the adapter program
  emitted the observation (`src/event.rs:108-115`); the closed set already
  contains `Agent` and needs no change. The proposal now states the
  mechanism a future `Agent` switch would need: the id excludes the actor
  (`src/adapters/turu.rs:218-222`), so the bytes would change under a
  stable id and the ledger would refuse the re-import with
  `kaya.ledger.id_reuse` — the correction path is a `data.supersedes`
  event, and a `SOURCE_VERSION` bump is required.
- **H2 (linking hooks overstated).** The only link from a session capture
  into the graph is `cites` (Decision → Source, exact pair,
  `src/model.rs:189`) to the Source node `<project>/Source/entry-<entry id>`,
  whose node attributes carry only `entry` (`src/model.rs:439-441`); topic,
  timestamp and scope live in the event, not in the node. A Finding cannot
  point at a session today (`influences` is Decision → Finding, `derives`
  is Finding → Observation) — that gap is named as a relationship-set delta
  for the MVP-2 ticket.
- **H3 (carrier precondition asserted, not stated).** The Decision section
  now states it: distilled is admissible only if agent-session material is
  published as turu whisper bundles; if a session lives only in pi/terapia,
  distilled has no carrier. The same item is in Risks with the measurement
  from the analysis artifact.

## Medium
- **M1** The evaluation claim is now marked unverified motivation; "MVP-2"
  is defined as the follow-up generation work that has no generator caller
  yet.
- **M2** Redaction semantics are explicit — the matched secret is replaced
  with the fixed marker `[redacted]`, the surrounding attribute survives —
  and the list gains `sk-proj-`, `ASIA…`, PGP private-key blocks and
  PuTTY user-key files, plus a default-secret rule for base64/hex runs of
  ≥ 32 characters.
- **M3** The unjustified 500-entry cap is gone: the per-import cap is not
  fixed by this proposal (the implementation ticket derives it from the
  ledger's per-line size budget), while the failure mode is fixed now —
  atomic refusal with `kaya.adapters.failure_atomic`, never a partial
  capture.
- **M4** The digest length contradiction is resolved: `bundle_digest` stays
  64 hex as shipped; shortening it is not proposed.
- **M5** The risk list now carries the three failure modes the review
  found — bundle-digest churn duplicating entries on append, the ledger's
  `id_reuse` refusal on any byte change under a stable id, and an unbounded
  unredacted `topic` — each with its trigger and detection signal.
- **M6** The target event shape is stated (C1 fix).

## Low
- **L1** "the closed set gains its intended use" is corrected: the set
  already contains `Agent`; the open question is which observations are
  agent-authored.
- **L2** The `cites` relationship is cited by location (`src/model.rs:189`);
  the kaya-q5e comparison is marked as an analogy.
- **L3** The bundle's contents are described accurately, including the
  `freeform` section the adapter never reads (`src/adapters/turu.rs:8-9`).
- **L4** `tasks.md` now asserts the corrected commitments and adds the
  implementation tests the proposal requires (append-and-re-import dedup,
  `topic` redaction, per-field bounds, over-cap atomic refusal).

## Verification (orchestrator, after the fixes)
- `openspec validate --changes --strict` → pass.
- `git diff --stat src/` → empty: still no production code.
- Proposal structure: exactly four top-level sections (Decision, Why, What
  Changes, Impact).
- `cargo test` → 31/31 suites ok; `cargo clippy --all-targets -- -D
  warnings` → clean.

## Deferred
Nothing. All 15 findings are closed.

