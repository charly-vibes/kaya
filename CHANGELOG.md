# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Repository bootstrapped on the dulce-de-leche fleet standard: wai tracking,
  beads issue tracking (no-db, tracked export), OpenSpec, lefthook gates
  (`ah check`, `pretender`, `spk lint`), LICENSE (Apache-2.0), and fleet README.
- Six capability specs (`add-core-research-record`: event, ledger, model,
  projection, promotion, adapters) — dual-format openspec deltas + specodelic
  layers, lint-clean, strict-validated.
- mdBook scaffolding (`book.toml`, `theme/charly.css`, `SUMMARY.md`).
- Rust CLI `kaya` (clap derive): `init`, `status`, `import wai|git|test`,
  `build [--out] [--expect-digest]`, `promote`, `challenge`; each executed
  command prints one genesis-style JSON envelope and failures carry a labeled
  class.
- Event layer: `kaya.event/v0` parsing, the closed 9-field schema, actor and
  provenance handling (`src/event.rs`).
- Ledger: append-only JSONL with SHA-256 digest anchors, verified reads,
  crash-truncated tail recovery, stored-line dedupe, and supersede/correction
  resolution (`src/ledger.rs`).
- Research model: 13 entity types, 9 typed direction-fixed relationships,
  graph validation (`src/model.rs`).
- Projections: research index and experiment reports, generated-block
  ownership, verbatim preservation of foreign blocks, stale authored-section
  detection (`src/projection/`).
- Promotion: proposal/approval/challenge replay with pending counts; CLI
  promotion commands require a named human `--actor` (`src/promotion.rs`).
- Adapters for the `.wai/` artifact tree, git history, and libtest JSONL run
  reports (`src/adapters/`).
- `tdd-ro5u` wai pipeline config (`orient` → `plan` → `red` → `green` →
  `refactor` → `ro5u-review` → `fix-review` → `quality-ledger` →
  `ship-close`), gated on passing tests and human approval.
- CI workflows: `ci.yml` runs `just ci` with pinned tools; `docs.yml` builds
  the mdBook and deploys it to GitHub Pages; `justfile` recipes for
  build/test/lint/docs/spec gates.
- Dogfooded dossier: `.kaya/ledger.jsonl` plus the `docs/research/`
  projections (`index.md`, `experiments/add-core-research-record.md`) are
  checked in and rebuilt by `kaya build`.

### Changed
- Docs review pass (Rule of 5, TypeSafe-verified findings): README de-
  duplicated in favor of the design doc, spec suite linked from README/status,
  projection grammar corrected (two block forms, proposals are a status),
  capture→preserve→understand→reveal labels unified across docs, FAQ added,
  data-outlives-kaya guarantee and agent-session-recording rejection stated,
  MVP-1 milestone tense aligned with the spec change.
- README and status now describe the shipped MVP-1 CLI (command surface,
  `.kaya/ledger.jsonl`, `docs/research/` projections, human-approved
  promotion) instead of a design-only phase; install section added for
  `cargo install --git`.
