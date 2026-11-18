---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-rdr-publish-wiring-preparation, pipeline-step:plan]
---

PLAN kaya-rdr — publish preparation

Steps (TDD order):
1. RED: record the failing meter — `cargo package --list --allow-dirty` = 482 files across tooling trees; manifest warning "no description". No production test covers packaging, so the meter is the cargo command itself, run before and after.
2. GREEN (metadata): add description, readme, keywords, categories and an explicit `exclude` list to Cargo.toml so the tarball carries only src/, tests/ and the repo metadata.
3. GREEN (runbook): add RELEASING.md with the exact maintainer steps — create the crates.io token, store CARGO_REGISTRY_TOKEN in the repository secrets, tag and push the authorized tag, verify `cargo search kaya`/the release channel report, and what to do if the token is revoked; reference it from the workflow header comment in place of the "authorization outstanding" note.
4. Verify: `cargo package --list` small and reviewed; `cargo publish --dry-run --allow-dirty` (documented outcome); `cargo test`; `just ci`; mdbook unaffected.

Out of scope: running the publish, setting secrets, tagging, kaya-dgz.

Files: Cargo.toml, RELEASING.md (new), .github/workflows/release.yml (comment only).

Commit shape: one commit for metadata + runbook + comment; evidence commit last.

