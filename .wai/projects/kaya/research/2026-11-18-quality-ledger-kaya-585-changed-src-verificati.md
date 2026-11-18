---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-585-kaya-check, pipeline-step:quality-ledger]
---

QUALITY LEDGER kaya-585

Changed:
- src/verification/mod.rs (new): read-only check core; worktree Source; root-anchored committed-ledger verification; expected research + publication bytes via shared planners; block/document comparison; deduped sorted drift.
- src/projection/{index,report,reuse}.rs: pure plan functions beside write wrappers; Planned/PlannedDocument types.
- src/publication/mod.rs: default_declarations, read_documents_from, closure-based plan; build applies plan.
- src/main.rs: Check subcommand (--root/--out/--publish-out/--expect-digest), emit_code, check usage/IO exit 2.
- src/lib.rs: pub mod verification.
- tests/verification.rs: 11 integration tests incl. authored/foreign preservation and publication drift/missing.
- .espectacular/changes/add-record-verification/kaya-verification/*.toml: 9 scenario contracts bound to tests.

Verified:
- cargo test --test verification: 11 passed.
- cargo test: all suites passed.
- cargo clippy --all-targets --all-features -- -D warnings: clean.
- cargo fmt --all --check: clean.
- just ci: passed (fmt, clippy, mdbook, ah check, spk lint, openspec validate --strict).
- ah check --changes add-record-verification --run-tests: 45 passed, 15 no-toml (owned by a4e/8ot/ntv).
- kaya check on this repository: {"drift":[],"stale":[]}, exit 0.

Review:
- RO5U: 0 Critical, 1 High, 2 Medium; all fixed and re-verified. Residual note: a stale omitted publication page that also carries authored bytes can produce a block changed plus a document changed finding (no duplicate identical entries).

Risks:
- --index, facts, assertions, staleness enforcement are the next slices; check currently reports no stale/facts.
- Hooks/CI do not call check yet (kaya-3kg/tm8).

Next:
- kaya-a4e: --index staged mode and check_io failures.

