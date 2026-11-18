# Proposal: add-document-taxonomy

## Why

kaya's origin conversation ("event-driven research documentation system",
chatgpt, November 2026, transcript `6afa5381-da7c-83e9-8f6b-9dfb71a97d9d`)
worked out what kinds of documentation a research-driven project should have.
That work never reached the repository: the in-repo distillation
(`.wai/projects/kaya/research/2026-11-16-origin-conversation-…md`) kept six
decisions and pointed at `docs/src/index.md`, which names four projection
outputs and specifies none of them. Grep across the repository for
`experiment specification`, `knowledge brief`, or `reproduction package`
returns nothing outside that transcript.

The omission is now visible in behavior:

- `experiment-report` is one generated block mixing brief, specification,
  record, and findings — the source conversation's central warning was that
  "what we intended to do is not the same as what we actually did".
- Empty sections render `_(none)_` rows instead of being omitted, so a
  dossier looks populated when nothing was recorded.
- The research index lists nodes grouped by type and never renders a single
  relationship, although nine typed relationships are validated.
- kaya has no client-facing layer at all: two of four named projections
  exist, both internal.

kaya's scope per the same conversation is two things at once: managing the
research data, and creating the client-facing documentation from it. Only the
first half has a spec.

## What Changes

- Add two capability deltas, dual-format (plain openspec delta + specodelic
  layer sidecar, per the spec-integration convention):
  - `kaya.documents` — the record-facing layer: four families, eleven kinds,
    intent-versus-record separation, family-scoped generated blocks,
    omit-empty projection, reference-only reuse/handoff families, and a
    research map that renders typed edges.
  - `kaya.publications` — the client-facing layer: three tiers (product,
    collection, package), a closed product vocabulary of twelve,
    composition without duplication, justified emission (no stub pages),
    reader-question binding, and one source for every delivery format.
- Add `docs/src/document-taxonomy.md`: the recovered taxonomy with provenance,
  the family → publication mapping, what exists today, and what is deferred.
- Update `docs/src/index.md`, `docs/src/status.md`, and the book summary to
  point at the taxonomy instead of the four-name placeholder list.
- No implementation code changes. Every scenario is a contract-test shape for
  tickets that follow (see tasks.md).

## Decisions baked in

1. **Two layers, not one list.** The eleven document kinds are the projection
   interface over the ledger and model; the twelve products are publication
   packages composed from those kinds. A product is never a document type, and
   a collection is never a product.
2. **Intent and record are separate projections.** `experiment-specification`
   reads declared intent; `experiment-record` reads observed execution. Neither
   is derived from the other, and one generated block cannot mix them.
3. **Omission over placeholders.** A kind with no records is omitted and
   reported, never rendered as an empty section.
4. **Reuse families carry references, not copies.** Technical reference,
   reproduction package, knowledge brief, and handoff point at their
   authoritative sources; kaya does not become a second knowledge store.
5. **Relationship rendering is part of the taxonomy.** A research map without
   edges is not a map, so `research-map` must render every declared edge.
6. **DocIR, doctests, and the documentation commit gate are out of scope.**
   The same transcript developed them (change → impact set → dispositions →
   validation → gate; Org as authoring frontend). They are parked as one
   design ticket (kaya-aaz) because they change kaya's central artifact, not
   its document list.

## Impact

- Specs: +2 capabilities (new). Nothing modified or removed.
- Docs: `docs/src/document-taxonomy.md` added; `docs/src/index.md`,
  `docs/src/status.md`, `docs/src/SUMMARY.md` updated.
- Code: none. Validation runs the spec corpus gates only.
- Tickets: kaya-s5v (document-family projections), kaya-i03 (research-map
  edges), kaya-3f6 (reference-only reuse families), kaya-qp8 (publication
  composition), kaya-aaz (DocIR/doctest/gate thread, parked), kaya-cc9
  (archive and deploy this change).