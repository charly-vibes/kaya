# canonical-line-convergence — any JSONL spelling converges to the same canonical line

Evidence file for [`property-catalog.md`](../property-catalog.md) (P2). Repo kaya @ `cb8be97696b59f99909ba7f39a10daa2819294bc`.

**Disposition (pinned):** `tests/event_properties.rs` via hegeltest nx4. The duplicate-key arm is excluded = ticket kaya-8tn.

## Invariant

Any two JSONL spellings of the same record (key permutation at any depth, whitespace variation, unterminated final line) either both parse to the same `Event` with byte-identical canonical `to_line`, or both are refused with the same label.

## Check mode

Pure in-process.

## Seam

`kaya::event::{parse_line, to_line}` (src/event.rs:184, 249).

## Generator sketch

Generate a valid `Event`, render via `to_line`, then apply structure-preserving byte mutations: shuffle top-level and `data` keys (serde_json's default `Map` is order-insensitive on parse), inject whitespace between tokens, strip the trailing `\n`, and re-serialize. Assert equal parse + equal canonical bytes.

## Predicted red/green

- **CERTAIN-green** for permutation/whitespace/unterminated (field set is closed and order-insensitive at parse, src/event.rs:267-296; canonicalization already asserted for the unterminated case, tests/event.rs:38-50).
- **HYPOTHESIZED-red, excluded (boundary — duplicate keys, ticket kaya-8tn):** duplicate keys — `serde_json`'s map silently last-wins, so `{"id":"X",…,"data":{"x":1,"x":2},…}` parses with `data.x == 2` and **no refusal** (src/event.rs:185-192), while the stored ledger bytes and the served data diverge; the byte-level dedupe and per-line `Anchor` digests (src/ledger.rs:179-186, 349-374) then operate on bytes the parsed data doesn't faithfully describe. Failure input sketch: that duplicate-key line → `read_recovered` serves `x:2`, original file bytes say `x:1,x:2`. Recommended fix, filed as kaya-8tn: refuse duplicate keys at `parse_line` (or bless last-wins explicitly in docs).

## Open questions

- None that change the invariant in the scoped domain; the duplicate-key boundary is ticketed (kaya-8tn).
