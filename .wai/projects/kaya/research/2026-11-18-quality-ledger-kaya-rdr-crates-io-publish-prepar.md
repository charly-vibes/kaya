---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-rdr-publish-wiring-preparation, pipeline-step:quality-ledger]
---

QUALITY LEDGER kaya-rdr — crates.io publish preparation

Purpose: make the crate publishable and make the human authorization path explicit, without performing any release action.

Changed:
- Cargo.toml: description, readme, keywords, categories; explicit exclude list for the research workspace, agent configuration, spec corpus, book, release tooling and maintainer docs.
- RELEASING.md (new): token creation/scopes, secret storage, pre-tag packaging checks, tag/version verification, authorized tag push, channel-propagation reading, search confirmation, retry semantics, yank procedure, and the limits of deleting a tag.
- .github/workflows/release.yml: header points at RELEASING.md; the crates.io step is unchanged (token-gated, `disabled` when absent).
- README.md: crates.io status sentence and a Releasing link.

Verified: `cargo package --list --allow-dirty` 44 files (src 16, tests 21, CHANGELOG, Cargo.lock, Cargo.toml(.orig), LICENSE, README, .cargo_vcs_info) down from 482; `cargo publish --dry-run --allow-dirty` packaged 44 files, compiled, aborted at upload as designed; cargo test 24 targets; just ci green.

Review: RO5U READY — Critical 0, High 0, Medium 0, Low 4 (documented/deferred). Convergence at stage 4; false positives 0.

Risks: the exclude list must grow with any new tooling directory; the acceptance criterion (channel stops reporting disabled) is unverifiable until a maintainer stores CARGO_REGISTRY_TOKEN and pushes an authorized tag.

Next: commit the preparation; kaya-rdr stays OPEN (its acceptance is maintainer-side), with the remaining steps named on the ticket.

