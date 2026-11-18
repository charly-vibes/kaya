---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-cc9-archive-and-deploy-document-taxonomy, pipeline-step:red]
---

RED kaya-cc9 — the archive meter fails before the bindings exist

Narrow command: `ah check --changes add-document-taxonomy --run-tests`
Result: 15 passed, 19 structural (all no-toml), 0 execution.
Expected failure: each of the 19 scenario ids lacks a contract file, exactly the no-toml finding the archive checklist forbids. No production test is missing — every scenario already has an executing test (mapping in ORIENT); the defect is missing contract metadata plus the unarchived change.
No setup breakage: the 13 pre-existing contracts and the lh1 contract still execute (15 selectors passed).
