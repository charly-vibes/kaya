# envelope-exit-code-agreement — one envelope per invocation; exit code ⇔ `ok`

Evidence file for [`property-catalog.md`](../property-catalog.md) (P3). Repo kaya @ `cb8be97696b59f99909ba7f39a10daa2819294bc`.

**Disposition (pinned):** asserted within the same subprocess state machine as status-build-parity (`tests/status_build_properties.rs`, nx4) — free on every spawn, no separate property cost.

| | |
|---|---|
| **Type** | Safety (CLI contract) |
| **Invariant** | For every invocation routed through `emit` (src/main.rs:288-304) — i.e. every command except `check` — stdout is exactly one pretty-JSON envelope carrying `ok`, `envelope_version`, `cli_version`, `envelope_kind`, `data`, `warnings`, `hints`, `meta` (src/envelope.rs:27-46, printed with one trailing newline, envelope.rs:63-70), error envelopes additionally carry a non-empty `error.label` (envelope.rs:19-25), and **`process.exit == 0 ⇔ ok == true`** (src/main.rs:289-303: success-print → `ExitCode::SUCCESS` only when `envelope.ok`; any print error → `FAILURE`). |
| **Check mode** | Subprocess; the same state machine as status-build-parity asserts it for free on every spawn. |
| **Seam** | None needed; pure oracle over `(exit code, stdout bytes)`. |

## Generator sketch

Reuse the ledger-mutation machine from [`status-build-parity.md`](status-build-parity.md); add argv mutations (missing subcommand, unknown flag, `--expect-digest` not-hex) to hit the usage-error envelopes (src/main.rs:155-176).

## Predicted red/green

**Green as scoped**: the single writer `emit` enforces the biconditional mechanically.

**Deliberately out of scope**: `check`, which uses `emit_code` (src/main.rs:306-318, envelope at :844-865) and by design returns an `ok: true` envelope with exit 1 on drift and exit 2 on usage/IO — catalog it as a separate `check/verdict-exit-mapping` property if wanted.

## Open questions

- None.
