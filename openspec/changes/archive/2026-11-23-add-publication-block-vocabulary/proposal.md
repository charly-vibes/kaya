# Proposal: add-publication-block-vocabulary

## Why

A publication page carries a `publication.<tier>.<name>` managed block, which
is outside the kaya.documents family.kind vocabulary, so the shared
`validate_generated_blocks` gate refuses it and is not applied to
`docs/public`.

That asymmetry is not merely a missing gate: it hid a defect. A page
carrying a stale block from an earlier naming scheme — a misspelled tier, a
name from another tier, or a document-vocabulary kind — passed the build
silently. The publisher replaces only the block it names, so the stale block
survived beside the fresh one, and `kaya check` reported no drift for it
either, because the comparison is byte-wise and never classifies a name.

A vocabulary that is only described is not enforced. Publications get their own
block vocabulary, derived from the tier vocabularies that already exist, and
their own validator, applied where publication pages are planned.

## What Changes

- `src/publication/mod.rs`: a publication page's managed blocks are validated
  against the publication vocabulary — `publication.<tier>.<name>`, with the
  tier and the name from that tier's closed vocabulary — before anything is
  written. A name outside it is refused with a labeled error naming the block
  and the page (`kaya.publications.block_vocabulary`). The validation runs both
  where a page is planned and where an omitted product's page is removed, so
  no path can write a page carrying an unnamed block.
- Spec: `kaya-publications` gains the publication block vocabulary requirement
  and its three scenarios, plus the `block_vocabulary` property row in the
  layer sidecar with its mirror scenario.
- Docs: `docs/src/document-taxonomy.md` names the publication block vocabulary
  alongside the record-layer vocabulary.

## Impact

- Specs: kaya-publications ADDED (publication block vocabulary).
- Code: `src/publication/mod.rs` only.
- Docs: `docs/src/document-taxonomy.md` and the regenerated book page.
- Tickets: kaya-8f4.

## Out of Scope

- The eleven research document kinds and their vocabulary are unchanged; the
  research-side gate keeps refusing an out-of-vocabulary `<family>.<kind>` and
  a repeated name.
- The managed-block marker grammar is unchanged; only which names a
  publication page may carry is pinned.
- No new vocabulary list: the publication vocabulary is derived from the tier
  vocabularies the layer already owns.
