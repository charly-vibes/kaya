---
tags: [pipeline-run:tdd-ro5u-2026-11-16-kaya-htk-release-workflow, pipeline-step:orient]
---

ORIENT kaya-htk: Release workflow slot (standardization §1).

Ticket: kaya-htk (task, P2, claimed). files: none declared; base_commit none.
Spec mapping: the openspec change add-core-research-record covers the six research
capabilities; release/CI infrastructure has no openspec scenario by design. Authority
for this ticket is dulce-de-leche docs/standardization.md §1 (3 workflow files, five-
target matrix, naming {tool}_{VERSION}_{os}_{arch}, checksums.txt, tap+bucket
auto-update) plus reference implementations ddl and wai release.yml. Precedent:
kaya-vt9 (CI workflow) shipped through this pipeline with cargo-test acceptance.

Testable acceptance statements:
1. .github/workflows/release.yml exists, name Release, triggers only on push tags v*.
2. Builds exactly five targets: x86_64/aarch64-unknown-linux-gnu, x86_64/aarch64-
   apple-darwin, x86_64-pc-windows-msvc.
3. Produces kaya_{VERSION}_{linux_amd64,linux_arm64,darwin_amd64,darwin_arm64}.tar.gz,
   kaya_{VERSION}_windows_amd64.zip, checksums.txt, provenance.json.
4. publish job needs all three build jobs plus a test job that runs cargo test --locked
   on the tagged commit; workflow default permissions contents: read, publish job
   contents: write; all third-party actions pinned to full commit SHAs.
5. scripts/update-homebrew.py renders a Formula with per-platform url+sha256 from
   checksums.txt; scripts/update-scoop.py renders a manifest from the windows sha;
   both fail loudly when a required checksum is missing, and re-render identical bytes
   for an already-updated channel (idempotent retry).
6. Tag/Cargo.toml version mismatch aborts before publishing.
7. crates.io publish is token-gated (no token => explicit notice, not a failed release)
   because publish authorization is outstanding (kaya-dgz).
8. Repo docs reflect the new slot: README Release badge, docs/src/status.md, CHANGELOG.

Out of scope: publishing/tagging, crates.io authorization, ddl registry PR (kaya-dgz),
Hegel tickets (kaya-152/kaya-nx4), badge-row completion beyond the Release badge.

Constraints: no push; action SHAs reused from ci.yml (checkout 11d5960, rust-toolchain
686976e, rust-cache 6323deb, upload ea165f8 v4.6.2, download d3f86a1 v4.3.0); lefthook
runs via global ~/.git-hooks shim.
