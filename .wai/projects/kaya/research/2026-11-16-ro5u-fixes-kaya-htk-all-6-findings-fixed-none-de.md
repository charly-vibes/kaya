---
tags: [pipeline-run:tdd-ro5u-2026-11-16-kaya-htk-release-workflow, pipeline-step:fix-review]
---

RO5U-FIXES kaya-htk: all 6 findings fixed, none deferred.
H1: cargo publish --allow-dirty --locked; bins/ and dist/ added to .gitignore (cargo package --list shows 0 entries from them; dry-run packages+verifies at 283 files).
M2: release step given id: release; homebrew, scoop and crates steps all gated if: !cancelled() && steps.release.outcome == 'success'; order is release -> homebrew -> scoop -> crates -> health, so a crates failure cannot suppress tap/bucket and every channel reports independently in the summary table.
M3: Create GitHub release now does gh release view -> upload --clobber, else create with the seven assets; a re-run repairs a partial release instead of dying on 'already exists'.
M4: scoop autoupdate url templated (v$version/kaya_$version_windows_amd64.zip) matching every scoop-charly manifest; test asserts the templated form.
L5: both scripts reject tag != 'v'+version before writing; new test both_scripts_reject_tag_version_mismatch covers both.
L6: homebrew test now asserts each platform url is followed by its own sha256 (indentation-agnostic), so a transposed mapping fails.
Verification: cargo test --test release_scripts 7/7; just ci green; actionlint -shellcheck=shellcheck clean; ruby -ryaml structural assertions on step order, ids and gates; cargo publish --dry-run --allow-dirty --locked packages and verifies; local packaging simulation renders formula (ruby -c OK) and templated scoop manifest.
