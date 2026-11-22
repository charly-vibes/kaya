# Tasks: add-agent-session-capture

Proposal-only change: decide the agent-session capture surface and record the
commitments the follow-up implementation must honour. No production code.

## 1. Research and decision (this change)

- [x] 1.1 Orient on the current capture surface: the turu adapter's
      `capture.entry.observed` event with its subject, actor and data fields,
      the pointers-only ownership rule (`src/adapters/turu.rs:41`), the wai
      bounded-attribute precedent (`src/adapters/wai.rs:7`), and the closed
      event, entity, relationship and actor vocabularies.
- [x] 1.2 Decide the surface: distilled whisper bundles; raw transcripts
      rejected with four explicit unlock conditions, plus the carrier
      precondition (agent-session material must actually be published as
      whisper bundles).
- [x] 1.3 Fix the five coverage areas as design commitments, corrected
      against the Rule-of-5 review: (a) redaction applies to every
      session-derived string — `topic` and `scope` included — and replaces
      the matched secret with a fixed marker while preserving the
      surrounding attribute, with a default-secret rule for high-entropy
      runs; (b) size bounds are per field (`topic` ≤ 80 chars on a character
      boundary, `entry`/`scope` ≤ 64, `bundle_digest` 64 hex as shipped),
      with the per-import entry cap deferred to the implementation ticket
      and the failure mode fixed now (atomic refusal, never a partial
      capture); (c) the dedup key is defined per entry and never over the
      whole bundle file, so appending an entry does not re-id earlier ones;
      (d) the event kind stays `capture.entry.observed` with
      `ActorType::Tool` as shipped — any actor change needs a
      `SOURCE_VERSION` bump because the id excludes the actor; (e) the
      linking hook is `cites` Decision → Source, and Finding → Source needs
      a relationship-set delta.
- [x] 1.4 State the target event shape (type, subject, actor, every data
      key with its bound and its redaction obligation) so the implementer
      does not infer it.
- [x] 1.5 Record the falsification evidence, the risks with their triggers
      and detection signals, and the four conditions that unlock the raw
      surface.

## 2. Follow-up tickets to file (spawned by this proposal, not part of it)

- 2.1 Implement the distilled session adapter: target event shape,
  redaction filter, per-field bounds, per-entry dedup key, contract tests.
  (to be filed)
- 2.2 Derive the per-import entry cap from the ledger's per-line size budget
  and state the number. (to be filed)
- 2.3 Validate the redaction pattern list against a real transcript corpus;
  extend it with the misses. (to be filed)
- 2.4 Verify the carrier precondition: agent-session material is published
  as turu whisper bundles. (to be filed)
- 2.5 Re-open the raw surface only when all four unlock conditions in the
  proposal's Decision section hold. (to be filed)

## 3. Verify (orchestrator)

- [x] 3.1 `openspec validate --changes --strict` passes.
- [x] 3.2 `git diff --stat src/` is empty — no production code.
- [x] 3.3 `cargo test` and `cargo clippy --all-targets -- -D warnings` stay
      green.
- [x] 3.4 Rule-of-5 review of the proposal: CRITICAL 0 after the fixes
      recorded in the run's fix artifact.

## 4. Tests the implementation ticket must carry (recorded for that ticket)

- 4.1 Append-and-re-import dedup test: import entries A and B, append C,
  re-import — exactly one new event, ids for A and B unchanged.
- 4.2 `topic` redaction fixture: a topic containing a pasted secret is
  redacted with the marker while the surrounding text survives.
- 4.3 Per-field bound fixtures: `topic` truncation on a character boundary,
  `entry`/`scope` length caps.
- 4.4 Over-cap import refused atomically with
  `kaya.adapters.failure_atomic`, naming the bundle.
