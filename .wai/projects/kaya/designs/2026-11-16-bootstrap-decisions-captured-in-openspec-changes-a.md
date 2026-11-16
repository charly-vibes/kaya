Bootstrap decisions captured in openspec/changes/add-core-research-record:

1. Events are not genesis envelopes — kaya.event/v0 is the ledger record
   schema; CLI command I/O keeps genesis envelope conventions. The seam is a
   declared extension_point (envelope_independence), revisited after the
   first end-to-end dossier. (Resolves spec-review finding F2 by decision,
   not accident.)
2. Capabilities namespaced kaya.* (event, ledger, model, projection,
   promotion, adapters) — pre-empts the near-name-collision class found in
   the specodelic suite review (model.check vs model_check).
3. Corrections are new superseding events; ledger append-only at byte level.
4. Dual-format authoring per specodelic's spec-integration convention:
   plain openspec delta + layer.md sidecar; all six layers lint clean under
   spk lint (EARS statements, closed row shapes, coverage-mirror properties,
   emits edges on failure terminals).
5. VERIFIES scenarios in deltas reference deployed-form ids
   (kaya.<cap>.<row>); layer-internal refs use the sidecar namespace
   (kaya.<cap>.layer.<row>) — mirrors the preservation precedent.

