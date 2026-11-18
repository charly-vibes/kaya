---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-cc9-archive-and-deploy-document-taxonomy, pipeline-step:refactor]
---

REFACTOR kaya-cc9 — tidy pass review

Applied: none beyond what green already required. `just ci` re-ran after the deployment edits (test, lint, docs, validate, spec-lint, spec-validate) and passed.

Candidates examined and rejected:
- Rewriting scripts/sync-specs.py around pathlib.partition: the current explicit index math is the clearer form for a 20-line script and is covered by running `just specs-docs`.
- Adding docs/src/specs to .gitignore: the pages are compiled docs like docs/research and docs/public, which this repository commits; dropping them would break a plain `mdbook build` that does not go through just.
- Collapsing the archived delta edits: the deployed spec and the archived delta now agree on the renamed scenario, which is the point; no duplication to remove.
No dead code, no unused contract files (`ah check` reports no orphan-toml).
