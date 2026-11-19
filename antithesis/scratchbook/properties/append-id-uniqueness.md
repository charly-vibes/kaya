# append-id-uniqueness — design assessment, not test material

Evidence file for [`property-catalog.md`](../property-catalog.md) (P4). Repo kaya @ `cb8be97696b59f99909ba7f39a10daa2819294bc`.

**Disposition (pinned):** NOT test material at this pin — design ticket kaya-1y9. The recovery-on-read preserve-branch regression also belongs to kaya-1y9, not to the nx4 property suite.

## Assessment

Id *generation* is not the risk: `event::new_id` mints ULIDs (src/event.rs:263-265), and adapter ids are content-derived sha256 prefixes (src/adapters/turu.rs:209-216), so intra-ledger id collision is negligible. The real hazards are **check-then-append races and destructive recovery on an unlocked file**:

- `Ledger::append` does: serialize → re-validate (src/ledger.rs:165-181) → `load_optional` reads and parses the **entire file** (src/ledger.rs:218-272) → linear byte-dup scan (src/ledger.rs:179-186) → id-reuse scan (src/ledger.rs:188-196) → single `write_all` through an `O_APPEND` fd (src/ledger.rs:198-206). There is **no lock anywhere** (repo-wide grep for flock/fs2/lock primitives: zero hits in src), no `fsync` (`flush` only), no compare-and-swap. `Ledger` is `&mut self`, but each CLI process owns its own handle — `kaya import` appends per event with no coordination (src/main.rs:503-538).
- **Race 1 — duplicate-line poisoning (sharpest):** two processes append the byte-identical line concurrently (e.g. two replays of the same turu bundle). Both scans miss the other's in-flight line, both write. The file now holds two identical lines with one id; the *next* read refuses the **whole ledger** with `kaya.ledger.id_reuse` naming both lines (src/ledger.rs:243-253) — permanent until hand-truncation.
- **Race 2 — torn line from `write_all` continuation:** `write_all` loops on short writes (src/ledger.rs:203). A regular-file `write(2)` under `O_APPEND` is atomic per call, but a partial write means the continuation lands at the *then-current* end — if another process appended in between, the surviving line is `prefix + foreign-line + suffix`, i.e. corrupt. Rare (short writes on regular files are uncommon), real.
- **Race 3 — destructive recovery vs in-flight append (worst consequence):** any read (`read_recovered`, and every `append` via `load_optional`) runs `recover` (src/ledger.rs:275-314). A reader that sees a writer's partially-written tail (prefix of a canonical line — fails `parse_line`, src/ledger.rs:287) **truncates the writer's line** (src/ledger.rs:303-313). If the writer's continuation then lands, the file holds a suffix fragment → next read truncates it silently → **the event is permanently lost**, reported only as `truncated_bytes`. Alternatively, if the reader's parse of the partial prefix *succeeds* (writer had finished all but `\n`), the reader appends a `\n` (src/ledger.rs:289-301) mid-write; the writer's `\n` then follows, producing an empty line → next read refuses `record_invalid` on the blank line (src/event.rs:185-190 wrapped at src/ledger.rs:236-241) → whole-ledger refusal again.

## Recommended handling

Not example-territory, but Antithesis multi-container is overkill: file a **design ticket** (kaya-1y9) for a file lock (`flock(LOCK_EX)` around the load+append+recover critical section, or an `O_EXCL` lockfile; make recovery refuse rather than truncate when it cannot take the lock). For Hegel fit, the TOCTOU seed *is* expressible in-process without threads: a state-machine arm that appends event A, externally injects a byte-identical copy of A's line via file write, then appends B and asserts reads stay servable — predicted red today (Race 1 deterministic under injection). True interleavings (Races 2-3) need a two-thread stress loop in a plain cargo integration test (tempdir already a dev-dependency, Cargo.toml) with the assertion "every read ends in Ok or a labeled refusal; appended-count conserves against readable-count + reported truncations" — recommend keeping that out of Hegel and in the design ticket.

## Also missing (test gap, not a bug)

No test exercises the preserve branch of `recover` (complete event minus `\n`) — the recovery-on-read q7 scenario is untested in tests/ledger.rs (truncated branch only, tests/ledger.rs:228-272). The concurrent-append ticket (kaya-1y9) should include it as a deterministic regression first. The q7 answer itself is recorded in [`correction-totality.md`](correction-totality.md).

## Open questions

- None for the assessment itself; kaya-1y9 carries the lock design decision.
