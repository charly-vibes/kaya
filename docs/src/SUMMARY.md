# Summary

- [kaya — motivation & design](index.md)
- [Documentation taxonomy](document-taxonomy.md)
- [Status](status.md)

---

# Design specs

Deployed capability specs live in `openspec/specs/`. The two document-taxonomy
capabilities keep a specodelic layer beside each as `layer.md`; the research
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

`kaya-adapters` is not deployed yet: its capability spec stays in the
`2026-11-18-add-core-research-record` archive (readable at
`openspec/changes/archive/2026-11-18-add-core-research-record/specs/kaya-adapters/`)
until the deferred turu adapter (kaya-syx) binds its last unbound scenario;
then it deploys with its contracts.
