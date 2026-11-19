---
sut_path: /var/home/sasha/para/areas/dev/gh/charly/kaya
commit: cb8be97696b59f99909ba7f39a10daa2819294bc
updated: 2026-11-19
external_references:
  - path: /var/home/sasha/para/areas/dev/gh/charly/reports/hegel-eval-2026-11-16/README.md
    why: Suite-level Hegel adoption verdict and hands-on engine facts (build times, ci profile, shrink quality).
  - path: /var/home/sasha/para/areas/dev/gh/charly/antithesis/scratchbook/property-catalog.md
    why: House per-property dossier format and the screening-depth kaya entries this run re-derives at a pinned commit.
  - path: https://github.com/antithesishq/antithesis-skills
    why: antithesis-research method and scratchbook conventions.
  - path: https://hegel.dev/
    why: Property-based testing engine under recommendation (hegeltest 0.52.7, static-engine).
---

# Synthesis — Hegel recommendation for kaya @ cb8be97

## Recommendation (pinned verdicts)

**Adopt hegeltest =0.52.7 with `features = ["static-engine"]` as a dev-dependency; `.hegel/` gitignored.** Approve the n×4 run (`cargo test` must stay green — every property is scoped to its well-formed/well-formed-inputs domain so a passing suite stays passing). Pinned nx4 list mapped to test files:

1. `one-event-one-line` → `tests/event_properties.rs` (boundary arms depth > 127, non-finite floats, duplicate keys are tickets kaya-8tn, excluded here)
2. `canonical-line-convergence` → `tests/event_properties.rs` (duplicate-key arm excluded = kaya-8tn)
3. `correction-totality` → `tests/ledger_properties.rs` (well-formed domain only: targets exist, string-or-string-array; cycles/self/dangling/malformed/order are tickets kaya-cl6 + kaya-8k5, excluded)
4. `append-byte-monotonicity` → `tests/ledger_properties.rs` (cosmetic-replay edge = ticket kaya-sh6, excluded)
5. `status-build-parity` → `tests/status_build_properties.rs`; subprocess state machine ACCEPTED (cost ≈ 0.07 s worst case: 4 cases × 12 steps × ~4 spawns ≈ 192 spawns at the eval-measured ≈ 0.37 ms/spawn; machine owns its tempdir git repo; deterministic exhaustive mutation alphabet — 5 atomic mutations × 2 git profiles — so the `ci` profile loses nothing). Scoped by the build-acceptance classifier; include the ledger-immutability oracle; `status.events` accounting and build-envelope-gap are tickets kaya-g11 / kaya-8yw / kaya-7ma.
6. `envelope-exit-code-agreement` → asserted within the same state machine (free on every spawn).

**Not test material:** `append-id-uniqueness` (design assessment → ticket kaya-1y9: a file lock — `flock(LOCK_EX)` or an `O_EXCL` lockfile — around the load+append+recover critical section, and recovery that refuses rather than truncates when it cannot take the lock). The recovery-on-read preserve-branch regression belongs to kaya-1y9, not to nx4.

Bug/design tickets filed from this pass: kaya-cl6, kaya-8k5, kaya-1y9, kaya-8tn, kaya-g11, kaya-8yw, kaya-7ma, kaya-sh6.

## Why scope-to-green-domain

Each property's dossier distinguishes CERTAIN-green behavior (read from code, cited file:line) from HYPOTHESIZED-red boundary/malformed arms. Scoping the properties to the domains where the invariant provably holds (depth ≤ 127 and finite floats for the codec; well-formed supersedes graphs for corrections; build-accepted ledgers for parity) keeps `cargo test` green while the reds are not lost — they are pinned as tickets with failure-input sketches and repro line references. This is also the cheapest way to satisfy the adoption constraint: the property suite must never block the crate's existing test gate.

## Subprocess accepted, with numbers

The eval's "expensive" tag on the parity property was relative to in-process properties. Measured: the eval's subprocess CLI state machine ran 100 cases × 12 steps in 0.44 s; the n×4 parity machine needs ≤ 192 spawns ≈ 0.07 s of subprocess time, plus one-time dev-binary and `static-engine` cold build (16.2 s, suite-level). The interesting divergences (three independently written parsers of the same bytes: `parse_ledger_bytes`, status's newline count, status's lenient `pending_proposals`) live only at the binary boundary, so the pure `sha256_hex` seam is not a substitute — keep the subprocess property, and file the one-line status-hasher unification onto `sha256_hex` as a cheap hardening commit alongside it.

## CI derandomization safe

Digest parity is a pure function of bytes (SHA-256; no wall-clock or id-derived fields in either envelope; `new_id`/timestamps fixed by the generator). Under `CI=true` the `ci` profile disables the database and derandomizes per test name; because the machine's mutation alphabet (5 atomic mutations × 2 git profiles) is swept exhaustively in one pass, the profile loses nothing and failures stay replayable via a stored failing example locally.

## recovery-on-read (open q7 — answered)

**Preserve on every path.** A complete-but-unterminated final line is never truncated and never refused: the library repairs-and-preserves (appends the missing `\n` on disk and reports `Recovery { truncated_bytes: 0, appended_newline: true }`, src/ledger.rs:289-301), truncating only a tail that cannot parse as one event (`set_len`, src/ledger.rs:303-313); the build path serves-without-repair (`split_inclusive`/`strip_suffix` accepts the unterminated line, src/main.rs:1031-1043) but may refuse the unverified tail under the git-history gate (`uncommitted_history`) unless `--expect-digest`/`--allow-uncommitted`. Neither side truncates a complete event; the only lossy path is a genuinely non-JSON tail, and only via the library repair. The untested preserve branch is a regression test owed to kaya-1y9. Full evidence: [`properties/correction-totality.md`](../properties/correction-totality.md).

## Negative result

**Neither `status` nor `build` mutates the ledger at this pin.** Status performs two read-only passes; build reads once, spawns read-only git commands, and writes only under `--out` (its stale-flag read-back never rewrites). The suspected write-on-read machinery (`Ledger::recover`) is reachable only from promote/challenge/acknowledge/append — not from the two commands. The immutability oracle stays in the state machine as one line to pin the contract against regressions. Evidence: [`properties/status-build-parity.md`](../properties/status-build-parity.md).
