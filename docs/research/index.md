# kaya research index

<!-- kaya:authored -->
Scope: this directory holds kaya's compiled research record for this
repository. The index below is a projection of the ledger at
`.kaya/ledger.jsonl`: it lists the typed relationships between admitted
`kaya.event/v0` entities, the admitted entities that have no relationship
edges (except the Artifacts, Runs, and Sources carried by their own detail
sections), and then those Artifact, Run, and Source detail sections. A
dossier under `docs/research/experiments/` answers a research question from
that same record.

How to read provenance: every rendered line names a namespace-qualified node
id (`<project>/<TYPE>/<key>`), and that id resolves through the ledger to the
event that created it. Event provenance names the emitting tool, its version,
and an external reference — an artifact path, commit sha, or session id —
that stays resolvable without kaya running. Authored blocks cite repository
files directly, pinned to a commit where the citation depends on content that
may move. A dossier's interpretation section shows proposals with their
promotion status (`proposed`, `established`, `challenged`); a proposal is never
a verdict.

kaya owns only the bodies of `kaya:begin generated:<name>` /
`kaya:end generated:<name>` blocks (generated) and `kaya:authored` /
`kaya:end authored` blocks (human-owned). Every other byte — including other
tools' managed blocks — is preserved verbatim. This file's generated block is
`orientation.research-map`: `kaya build` appends it on its first run and
rewrites only that block afterwards.

Authored blocks may cite ledger evidence with a `kaya:cites` marker. This
page cites the record's founding acceptance question
<!-- kaya:cites evt-b64818d83f41e6892fda19b1fc -->.
<!-- kaya:end authored -->

<!-- other-tool:begin -->
This block belongs to another tool. kaya must preserve these bytes exactly
across every rebuild; it is here to prove the preservation in acceptance
tests and in real use.
<!-- other-tool:end -->

<!-- kaya:begin generated:orientation.research-map -->
### Relationships
- kaya/Hypothesis/H-001 -[predicts]-> kaya/Experiment/add-core-research-record
- kaya/Model/M-001 -[realizes]-> kaya/Hypothesis/H-001
- kaya/Question/mvp-1-acceptance -[investigates]-> kaya/Experiment/add-core-research-record

### Artifacts
- kaya/Artifact/0cd2fd6 — 0cd2fd6af445980341e9687ca6ebb1b36882ba6c
- kaya/Artifact/1e0c0d7 — 1e0c0d797827152a76043f8dac2bc7a120fda03a
- kaya/Artifact/510751b — 510751b95220661ef118a3a59d3447e22ae125d3
- kaya/Artifact/565343c — 565343cf4dd30d4b1b010acb332259ecb44127f4
- kaya/Artifact/59fea0a — 59fea0aec4136495b44e93afefdae9590c49851c
- kaya/Artifact/645a6b5 — 645a6b525500fb99d35df579a091eb0991f51c00
- kaya/Artifact/687b6d9 — 687b6d962743a6eeb544cb6ffcd0b184bd35a610
- kaya/Artifact/798376c — 798376ce2a5d223c00a2e59ce71c112d18341b86
- kaya/Artifact/904f37f — 904f37fc6fa03785c3df4b48960dda5cf6120d2f
- kaya/Artifact/a3ecdc2 — a3ecdc294927a6a915831c4ed7ed73808897970c
- kaya/Artifact/a68206a — a68206a258ed7a67142c9cba0924751904b0d682
- kaya/Artifact/ab319d5 — ab319d59bb12c47e1da09783bb91c4300d61db61
- kaya/Artifact/d3bc2f4 — d3bc2f46f85e61173de0bd7e492b15deaf6361f4
- kaya/Artifact/d56e106 — d56e10647381a3272ef72d759a919bab8cdeff17
- kaya/Artifact/designs-2026-11-16-bootstrap-decisions-captured-in-openspec-changes-a — .wai/projects/kaya/designs/2026-11-16-bootstrap-decisions-captured-in-openspec-changes-a.md
- kaya/Artifact/designs-2026-11-16-green-kaya-10k-serde-event-with-closed-field-set — .wai/projects/kaya/designs/2026-11-16-green-kaya-10k-serde-event-with-closed-field-set.md
- kaya/Artifact/designs-2026-11-16-green-kaya-5ea-requirement-scenarios-added-lay — .wai/projects/kaya/designs/2026-11-16-green-kaya-5ea-requirement-scenarios-added-lay.md
- kaya/Artifact/designs-2026-11-16-green-kaya-f2f-btreemap-graph-direction-grammar — .wai/projects/kaya/designs/2026-11-16-green-kaya-f2f-btreemap-graph-direction-grammar.md
- kaya/Artifact/designs-2026-11-16-green-kaya-lb2-clap-cli-init-status-local-genesi — .wai/projects/kaya/designs/2026-11-16-green-kaya-lb2-clap-cli-init-status-local-genesi.md
- kaya/Artifact/designs-2026-11-16-green-kaya-vt9-justfile-recipes-and-two-workflows — .wai/projects/kaya/designs/2026-11-16-green-kaya-vt9-justfile-recipes-and-two-workflows.md
- kaya/Artifact/designs-2026-11-16-implement-every-ticket-as-one-tdd-ro5u-pipeline-ru — .wai/projects/kaya/designs/2026-11-16-implement-every-ticket-as-one-tdd-ro5u-pipeline-ru.md
- kaya/Artifact/designs-2026-11-16-kaya-motivation-design-documentation-that — .wai/projects/kaya/designs/2026-11-16-kaya-motivation-design-documentation-that.md
- kaya/Artifact/designs-2026-11-16-mvp-1-implementation-contract-orchestrated-2026-1 — .wai/projects/kaya/designs/2026-11-16-mvp-1-implementation-contract-orchestrated-2026-1.md
- kaya/Artifact/designs-2026-11-16-post-bootstrap-alignment-consumers-mvp-1-accep — .wai/projects/kaya/designs/2026-11-16-post-bootstrap-alignment-consumers-mvp-1-accep.md
- kaya/Artifact/designs-2026-11-16-refactor-kaya-10k-resolve-resolve-effective-split — .wai/projects/kaya/designs/2026-11-16-refactor-kaya-10k-resolve-resolve-effective-split.md
- kaya/Artifact/designs-2026-11-16-refactor-kaya-5ea-front-matter-statement-now-enum — .wai/projects/kaya/designs/2026-11-16-refactor-kaya-5ea-front-matter-statement-now-enum.md
- kaya/Artifact/designs-2026-11-16-refactor-kaya-f2f-attribute-merge-on-node-redefin — .wai/projects/kaya/designs/2026-11-16-refactor-kaya-f2f-attribute-merge-on-node-redefin.md
- kaya/Artifact/designs-2026-11-16-refactor-kaya-lb2-envelope-kept-isolated-in-src-e — .wai/projects/kaya/designs/2026-11-16-refactor-kaya-lb2-envelope-kept-isolated-in-src-e.md
- kaya/Artifact/designs-2026-11-16-refactor-kaya-lb2-no-production-refactor-needed — .wai/projects/kaya/designs/2026-11-16-refactor-kaya-lb2-no-production-refactor-needed.md
- kaya/Artifact/designs-2026-11-16-refactor-kaya-vt9-ci-smoke-step-checks-bd-ah-spk — .wai/projects/kaya/designs/2026-11-16-refactor-kaya-vt9-ci-smoke-step-checks-bd-ah-spk.md
- kaya/Artifact/e178048 — e178048e81017f9667b091311aa7ce912483cd0e
- kaya/Artifact/e2ada43 — e2ada437871f35b4c3cc2857022e3d2b92ba1c86
- kaya/Artifact/e4c40ef — e4c40efaeb1c83759d062bde90d65477d2926d38
- kaya/Artifact/e4e3aaa — e4e3aaab7b9d8ae5d94504b27f2b6ec0ba5e0393
- kaya/Artifact/e5cf768 — e5cf768fa020c223b7ea533fff65864528703312
- kaya/Artifact/fbd08ac — fbd08ac7a63976b3985b270856a4df7313d9f65c
- kaya/Artifact/plans-2026-11-16-plan-kaya-10k-event-serde-parse-line-to-line-wi — .wai/projects/kaya/plans/2026-11-16-plan-kaya-10k-event-serde-parse-line-to-line-wi.md
- kaya/Artifact/plans-2026-11-16-plan-kaya-5ea-one-requirement-proposal-budget-is — .wai/projects/kaya/plans/2026-11-16-plan-kaya-5ea-one-requirement-proposal-budget-is.md
- kaya/Artifact/plans-2026-11-16-plan-kaya-f2f-entitytype-relationship-enums-node — .wai/projects/kaya/plans/2026-11-16-plan-kaya-f2f-entitytype-relationship-enums-node.md
- kaya/Artifact/plans-2026-11-16-plan-kaya-lb2-desired-behavior-init-status-envelo — .wai/projects/kaya/plans/2026-11-16-plan-kaya-lb2-desired-behavior-init-status-envelo.md
- kaya/Artifact/plans-2026-11-16-plan-kaya-vt9-justfile-recipes-build-test-fmt-ch — .wai/projects/kaya/plans/2026-11-16-plan-kaya-vt9-justfile-recipes-build-test-fmt-ch.md
- kaya/Artifact/research-2026-11-16-orient-kaya-10k-closed-nine-field-kaya-event-v0-s — .wai/projects/kaya/research/2026-11-16-orient-kaya-10k-closed-nine-field-kaya-event-v0-s.md
- kaya/Artifact/research-2026-11-16-orient-kaya-5ea-promotion-rate-cap-requirement-fr — .wai/projects/kaya/research/2026-11-16-orient-kaya-5ea-promotion-rate-cap-requirement-fr.md
- kaya/Artifact/research-2026-11-16-orient-kaya-f2f-closed-thirteen-entity-vocabulary — .wai/projects/kaya/research/2026-11-16-orient-kaya-f2f-closed-thirteen-entity-vocabulary.md
- kaya/Artifact/research-2026-11-16-orient-kaya-lb2-crate-scaffold-with-kaya-init-sta — .wai/projects/kaya/research/2026-11-16-orient-kaya-lb2-crate-scaffold-with-kaya-init-sta.md
- kaya/Artifact/research-2026-11-16-orient-kaya-vt9-just-ci-ci-yml-per-fleet-standa — .wai/projects/kaya/research/2026-11-16-orient-kaya-vt9-just-ci-ci-yml-per-fleet-standa.md
- kaya/Artifact/research-2026-11-16-origin-conversation-event-driven-research-docum — .wai/projects/kaya/research/2026-11-16-origin-conversation-event-driven-research-docum.md
- kaya/Artifact/research-2026-11-16-quality-ledger-kaya-10k-changed-src-event-rs-src — .wai/projects/kaya/research/2026-11-16-quality-ledger-kaya-10k-changed-src-event-rs-src.md
- kaya/Artifact/research-2026-11-16-quality-ledger-kaya-5ea-changed-kaya-promotion-sp — .wai/projects/kaya/research/2026-11-16-quality-ledger-kaya-5ea-changed-kaya-promotion-sp.md
- kaya/Artifact/research-2026-11-16-quality-ledger-kaya-f2f-changed-src-model-rs-test — .wai/projects/kaya/research/2026-11-16-quality-ledger-kaya-f2f-changed-src-model-rs-test.md
- kaya/Artifact/research-2026-11-16-quality-ledger-kaya-lb2-changed-cargo-toml-src — .wai/projects/kaya/research/2026-11-16-quality-ledger-kaya-lb2-changed-cargo-toml-src.md
- kaya/Artifact/research-2026-11-16-quality-ledger-kaya-vt9-changed-justfile-github — .wai/projects/kaya/research/2026-11-16-quality-ledger-kaya-vt9-changed-justfile-github.md
- kaya/Artifact/research-2026-11-16-red-kaya-10k-tests-event-rs-and-tests-ledger-rs-w — .wai/projects/kaya/research/2026-11-16-red-kaya-10k-tests-event-rs-and-tests-ledger-rs-w.md
- kaya/Artifact/research-2026-11-16-red-kaya-5ea-spec-linter-would-reject-unbound-pro — .wai/projects/kaya/research/2026-11-16-red-kaya-5ea-spec-linter-would-reject-unbound-pro.md
- kaya/Artifact/research-2026-11-16-red-kaya-f2f-model-tests-written-first-expected — .wai/projects/kaya/research/2026-11-16-red-kaya-f2f-model-tests-written-first-expected.md
- kaya/Artifact/research-2026-11-16-red-kaya-lb2-command-cargo-test-test-cli-expec — .wai/projects/kaya/research/2026-11-16-red-kaya-lb2-command-cargo-test-test-cli-expec.md
- kaya/Artifact/research-2026-11-16-red-kaya-vt9-no-justfile-workflow-existed-just-c — .wai/projects/kaya/research/2026-11-16-red-kaya-vt9-no-justfile-workflow-existed-just-c.md
- kaya/Artifact/research-2026-11-16-ro5u-fixes-kaya-10k-read-verified-anchor-mandato — .wai/projects/kaya/research/2026-11-16-ro5u-fixes-kaya-10k-read-verified-anchor-mandato.md
- kaya/Artifact/research-2026-11-16-ro5u-fixes-kaya-5ea-pull-only-surfacing-constrain — .wai/projects/kaya/research/2026-11-16-ro5u-fixes-kaya-5ea-pull-only-surfacing-constrain.md
- kaya/Artifact/research-2026-11-16-ro5u-fixes-kaya-f2f-exact-direction-pairs-correc — .wai/projects/kaya/research/2026-11-16-ro5u-fixes-kaya-f2f-exact-direction-pairs-correc.md
- kaya/Artifact/research-2026-11-16-ro5u-fixes-kaya-lb2-init-non-regular-file-refusal — .wai/projects/kaya/research/2026-11-16-ro5u-fixes-kaya-lb2-init-non-regular-file-refusal.md
- kaya/Artifact/research-2026-11-16-ro5u-fixes-kaya-vt9-none-required-ci-recipe-re-r — .wai/projects/kaya/research/2026-11-16-ro5u-fixes-kaya-vt9-none-required-ci-recipe-re-r.md
- kaya/Artifact/research-2026-11-16-ship-kaya-10k-committed-in-904f37f-family-ticket — .wai/projects/kaya/research/2026-11-16-ship-kaya-10k-committed-in-904f37f-family-ticket.md
- kaya/Artifact/research-2026-11-16-ship-kaya-5ea-committed-a68206a-spec-follow-ups — .wai/projects/kaya/research/2026-11-16-ship-kaya-5ea-committed-a68206a-spec-follow-ups.md
- kaya/Artifact/research-2026-11-16-ship-kaya-f2f-committed-e178048-904f37f-ticket — .wai/projects/kaya/research/2026-11-16-ship-kaya-f2f-committed-e178048-904f37f-ticket.md
- kaya/Artifact/research-2026-11-16-ship-kaya-lb2-cli-scaffold-shipped-in-commits-d3b — .wai/projects/kaya/research/2026-11-16-ship-kaya-lb2-cli-scaffold-shipped-in-commits-d3b.md
- kaya/Artifact/research-2026-11-16-ship-kaya-vt9-committed-798376c-ticket-close-fol — .wai/projects/kaya/research/2026-11-16-ship-kaya-vt9-committed-798376c-ticket-close-fol.md

### Sources
- kaya/Source/docs-src-index.md@43b606f — docs/src/index.md@43b606f
- kaya/Source/openspec-changes-add-core-research-record-proposal.md@43b606f — openspec/changes/add-core-research-record/proposal.md@43b606f
- kaya/Source/openspec-changes-add-core-research-record-specs-kaya-adapters-spec.md@43b606f — openspec/changes/add-core-research-record/specs/kaya-adapters/spec.md@43b606f
- kaya/Source/openspec-changes-add-core-research-record-specs-kaya-event-layer.md@43b606f — openspec/changes/add-core-research-record/specs/kaya-event/layer.md@43b606f
- kaya/Source/openspec-changes-add-core-research-record-specs-kaya-promotion-spec.md@43b606f — openspec/changes/add-core-research-record/specs/kaya-promotion/spec.md@43b606f
- kaya/Source/wai-projects-kaya-designs-2026-11-16-post-bootstrap-alignment-consumers-mvp-1-accep.md@43b606f — .wai/projects/kaya/designs/2026-11-16-post-bootstrap-alignment-consumers-mvp-1-accep.md@43b606f

_77 nodes from 80 events._
<!-- kaya:end generated:orientation.research-map -->
