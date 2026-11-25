---
tags: [pipeline-run:tdd-ro5u-2026-11-25-kaya-g9u-split-main-rs-into-command-modules-waves-tighten-pretender-ceiling, pipeline-step:refactor]
---

REFACTOR kaya-g9u: the ticket IS a refactor; no further tidy pass. Post-split tidiness audit: main.rs 859 lines retains envelope plumbing, clap tree, dispatch match, shared helpers - no dead code introduced (all pub(crate) helpers have live consumers: read_build_ledger/collect_document_ir still called from commands/publication.rs; human_actor from verification+doctest+impact). No second convention added: modules follow existing src/capture.rs pattern. Oracle: cargo test 504/0 (re-run after refactor step).
