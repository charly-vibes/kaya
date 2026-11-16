# kaya research index

<!-- kaya:authored -->
Scope: this directory holds kaya's compiled research record for this
repository. The index below is a projection of the ledger at
`.kaya/ledger.jsonl`; it lists the Questions, Hypotheses, Experiments,
Findings, Decisions, Artifacts, and Runs that admitted `kaya.event/v0`
records describe. A dossier under `docs/research/experiments/` answers a
research question from that same record.

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
tools' managed blocks — is preserved verbatim. This file checks in no
generated block: `kaya build` appends the `research-index` block on its first
run and rewrites only that block afterwards.
<!-- kaya:end authored -->

<!-- other-tool:begin -->
This block belongs to another tool. kaya must preserve these bytes exactly
across every rebuild; it is here to prove the preservation in acceptance
tests and in real use.
<!-- other-tool:end -->

<!-- kaya:begin generated:research-index -->

### Artifacts
- kaya/Artifact/0cd2fd6 — 0cd2fd6af445980341e9687ca6ebb1b36882ba6c
- kaya/Artifact/2026-11-16-bootstrap-decisions-captured-in-openspec-changes-a — .wai/projects/kaya/designs/2026-11-16-bootstrap-decisions-captured-in-openspec-changes-a.md
- kaya/Artifact/2026-11-16-green-kaya-lb2-clap-cli-init-status-local-genesi — .wai/projects/kaya/designs/2026-11-16-green-kaya-lb2-clap-cli-init-status-local-genesi.md
- kaya/Artifact/2026-11-16-implement-every-ticket-as-one-tdd-ro5u-pipeline-ru — .wai/projects/kaya/designs/2026-11-16-implement-every-ticket-as-one-tdd-ro5u-pipeline-ru.md
- kaya/Artifact/2026-11-16-kaya-motivation-design-documentation-that — .wai/projects/kaya/designs/2026-11-16-kaya-motivation-design-documentation-that.md
- kaya/Artifact/2026-11-16-mvp-1-implementation-contract-orchestrated-2026-1 — .wai/projects/kaya/designs/2026-11-16-mvp-1-implementation-contract-orchestrated-2026-1.md
- kaya/Artifact/2026-11-16-orient-kaya-lb2-crate-scaffold-with-kaya-init-sta — .wai/projects/kaya/research/2026-11-16-orient-kaya-lb2-crate-scaffold-with-kaya-init-sta.md
- kaya/Artifact/2026-11-16-origin-conversation-event-driven-research-docum — .wai/projects/kaya/research/2026-11-16-origin-conversation-event-driven-research-docum.md
- kaya/Artifact/2026-11-16-plan-kaya-lb2-desired-behavior-init-status-envelo — .wai/projects/kaya/plans/2026-11-16-plan-kaya-lb2-desired-behavior-init-status-envelo.md
- kaya/Artifact/2026-11-16-post-bootstrap-alignment-consumers-mvp-1-accep — .wai/projects/kaya/designs/2026-11-16-post-bootstrap-alignment-consumers-mvp-1-accep.md
- kaya/Artifact/2026-11-16-red-kaya-lb2-command-cargo-test-test-cli-expec — .wai/projects/kaya/research/2026-11-16-red-kaya-lb2-command-cargo-test-test-cli-expec.md
- kaya/Artifact/2026-11-16-refactor-kaya-lb2-no-production-refactor-needed — .wai/projects/kaya/designs/2026-11-16-refactor-kaya-lb2-no-production-refactor-needed.md
- kaya/Artifact/43b606f — 43b606f4b64d6efd6e8d7e33c040a0105b1430b7
- kaya/Artifact/510751b — 510751b95220661ef118a3a59d3447e22ae125d3
- kaya/Artifact/59fea0a — 59fea0aec4136495b44e93afefdae9590c49851c
- kaya/Artifact/664b918 — 664b9186c2ce76cc1e8c0bf74771d55bd7b436b0
- kaya/Artifact/6ca120a — 6ca120a3048433384050f786778a908e8b4e8c44
- kaya/Artifact/798376c — 798376ce2a5d223c00a2e59ce71c112d18341b86
- kaya/Artifact/7b5b94d — 7b5b94d0fc78e07d065671db9dcf148420b58724
- kaya/Artifact/904f37f — 904f37fc6fa03785c3df4b48960dda5cf6120d2f
- kaya/Artifact/9a51d1e — 9a51d1ef20e513b85080c751fe949f66eedb9955
- kaya/Artifact/9fb3ea7 — 9fb3ea70195fc96518e358a7954198b8ed2d0e62
- kaya/Artifact/a3ecdc2 — a3ecdc294927a6a915831c4ed7ed73808897970c
- kaya/Artifact/a68206a — a68206a258ed7a67142c9cba0924751904b0d682
- kaya/Artifact/ab319d5 — ab319d59bb12c47e1da09783bb91c4300d61db61
- kaya/Artifact/d3bc2f4 — d3bc2f46f85e61173de0bd7e492b15deaf6361f4
- kaya/Artifact/e178048 — e178048e81017f9667b091311aa7ce912483cd0e
- kaya/Artifact/e4c40ef — e4c40efaeb1c83759d062bde90d65477d2926d38
- kaya/Artifact/e5cf768 — e5cf768fa020c223b7ea533fff65864528703312
- kaya/Artifact/e644f4c — e644f4cc55d33a16dd530cf2a2a0f297983f98e2
- kaya/Artifact/f39a1fd — f39a1fd3da9a6a6861cfacce47294a00404efa81

_0 nodes from 31 events._
<!-- kaya:end generated:research-index -->
