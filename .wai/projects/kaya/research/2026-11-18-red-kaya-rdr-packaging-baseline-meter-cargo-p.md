---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-rdr-publish-wiring-preparation, pipeline-step:red]
---

RED kaya-rdr — packaging baseline

Meter: `cargo package --list --allow-dirty`
Result before: 482 files. Top-level counts: .wai 250, .espectacular 105, openspec 27, tests 21, docs 20, src 16, .agents 9, scripts 3, .github 3, .kaya 2, .cursor 2, .codex 2, .beads 2, plus theme, book.toml, justfile, lefthook.yml, versions.ddl.toml.
Cargo also warns: "manifest has no description".

Expected failure: a published crate would carry the entire research workspace, agent configuration and spec corpus, and the registry listing would have no description. No test exists for packaging (and none should pin tarball contents beyond the exclusion invariant), so the cargo command is the meter; engine tests stay untouched.

