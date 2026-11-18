---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-cc9-archive-and-deploy-document-taxonomy, pipeline-step:fix-review]
---

RO5U-FIXES kaya-cc9 — review findings resolved

CORR-001 fixed: deleted docs/src/specs/4045993name.md, the 6-byte residue of the failed inline recipe.
CORR-002 fixed: scripts/sync-specs.py now prefixes every page with an ownership marker and prunes stale marked pages whose capability disappeared, while leaving unmarked files untouched.
CLAR-001 fixed: generated pages carry a visible note naming the specodelic layer sidecar that follows the spec.
EDGE-001 (freshness gate) deferred with reason: a `git diff --exit-code` gate would make every dirty local tree fail `just ci`; `just docs` regenerates on every build, so the book output is never stale.
EDGE-002 and EXCL-001 accepted as documented (skip-warning path; intentional divergence between archived delta and deployed spec).

Verification: just ci passed (test, lint, docs, ah check, spk lint, openspec changes strict); ah check --run-tests 36 passed / 0 findings; ah report 35/35 covered; spk lint openspec/specs 2 files 0 issues; openspec validate --specs --strict 2 passed.
