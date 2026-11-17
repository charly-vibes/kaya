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

- [x] 3.1 Experiment dossier slice: closed vocabulary, `<family>.<kind>`
      block names, intent/record separation, omit-empty with coverage summary
      for `exploration.experiment-specification` and
      `exploration.experiment-record`. (kaya-s5v)
- [x] 3.2 Understanding slice: `findings-and-analysis` and `decision-record`.
      (kaya-249)
- [x] 3.3 Orientation slice: `project-overview`. (kaya-1b9)
- [x] 3.4 Research map: render typed edges with both endpoints; list isolated
      entities. (kaya-i03)
- [x] 3.5 Reuse slice with existing adapters: `reproduction-package` and
      `handoff` from git and wai references. (kaya-3f6)
- [x] 3.6 Publication slice: tier vocabulary, justified emission, composition,
      reader-question binding, exercised by `project-site` over `overview` and
      `reference`. (kaya-qp8)
- [ ] 3.7 Developer-guide products: `getting-started`, `how-to`, `concept`,
      `example`, `troubleshooting`, `faq`. (kaya-duv)
- [ ] 3.8 Remaining products plus structured export: `status-compatibility`,
      `release-migration`, `architecture-design`, one source for every format.
      (kaya-scq)
- [ ] 3.9 Adapter-dependent reuse kinds: `technical-reference` (specodelic) and
      `knowledge-brief` (turu). Blocked on kaya-24c and kaya-syx. (kaya-lh1)
- [ ] 3.10 Editorial cutover: replace hand-written overview and concept prose
      with generated products plus authored blocks. HITL. (kaya-n50)
- [ ] 3.11 Publish authorization: crates.io credentials and release channel.
      HITL; unblocks kaya-dgz. (kaya-rdr)
- [ ] 3.12 Deferred design thread: DocIR, doctests, documentation commit gate.
      DESIGN lane, parked until the taxonomy lands. (kaya-aaz)

## Archive checklist

- [ ] All `kaya.documents` and `kaya.publications` scenarios have executing
      contract tests (`ah check --changes add-document-taxonomy --run-tests`).
- [ ] `spk lint openspec/specs` clean after the two layers deploy.
- [ ] bd tickets closed with scenario references.
- [ ] Archive companion deploys the layers; book "Design specs" section
      updated. (kaya-cc9)