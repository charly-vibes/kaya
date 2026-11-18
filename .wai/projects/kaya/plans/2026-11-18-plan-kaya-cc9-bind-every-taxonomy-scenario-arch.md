---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-cc9-archive-and-deploy-document-taxonomy, pipeline-step:plan]
---

PLAN kaya-cc9 — bind every taxonomy scenario, archive, deploy

Desired end state: ah check --changes add-document-taxonomy --run-tests reports 0 structural with every selector passing; openspec/specs holds kaya-documents/ and kaya-publications/; spk lint openspec/specs is clean; docs/src/SUMMARY.md Design specs names the deployed layers.

Out of scope: kaya-n50 editorial cutover, kaya-rdr publish authorization, kaya-aaz DocIR design thread; those tasks stay unchecked in tasks.md and are open tickets.

Steps (TDD order):
1. RED: record the failing meter — `ah check --changes add-document-taxonomy --run-tests` = 0 passed / 19 structural (no-toml). No production test is missing: each of the 19 scenarios is already executed by an existing test (mapping recorded in ORIENT).
2. GREEN (bind): author 19 contract files under .espectacular/changes/add-document-taxonomy/{kaya-documents,kaya-publications}/ using the deployed format, one scenario id each, cargo flags = the mapped test function. Verify `ah check --changes add-document-taxonomy --run-tests` reports 0 structural and every selector passes.
3. GREEN (strict): if `openspec validate add-document-taxonomy --type change --strict` still fails on the two long-requirement warnings, fold the enumerations/examples out of the requirement prose into the scenarios it already has (no behavior change; the warning names the exact requirements).
4. GREEN (archive): `openspec archive add-document-taxonomy --yes`; confirm openspec/specs/{kaya-documents,kaya-publications}/ appear with spec.md + layer.md and that ah check (deployed) still reports no issues.
5. GREEN (deploy docs): update docs/src/SUMMARY.md Design specs to list the deployed layers (kaya-documents, kaya-publications plus the six from the first change if they landed) instead of pointing at change deltas.
6. GREEN (lint): `spk lint openspec/specs` clean.

Narrow command: ah check --changes add-document-taxonomy --run-tests
Full verification: cargo test; ah check; cargo run -- build/publish idempotent (second run recomputed []); openspec validate --specs --strict after deployment; spk lint openspec/specs; just ci.

Files: .espectacular/changes/add-document-taxonomy/**/*.toml (19 new), openspec/changes/add-document-taxonomy/specs/{kaya-documents,kaya-publications}/*.md (only if strict warnings require prose edits), openspec/specs/** (archive output), openspec/changes/archive/** (archive output), docs/src/SUMMARY.md, openspec/changes/add-document-taxonomy/tasks.md (archive checklist + 3.9/3.10 status honesty).

Commit shape: one commit for bindings + strict fixes; one commit for the archive/deploy output + SUMMARY if the archive tool rewrites many files; evidence commit last.
