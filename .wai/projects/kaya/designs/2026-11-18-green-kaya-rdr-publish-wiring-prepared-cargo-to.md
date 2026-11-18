---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-rdr-publish-wiring-preparation, pipeline-step:green]
---

GREEN kaya-rdr — publish wiring prepared

Cargo.toml: added description, readme, keywords, categories and an explicit exclude list covering the research workspace and tooling trees (.agents, .beads, .claude, .codex, .cursor, .espectacular, .github, .kaya, .wai, .whisper, book, book.toml, docs, justfile, lefthook.yml, openspec, scripts, theme, update-homebrew.py, update-scoop.py, versions.ddl.toml, .genesis, .gitignore, AGENTS.md, CLAUDE.md, RELEASING.md).

RELEASING.md (new): the exact human steps — token creation and scopes, `gh secret set CARGO_REGISTRY_TOKEN`, pre-tag packaging checks, tag/version verification, the authorized tag push, how to read the channel-propagation table, `cargo search kaya` confirmation, retry behavior, version yanking, and the rule that deleting a tag does not roll back crates.io or GitHub releases.

.github/workflows/release.yml: header comment now points at RELEASING.md instead of the outstanding-authorization note; the crates.io step itself is unchanged (token-gated, reports disabled, publishes with `cargo publish --allow-dirty --locked`).

README.md: the crates.io deferral sentence now states the channel is wired but gated on maintainer authorization, and links RELEASING.md.

Verification:
- `cargo package --list --allow-dirty` = 44 files: src/ 16, tests/ 21, CHANGELOG.md, Cargo.lock, Cargo.toml(.orig), LICENSE, README.md, .cargo_vcs_info.json — down from 482.
- `cargo publish --dry-run --allow-dirty` packaged 44 files (172 KiB compressed), verified by compiling, and aborted only at the upload step as a dry run.
- `cargo test` 24 targets green; `just ci` green; mdbook unaffected.

Not done, by design: no secret set, no tag created, no push, no publish — those remain authorized human release actions.

