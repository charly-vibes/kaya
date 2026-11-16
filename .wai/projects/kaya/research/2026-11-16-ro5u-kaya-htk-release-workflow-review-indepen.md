---
tags: [pipeline-run:tdd-ro5u-2026-11-16-kaya-htk-release-workflow, pipeline-step:fix-review]
---

# RO5U kaya-htk — release workflow review

Independent reviewer + local probes; authority = dulce-de-leche docs/standardization.md §1
and the ddl/wai reference workflows. Verdict: incorrect (confidence 0.85).

## Critical
None.

## High
- **H1** `.github/workflows/release.yml:310` — `cargo publish --locked` aborts on the job's own
  untracked `bins/` and `dist/` trees (cargo treats untracked files as dirty; reproduced locally).
  Once `CARGO_REGISTRY_TOKEN` exists every release fails here and tap/bucket are skipped.
  Fix: `cargo publish --allow-dirty --locked`, and ignore `bins/` + `dist/` so `cargo package`
  cannot ship downloaded binaries.

## Medium
- **M2** `release.yml:318-322` — tap/bucket steps inherit `success()` from the crates step, so a
  crates failure silently skips the mandatory channels while the health grep reports nothing.
  Fix: id the release step; gate all channel steps on `!cancelled() && steps.release.outcome ==
  'success'`; run crates last so it cannot suppress tap/bucket.
- **M3** `release.yml:276` — re-running a partially failed publish dies at `gh release create`
  (`already exists`), so documented idempotent retries are unreachable.
  Fix: `gh release view` → `upload --clobber`, else `create`.
- **M4** `scripts/update-scoop.py:48` — `autoupdate.url` reuses the concrete-version URL, so
  checkver bumps `version` while the URL keeps the old asset; every scoop-charly manifest uses
  `v$version/<tool>_$version_windows_amd64`. Fix: templated URL + test update.

## Low
- **L5** both scripts never check `tag == "v" + version` although `release.yml:315-317` claims
  they do; a mismatched call writes a dead URL and exits 0. Fix: guard + test.
- **L6** `tests/release_scripts.rs:84-92` asserts each digest appears somewhere in the formula,
  so a platform/digest transposition still passes. Fix: assert url→sha256 pairing per platform.
