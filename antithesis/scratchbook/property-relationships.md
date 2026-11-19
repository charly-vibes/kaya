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

# Property relationships — kaya @ cb8be97

Derived from [`property-catalog.md`](property-catalog.md) (same write moment, same context); every slug below is cataloged there.

## Clusters

### correction-family
Members: `correction-totality`; tickets kaya-cl6 (supersedes cycles/self-reference/dangling silently erase records; also the `resolve` vs `resolve_effective` divergence), kaya-8k5 (malformed `supersedes` shapes + ordering semantics).

The excluded boundary arms of correction-totality are exactly these tickets' failure modes: append validates only line-parse, byte-dup, and id-reuse (src/ledger.rs:164-213), so every out-of-domain shape is reachable through the public API and fails silently (src/ledger.rs:478-479, 459, 488). Scope decision: the nx4 property holds only on the well-formed domain (targets exist, string-or-string-array); the tickets carry the refusal/validation fix.

### codec-family
Members: `one-event-one-line`, `canonical-line-convergence`; ticket kaya-8tn (depth > 127 asymmetry, non-finite floats, duplicate keys).

The two properties share the `kaya::event::{parse_line, to_line}` seam (src/event.rs:184, 249) and the same generator; one-event-one-line pins the round-trip and single-`\n` line-atomicity (the crash-recovery contract, src/ledger.rs:287), canonical-line-convergence pins that every spelling of a record converges to the same canonical bytes. kaya-8tn collects the shared boundary arms where `serde_json` behavior breaks both (recursion limit, NaN→`null` silent mutation, duplicate-key last-wins).

### envelope-family
Members: `status-build-parity`, `envelope-exit-code-agreement`; tickets kaya-g11 (status.events newline-count accounting), kaya-8yw (build serves a crash tail the library refuses), kaya-7ma (build envelope lacks `events`).

One subprocess state machine asserts both properties: the envelope-exit-code biconditional is a free per-spawn oracle inside the parity machine, and the parity machine's classifier (`build_accepts(B, profile)`) scopes parity to build-accepted ledgers. The tickets are the scoped-out divergences: status's newline-count `events` (src/main.rs:445) vs build's `parse_ledger_bytes` acceptance (src/main.rs:1031-1043), and the missing envelope field that would make event-count parity observable (kaya-7ma).

### durability-family
Members: `append-byte-monotonicity`, `append-id-uniqueness` (design assessment); tickets kaya-1y9 (file lock + non-destructive recovery + preserve-branch regression), kaya-sh6 (cosmetic-replay dedupe policy).

append-byte-monotonicity is the single-process slice of the same seam `Ledger::append` (src/ledger.rs:164-206): bytes only grow by whole canonical lines, refusals are byte-stable. append-id-uniqueness is its concurrent counterpart — the unlocked check-then-append (Races 1–3: duplicate-line poisoning, torn-line continuation, recovery truncating an in-flight append, src/ledger.rs:275-314) — deliberately not test material at this pin. kaya-1y9 also owns the recovery-on-read preserve-branch regression (the untested branch, tests/ledger.rs:228-272 covers only truncation).

## Cross-links

- **Append repair interacts with parity's immutability oracle.** `append` repairs on load (`load_optional` → `recover`, src/ledger.rs:218-222 → 275-314), so an append can mutate the file before its own dedupe scan (append-byte-monotonicity's adjacent hazard). Status and build bypass `Ledger` entirely, which is why the parity machine's ledger-immutability oracle holds green at this pin — but the oracle is kept as one line to pin the contract against regressions (negative result recorded in [`properties/status-build-parity.md`](properties/status-build-parity.md)).
- **Recovery-on-read is the shared durability/parity junction.** The q7 answer (preserve on every path; library repairs-and-preserves on disk, build serves-without-repair under the git gate; only non-JSON tails truncate, library-side) lives in [`properties/correction-totality.md`](properties/correction-totality.md). Build's silent serve-without-repair of a crash tail is ticket kaya-8yw; the untested library preserve-branch regression is ticket kaya-1y9.
- **Codec correctness underpins everything downstream.** one-event-one-line's single-`\n` guarantee is what makes line-splitting recovery and byte-level dedupe/anchor digests (src/ledger.rs:179-186, 349-374) sound; kaya-8tn's duplicate-key arm is where canonical bytes and served data diverge, which also corrupts append-byte-monotonicity's dedupe premise (the kaya-sh6 cosmetic-replay edge).
- **Correction resolution consumes the read paths.** correction-totality's production read path is `read_build_ledger` (build) and `read_recovered` (promote/acknowledge) — the same recovery-on-read junction; a correction fix (kaya-cl6/8k5) must not regress the immutability oracle, since `append`-driven validation would route through the repairing load path.

## Dominance notes

- envelope-exit-code-agreement is dominated by the status-build-parity machine (no separate test cost); drop it only if the machine is ever retired.
- canonical-line-convergence shares its generator with one-event-one-line; implement as arms of one Hegel module in `tests/event_properties.rs`.
- append-byte-monotonicity dominates nothing but is cheap; its refused-append arms already cover the file-unchanged assertions in tests/ledger.rs:100-108 as a property.
