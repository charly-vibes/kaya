---
tags: [pipeline-run:tdd-ro5u-2026-11-16-kaya-htk-release-workflow, pipeline-step:plan]
---

PLAN kaya-htk: minimal TDD plan.

Desired behavior: standardization §1 release slot (see ORIENT acceptance 1-8).
Out of scope: tag creation, push, crates.io authorization, ddl registry PR, badge row
beyond Release.

Test cases first (tests/release_scripts.rs, python3-driven, tempdir fixtures):
1. homebrew_renders_from_checksums — formula carries class Kaya, version, license
   Apache-2.0, four release URLs and their sha256 digests, bin.install "kaya".
2. homebrew_rejects_missing_platform — checksums lacking darwin_arm64 => nonzero
   exit, error names the missing platform, formula not written.
3. scoop_renders_from_checksums — manifest version/url/hash/bin/autoupdate correct.
4. scoop_rejects_missing_windows_checksum — nonzero exit, manifest not written.
5. both_scripts_are_idempotent — second render byte-identical (retry safety, DDL-9e7).

Narrow command: cargo test --test release_scripts
Full verification: just ci + actionlint .github/workflows/release.yml + a local
throwaway simulation of the packaging/provenance steps with fake bin/ payloads
(workflow itself only runs on a v* tag; that run is not part of this ticket).

Files: .github/workflows/release.yml, scripts/update-homebrew.py,
scripts/update-scoop.py, tests/release_scripts.rs, README.md, docs/src/status.md,
CHANGELOG.md.

Commit shape: single feature commit; no refactor commit expected (new files only).
