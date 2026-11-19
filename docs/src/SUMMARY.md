# Summary

- [kaya — motivation & design](index.md)
- [Documentation taxonomy](document-taxonomy.md)
- [Status](status.md)

---

# Design specs

Deployed capability specs live in `openspec/specs/`. Three capabilities keep a
specodelic layer beside each as `layer.md` — the two document-taxonomy
capabilities and kaya-adapters; the research
record capabilities carry their layer inline (frontmatter plus
Constraints/Model/Properties tables inside `spec.md`). `just specs-docs`
copies each capability into this book, and `just docs` runs it before
`mdbook build`:

- [kaya.documents](specs/kaya-documents.md)
- [kaya.publications](specs/kaya-publications.md)
- [kaya.event](specs/kaya-event.md)
- [kaya.ledger](specs/kaya-ledger.md)
- [kaya.model](specs/kaya-model.md)
- [kaya.projection](specs/kaya-projection.md)
- [kaya.promotion](specs/kaya-promotion.md)
- [kaya.verification](specs/kaya-verification.md)
- [kaya.adapters](specs/kaya-adapters.md)

`kaya-adapters` is deployed: its capability spec lives at
`openspec/specs/kaya-adapters/spec.md` (formalized from
`openspec/changes/archive/2026-11-18-add-core-research-record/specs/kaya-adapters/`)
with its contracts under `.espectacular/kaya-adapters/`.
