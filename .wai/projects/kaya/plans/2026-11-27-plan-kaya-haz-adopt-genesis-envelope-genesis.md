---
tags: [pipeline-run:tdd-ro5u-2026-11-27-kaya-haz-adopt-genesis-envelope-and-cli, pipeline-step:plan]
---

PLAN kaya-haz: adopt genesis::envelope + genesis::cli with wire-identical thin wrapper.

DESIGN
- Cargo.toml: add genesis-vibes = "0.15" (crates.io, matches wai's pin).
- src/envelope.rs becomes a thin wrapper (dont/src/envelope.rs precedent): kaya Envelope holds genesis::envelope::Envelope<Value> as inner (genesis owns ok invariant, envelope_version/cli_version stamping, data, Meta) plus kaya-owned deltas the genesis closed enum cannot express: free-form kind &'static str and error{label,message}. Custom Serialize emits the exact 0.1 wire: ok, envelope_version(0.1), cli_version, envelope_kind, data, warnings([]), hints([]), meta({}), error. Public API unchanged: Envelope::ok(kind,data), Envelope::error(kind,label,message), print(); ~30 call sites untouched.
- src/main.rs: wire genesis::cli::maybe_print_version_json("kaya", version) before Cli::parse (podras/wai precedent; no completions subcommand — podras ships without one).

SPEC SCENARIOS
- kaya.event.envelope_independence (extension_point): CLI I/O uses genesis envelopes while ledger rows stay kaya.event/v0 — delivered by this adoption; ledger rows covered by existing event tests.

OUT OF SCOPE
- kaya-bov (genesis::feedback), completions subcommand, migrating warnings/hints/meta to genesis typed fields (would change 0.1 wire), receipt/ephemeral, cli_envelope_fixture() new unit test (ledger-row independence already covered by tests/event.rs).

RED FIRST (tests/cli.rs, new test run_version_json_prints_envelope)
- kaya --version --json => exit 0, one JSON line: ok==true, envelope_kind=="version", data.name=="kaya", data.version=="0.4.1" (shape per genesis::cli print_version_envelope; new surface, no existing consumers).
- Fails today: clap errors on unknown --json before genesis dep exists.

NARROW CHECK: cargo test --test cli
FULL VERIFY: cargo test (whole suite: existing envelope-shape assertions in tests/cli.rs:31-33, capture.rs, acceptance.rs:913 are the unchanged-shape net; ledger rows by tests/event.rs)

FILES: Cargo.toml, Cargo.lock, src/envelope.rs, src/main.rs, tests/cli.rs

COMMITS: one feature commit (genesis envelope+cli adoption), no separate tidy — the wrapper replaces envelope internals in place.
