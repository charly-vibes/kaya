---
tags: [pipeline-run:tdd-ro5u-2026-11-23-kaya-8f4-publication-block-vocabulary, pipeline-step:ship-close]
---

SHIP kaya-8f4: commit f965984354 on main (32 paths: src/publication/mod.rs validator + two plan-path call sites, tests/publications.rs three new cases, the archived change add-publication-block-vocabulary with proposal/tasks/spec delta/layer sidecar, the deployed kaya-publications spec.md + layer.md, the deployed and overlay .espectacular bindings, docs/src/document-taxonomy.md, docs/src/specs/kaya-publications.md regenerated, .beads/issues.jsonl export). kaya-8f4 closed with bd.

Final verification: cargo test 379 tests passed 0 failed across 32 suites; cargo clippy --all-targets --all-features -- -D warnings clean; cargo fmt --all --check clean; ah check --run-tests 184 passed 0 findings; openspec validate --specs --strict 9 passed 0 failed; openspec validate --archived --strict - change/2026-11-23-add-publication-block-vocabulary passes; spk lint openspec/specs 0 issues; just specs-docs 9 pages; just links 36 pages 0 unresolved.

No push: fleet policy requires explicit user authorization for push, deploy, or release. Proposed: git push origin main.
