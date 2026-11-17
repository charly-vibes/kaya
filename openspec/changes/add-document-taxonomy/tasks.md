# Tasks: add-document-taxonomy

This change is specification and documentation only. Each scenario in the two
deltas is a contract-test shape for the implementation tickets that follow; no
code changes here. bd ticket mapping noted per phase.

## 1. Spec deltas

- [x] 1.1 `kaya.documents` delta + specodelic layer: family vocabulary,
      intent-versus-record, family-scoped blocks, omit-empty, reference-only
      reuse families, research map edges. (kaya-s5v, kaya-i03, kaya-3f6)
- [x] 1.2 `kaya.publications` delta + specodelic layer: three tiers, closed
      product vocabulary, composition without duplication, justified emission,
      reader-question binding, one source for every format. (kaya-qp8)
- [x] 1.3 Record the deferred DocIR / doctest / documentation-commit-gate
      thread as a design ticket instead of a spec. (kaya-aaz)

## 2. Documentation

- [x] 2.1 Add `docs/src/document-taxonomy.md`: recovered taxonomy with
      provenance, family and product tables, mapping, status, deferrals.
- [x] 2.2 Point `docs/src/index.md` architecture and milestones at the
      taxonomy; add the book summary entry; add a status row.
- [x] 2.3 Archive checklist for this change recorded (kaya-cc9).

## 3. Implementation tickets (red → green, one tdd-ro5u run each)

- [ ] 3.1 Document-family projection: closed vocabulary, `<family>.<kind>`
      block names, intent/record separation, omit-empty with coverage summary.
      Red tests are the `kaya.documents` coverage-mirror scenarios. (kaya-s5v)
- [ ] 3.2 Research map: render typed edges with both endpoints; list isolated
      entities. (kaya-i03)
- [ ] 3.3 Reuse and handoff families: reference-only projections; no ledger
      entities created. Waits on the specodelic and turu adapters for its
      richest inputs; wai handoff ids suffice to start. (kaya-3f6)
- [ ] 3.4 Publication composition: tier separation, admission table,
      justified emission, multi-format export from one source. (kaya-qp8)

## Archive checklist

- [ ] All `kaya.documents` and `kaya.publications` scenarios have executing
      contract tests (`ah check --changes add-document-taxonomy --run-tests`).
- [ ] `spk lint openspec/specs` clean after the two layers deploy.
- [ ] bd tickets closed with scenario references.
- [ ] Archive companion deploys the layers; book "Design specs" section
      updated. (kaya-cc9)