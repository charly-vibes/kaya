---
sut_path: /var/home/sasha/para/areas/dev/gh/charly/kaya
commit: cb8be97696b59f99909ba7f39a10daa2819294bc
updated: 2026-11-19
external_references:
  - path: /var/home/sasha/para/areas/dev/gh/charly/reports/hegel-eval-2026-11-16/README.md
    why: Suite-level screening baseline for kaya.
---

# Existing assertions

No property-based testing framework is in use today (no proptest, no Hegel).
The assertion surface is:

- **Contract tests** (`tests/*.rs`): one test function per deployed spec
  scenario, bound 1:1 from `.espectacular/<capability>/<scenario>.toml` by
  test-function name; `ah check --run-tests --all-tests` executes the bindings
  (149 at the pinned commit). Example-territory: fixed fixtures, deterministic
  assertions.
- **Integration drivers**: `assert_cmd` + `tempfile`, real binary, tempdir
  ledgers (`tests/cli.rs`, `tests/cli_pipeline.rs`, `tests/turu.rs`).
- **Codec/model unit tests** in-crate (`src/event.rs`, `src/model.rs`).
- **`kaya check`**: read-only byte-level drift verification — an oracle, not a
  property engine.

Gap this run addresses: generated-input properties (unicode/boundary
serialization, correction-chain shapes, ledger mutation sequences) and
differential envelope properties (status vs build digest) have no coverage.
