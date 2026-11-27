# RO5U Review — kaya-haz (genesis envelope+cli adoption)

Diff reviewed: Cargo.toml, Cargo.lock, src/envelope.rs, src/main.rs, tests/cli.rs.
Pre-existing dirty docs/src/specs/{kaya-impact,kaya-projection}.md excluded — not this ticket.

## Convergence
CONVERGED at stage 2 (0 new criticals, issue rate <10% thereafter). Clippy --all-targets clean.

## Findings
- [EDGE-001] LOW — src/main.rs:332: `--version` combined with `--json`/`-j` is intercepted
  pre-parse, so `kaya <subcmd> --version --json` that previously produced a labeled usage
  error envelope now prints a version envelope with exit 0. No kaya subcommand defines
  `--version`, so no legitimate invocation is shadowed; this is the fleet-wide
  `genesis::cli` convention (podras, wai, dont wire the identical helper). Deferred: no fix.
- [EXCL-001] LOW — custom Serialize uses `serialize_entry(&"ok", …)`; `"ok"` would do.
  Not flagged by clippy; cosmetic. Deferred.

## Verified invariants
- Wire field order and shapes unchanged: ok, envelope_version("0.1"), cli_version,
  envelope_kind (free-form command names), data, warnings([]), hints([]), meta({}),
  error{label,message} omitted on success — existing assertions in tests/cli.rs:31-33,
  tests/capture.rs, tests/acceptance.rs:913 pass UNMODIFIED.
- ok invariant owned by genesis (`ok_for`: Ok→true, Error→false); emit() maps is_ok() to exit codes.
- Full `cargo test` green (~290 tests across all suites).

## Verdict: READY
