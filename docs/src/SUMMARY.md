# Summary

- [kaya — motivation & design](index.md)
- [Documentation taxonomy](document-taxonomy.md)
- [Status](status.md)

---

# Design specs

Deployed capability specs live in `openspec/specs/` with their specodelic
layer beside each as `layer.md`. `just specs-docs` copies both into this book,
and `just docs` runs it before `mdbook build`:

- [kaya.documents](specs/kaya-documents.md)
- [kaya.publications](specs/kaya-publications.md)

`add-core-research-record` is still an active change, so its six layers
(kaya-event, kaya-ledger, kaya-model, kaya-projection, kaya-promotion,
kaya-adapters) are readable only as deltas in
`openspec/changes/add-core-research-record/specs/` for now.
