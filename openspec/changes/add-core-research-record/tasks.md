# Tasks: add-core-research-record

Red→green→refactor per capability; each scenario in the deltas is a
contract-test shape. bd ticket mapping noted per phase.

## 1. Red — failing tests first

- [ ] 1.1 Event fixtures: valid `kaya.event/v0` record; unknown-schema
      refused; missing provenance refused; id reuse refused; extra field
      refused. (kaya-10k)
- [ ] 1.2 Ledger fixtures: append-only byte check; supersede round trip;
      idempotent replay; digest stability; crash-tail (partial last line)
      recovery. (kaya-10k)
- [ ] 1.3 Model fixtures: unknown entity refused; unknown relationship
      refused; dangling reference refused; influence cycle refused.
      (kaya-f2f)
- [ ] 1.4 Projection fixtures: authored block byte-preserved across
      regeneration; deterministic bytes; incremental rebuild scope; stale
      flag on evidence change. (kaya-e9s, kaya-5cg)
- [ ] 1.5 Promotion fixtures: tool actor cannot create assertion/conclusion;
      unapproved conclusion refused; AI interpretation lands `proposed`;
      approval records human actor. (kaya-5cg)
- [ ] 1.6 Adapter fixtures: wai artifact tree → events (kaya-aw3); git +
      test-runner → events (kaya-dnk); failure writes nothing; every event
      carries provenance. (kaya-aw3, kaya-dnk)
- [ ] 1.7 Promotion budget fixtures: pending-proposal count surfaces
      without interruption; over-cap generation ranks/caps; unapproved
      proposals never expire or settle. (kaya-5ea)

## 2. Green — make the tests pass

- [ ] 2.1 Scaffold the Rust crate; CLI `kaya init`/`status` skeleton
      (kaya-lb2).
- [ ] 2.2 Event parser/validator + ledger store (kaya-10k).
- [ ] 2.3 Adapters: wai (kaya-aw3), git + test-runner (kaya-dnk).
- [ ] 2.4 Research graph builder over the closed vocabulary (kaya-f2f).
- [ ] 2.5 Projection compiler: managed-block grammar, deterministic
      generation, authored preservation, stale flagging (kaya-e9s, kaya-5cg).
- [ ] 2.6 Promotion gate: category transitions with approval provenance
      (kaya-5cg).
- [ ] 2.7 Promotion budget: pending-proposal count in status and dossier
      build, ranked/capped generation. (kaya-5ea)

## 3. Refactor — separate commit

- [ ] 3.1 Genesis-envelope seam: CLI I/O envelopes vs ledger events —
      revisit the `envelope_independence` extension point once the first
      dossier compiles end to end.
- [ ] 3.2 `just ci` green; MVP-1 validation: a useful dossier from existing
      kaya repo artifacts (self-hosting proof).

## Archive checklist

- [ ] All scenarios have executing contract tests (`ah check --run-tests`).
- [ ] `spk lint openspec/specs` clean after the archive companion deploys
      the six layers.
- [ ] bd tickets closed with scenario references.
