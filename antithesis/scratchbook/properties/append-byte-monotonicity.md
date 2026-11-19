# append-byte-monotonicity — the ledger grows only by whole canonical lines at the end

Evidence file for [`property-catalog.md`](../property-catalog.md) (P2). Repo kaya @ `cb8be97696b59f99909ba7f39a10daa2819294bc`.

**Disposition (pinned):** `tests/ledger_properties.rs` via hegeltest nx4. The cosmetic-replay edge is excluded = ticket kaya-sh6.

## Invariant

For any sequence of `append` calls of generated events, the ledger file only ever changes by whole canonical lines appended at the end; every outcome is exactly one of `Appended` (bytes grew by `to_line(e)`), `Duplicate` (bytes unchanged), or a labeled refusal (`record_invalid`/`id_reuse`, bytes bit-identical to before).

## Check mode

In-process with tempdir — a small state machine (append / refused-append / read) per generated sequence.

## Seam

`kaya::ledger::{Ledger::append, Ledger::read_recovered, AppendOutcome}` (src/ledger.rs:90, 164; enum at src/ledger.rs:321-330).

## Generator sketch

Sequences of events drawn from the one-event-one-line generator plus deliberate near-duplicates: identical id with different `data`, identical bytes with different id, replays of earlier events. Byte-level postcondition: `fs::read(path)` is a concatenation of canonical lines and is prefix-monotone across steps.

## Predicted red/green

- **CERTAIN-green** for the scoped behavior — dedupe is byte-identical (src/ledger.rs:179-186), id-reuse refuses without writing (src/ledger.rs:188-196; file-unchanged already asserted in tests/ledger.rs:100-108), first append creates the file (tests/ledger.rs:292-300), refused writes verified byte-stable (tests/ledger.rs:103-108).
- **HYPOTHESIZED-red, excluded (adjacent, same seam — cosmetic replay, ticket kaya-sh6):** dedupe compares raw stored bytes, so a semantically-equal but cosmetically-different replay of a hand-edited line (e.g. duplicate JSON keys last-wins at parse, src/event.rs:185-192 — the canonical `to_line` differs from stored bytes) is refused as `id_reuse` rather than deduped: append of the canonical line fails the byte-compare (src/ledger.rs:179-186) then hits the id check (src/ledger.rs:188-196). Failure input sketch: write a line for id X with `"data":{"x":1,"x":2}` by hand; then `append(canonical(X))` → `kaya.ledger.id_reuse` refusal for a *faithful* replay. Severity low; behavior arguably correct (protects the log), but worth an explicit policy statement — ticket kaya-sh6: "define replay policy for cosmetically-different stored lines".

## Recovery interaction

Append itself repairs on load (`load_optional` → `recover`, src/ledger.rs:218-222 → 275-314), so an append can mutate the file before its own dedupe scan — the concurrency ticket's core fact; see [`correction-totality.md`](correction-totality.md) (recovery-on-read section) and [`append-id-uniqueness.md`](append-id-uniqueness.md) (ticket kaya-1y9).

## Open questions

- None that change the invariant in the scoped domain; the cosmetic-replay edge is ticketed (kaya-sh6), and the append-repairs-before-scan interaction is ticketed (kaya-1y9).
