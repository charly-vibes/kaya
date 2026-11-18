---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-rdr-publish-wiring-preparation, pipeline-step:orient]
---

ORIENT kaya-rdr — crates.io publish preparation (approved scope: prepare wiring and steps; maintainer runs the publish)

Ticket: kaya-rdr, P2, complexity:s, lane:hitl, files [".github/workflows/release.yml"]. Claims: a maintainer sets CARGO_REGISTRY_TOKEN in repository secrets and an authorized tag publishes kaya; the release workflow crates.io channel stops reporting disabled.

State found:
- The channel is already wired: `.github/workflows/release.yml` publishes in the `publish` job after `release` succeeds, token-gated on CARGO_REGISTRY_TOKEN, writing `disabled` when absent and running `cargo publish --allow-dirty --locked` when present; the release-health step reports it.
- The crate itself is not publishable as-is: `cargo package --list --allow-dirty` returns 482 files, dominated by .wai (250), .espectacular (105), openspec (27), docs (20) and agent/tooling directories (.agents, .beads, .codex, .cursor, .claude, .kaya, scripts, .github, theme, book). Cargo also warns "manifest has no description".
- Cargo.toml has name/version/edition/rust-version/license/repository but no description, readme, keywords, categories or exclude list.

Acceptance restated as checkable statements:
1. `cargo package --list` includes only crate sources, tests, Cargo metadata, LICENSE, README and the release runbook.
2. `cargo publish --dry-run --allow-dirty` passes without network-side surprises (or its remaining failure is documented as credential-only).
3. The exact maintainer steps (create token, store secret, tag, verify, rollback) exist in one place and are referenced from the workflow.
4. No tag is created, no secret is set, nothing is pushed: those remain human release actions.

Out of scope: kaya-dgz registry wiring (blocked by this ticket), any release execution.

