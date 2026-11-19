# one-event-one-line — event serialization is line-atomic and round-trips

Evidence file for [`property-catalog.md`](../property-catalog.md) (P2). Repo kaya @ `cb8be97696b59f99909ba7f39a10daa2819294bc`.

**Disposition (pinned):** `tests/event_properties.rs` via hegeltest nx4. Boundary arms — depth > 127, non-finite floats — and the duplicate-key arm (which belongs to canonical-line-convergence's domain) are tickets kaya-8tn and excluded here.

## Invariant (falsifiable)

For every `Event` accepted by `parse_line` (the ingest-validity gate), `parse_line(to_line(e)) == e` exactly, and `to_line` contains exactly one `\n`, always in field-declaration order (`to_line(parse(to_line(e))) == to_line(e)`).

## Check mode

Pure in-process. Deterministic regression arm already exists as a fixed fixture (tests/event.rs:23-52); the property adds the generated distribution.

## Seam

`kaya::event::{parse_line, to_line}` (src/event.rs:184, src/event.rs:249) — both public, no extraction. `Event` is `Serialize + Deserialize + PartialEq` with `deny_unknown_fields` (src/event.rs:159).

## Generator sketch

A custom generator (not `derive(DefaultGenerator)` — validity is grammar-heavy): `id`/`actor.name`/`subject.id`/`provenance.{tool,version,reference}`/artifact items = arbitrary non-empty-after-trim strings from a corpus of ASCII, unicode (astral, combining, U+2028/2029), control chars, quotes/backslashes, whitespace-padded forms (whitespace-only is refused by `require_string`, src/event.rs:297-316 — keep in the negative arm); `type` from the namespaced grammar (≥3 dotted lowercase segments, src/event.rs:460-477, plus the `verification.acknowledged` exception); `timestamp` = one instant spelled as `…Z`, `…+00:00`, `…Z.123456789` — all accepted and preserved verbatim (src/event.rs:318-337; `+00:00` accepted per tests/event.rs:213-222); `data` = recursive JSON-object generator, keys arbitrary except the five interpretation keys at top level unless `type` starts with `promotion.` (src/event.rs:31-37, 390-403); numbers across int/u64/i64/finite-float/negative/`-0.0`/big; nested objects/arrays; empty `{}`; long strings. Depth cap 127 for the main arm (serde_json's default recursion limit — see excluded red below).

## Predicted red/green

- **CERTAIN-green** (from code): field order — `to_line` serializes the struct, so bytes are always declaration order (src/event.rs:246-261), canonical and idempotent; `parse_line` trims, so `to_line`'s trailing `\n` parses back (src/event.rs:185); single `\n` per line — `serde_json::to_string` escapes any control char inside strings, so a literal newline in any field can never terminate the line early (this is exactly what ledger crash-recovery relies on, src/ledger.rs:287); all fixed-string fields round-trip verbatim (`require_string` stores `as_str` unmodified, src/event.rs:297-316); unicode passes through unescaped and byte-stable; empty data `{}` passes the object check (src/event.rs:381-389); serde_json's default number handling is class-stable (`1` stays `1`, `1.0` stays `1.0` via ryu shortest-repr), so `parse∘to_line` is idempotent; existing fixture test confirms the shape (tests/event.rs:23-52).
- **HYPOTHESIZED-red, excluded (boundary class 1 — depth asymmetry, ticket kaya-8tn):** `to_line` serializes arbitrarily deep `data`, but `parse_line`'s `serde_json::from_str` enforces the default 128-level recursion limit (serde_json `1.x`, `unbounded_depth` feature **not** enabled — Cargo.toml has bare `serde_json = "1"`). Input sketch: construct `Event` in code with `data` nested 200 objects deep → `to_line` succeeds, `parse_line(to_line(e))` errors. Contained in production: `append` re-parses and refuses the half-record (src/ledger.rs:175-181), and adapters self-check (src/adapters/turu.rs:170-177) — so this is a codec-totality wart, not ingest corruption. Generator: dedicated deep-arm case; the property scopes "valid" to depth ≤ 127; the asymmetric refusal is documented in the ticket.
- **HYPOTHESIZED-red, excluded (boundary class 2 — non-finite floats, ticket kaya-8tn):** `serde_json` serializes non-finite `f64` as `null`. Input sketch: `Event { data: {"ratio": f64::NAN}, … }` → `to_line(e)` yields `"ratio":null` → `parse_line` **succeeds** with `data == {"ratio": null} ≠ e` — a *silent mutation*, strictly worse than a refusal. Unreachable through every current producer (all build `data` from parsed source JSON), so latent; but the property as stated ("every constructible Event round-trips") fails. Generator: boundary arm with `f64::INFINITY`/`NAN` payloads is deliberately not in the property domain.
- **Observation (not red):** timestamps are validated but not normalized — `…Z` and `…+00:00` are different bytes for the same instant (src/event.rs:318-337). The doc claim "equal events always produce equal bytes" (src/event.rs:246-247) holds because `Event` equality is string-based; semantic-instant dedup simply doesn't exist. turu normalizes its own emitted timestamps (src/adapters/turu.rs:113-118) while ingest doesn't — inconsistent discipline, harmless to this property.

## Open questions

- None that change the invariant in the scoped domain; both boundary classes are ticketed (kaya-8tn).
