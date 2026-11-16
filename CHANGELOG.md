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

### Changed
- Docs review pass (Rule of 5, TypeSafe-verified findings): README de-
  duplicated in favor of the design doc, spec suite linked from README/status,
  projection grammar corrected (two block forms, proposals are a status),
  capture→preserve→understand→reveal labels unified across docs, FAQ added,
  data-outlives-kaya guarantee and agent-session-recording rejection stated,
  MVP-1 milestone tense aligned with the spec change.
