---
tags: [pipeline-run:tdd-ro5u-2026-11-27-kaya-haz-adopt-genesis-envelope-and-cli, pipeline-step:quality-ledger]
---

QUALITY LEDGER kaya-haz: 

CHANGED:
- Cargo.toml/Cargo.lock — add genesis-vibes 0.15 dependency (crates.io)
- src/envelope.rs — thin wrapper: Envelope holds genesis::envelope::Envelope<Value> (ok invariant, version stamps, data); kaya keeps 0.1 wire deltas (free-form kind, error{label,message}, empty warnings/hints/meta); custom Serialize emits pre-adoption shape; is_ok() accessor replaces removed ok field
- src/main.rs — genesis::cli::maybe_print_version_json hook before clap parse
- tests/cli.rs — red-first test version_json_prints_version_envelope

VERIFIED:
- cargo test: all suites green (~290 tests; existing shape assertions in tests/cli.rs:31-33, capture.rs, acceptance.rs:913 unmodified)
- cargo clippy --all-targets: clean
- ah check: 0 structural findings; ah check --run-tests: OK 235 passed, 0 findings
- openspec validate --all: 13 passed, 0 failed

REVIEW:
- RO5U converged stage 2, 0 criticals/high/medium; 2 LOW deferred (EDGE-001: --version+-json interception pre-parse, fleet-wide genesis::cli convention, no subcommand defines --version; EXCL-001 cosmetic serialize_entry key reference). Verdict READY. Artifact: .wai/projects/kaya/reviews/ro5u-kaya-haz-findings.md

RISKS:
- none known beyond deferred LOWs; wire unchanged so fleet consumers see identical envelopes

NEXT:
- ship-close: human approval gate, atomic feature commit (code+tests+Cargo+wai artifacts), close kaya-haz, then start kaya-bov run
