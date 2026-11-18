---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-a4e-kaya-check-index, pipeline-step:quality-ledger]
---

QUALITY LEDGER kaya-a4e
Changed: src/verification/mod.rs (Source enum, worktree/index read paths, git index plumbing, mode-aware ledger with fail-closed HEAD baseline, one-call stage parsing); src/main.rs (--index); tests/verification.rs (5 scenario tests + ledger-index and stray-index coverage); five TOMLs.
Verified: cargo test --test verification 18/18; cargo test all suites; clippy -D warnings; fmt; ah check --changes add-record-verification --run-tests 50 passed / 10 no-toml (facts, assertions, stale slices); dogfood `kaya check --index` empty drift; just ci green at 585 and affected gates re-run here.
Review: RO5U 0 Critical, 0 High, 2 Medium, 2 Low; all fixed.
Risks: facts/assertions/staleness still later slices; hooks/CI not wired; index mode spawns git per read (acceptable for hook-sized records).
Next: kaya-8ot facts and canonical vocabulary.

