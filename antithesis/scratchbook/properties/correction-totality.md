# correction-totality — corrections resolve totally; no observation silently disappears

Evidence file for [`property-catalog.md`](../property-catalog.md) (P1). Repo kaya @ `cb8be97696b59f99909ba7f39a10daa2819294bc`; all line numbers verified in this tree.

**Disposition (pinned):** `tests/ledger_properties.rs` via hegeltest nx4, well-formed domain only. Cycles/self-reference/dangling/malformed shapes/ordering are tickets kaya-cl6 + kaya-8k5 and are excluded from the property's domain.

## Invariant (falsifiable)

For any ledger produced by a sequence of `Ledger::append` calls where every correction event's `data.supersedes` names ids **already present in the ledger**, `kaya::ledger::resolve_effective` returns every non-correction event exactly once — each superseded event carrying the payload of the **last correction in ledger order** that transitively reaches it (chains A→B→C flatten to C, `supersedes` key stripped, id/type/subject/timestamp/actor/provenance/artifacts preserved) — and never returns a correction event.

## Check mode

Pure in-process (`Vec<Event>`); the resolver needs no file. The well-formed scope is generator-side: append-order = ledger order.

## Seam

`kaya::ledger::resolve_effective` (src/ledger.rs:429), helpers `apply_effective_data` (src/ledger.rs:471) and `supersede_targets` (src/ledger.rs:494); oracle consumer `kaya::model::build` (src/model.rs:418) and `kaya::ledger::changed_event_ids` (src/ledger.rs:404-405). No seam extraction needed — everything is public (`src/lib.rs` exports `ledger`, `model`).

## Generator sketch

Two-stage generator: (a) ids from a small alphabet with a uniqueness filter; observations = `experiment.run.completed` with `data {"passed": u32}`; corrections = `experiment.run.corrected` with `data.supersedes` drawn from ids already emitted (tracks append index). Explicit chain-depth strategy: depth 0 (plain observation), 1, 2, 3 chains, plus diamonds (`A supersedes [B, D]` where both reach one target) and multi-targets (`supersedes` as string vs array — both supported, src/ledger.rs:495-504). (b) A separate boundary arm (outside the invariant, feeds the bug leads / tickets kaya-cl6 + kaya-8k5): dangling targets, self-reference, mutual cycles, `supersedes: []`, `supersedes: [42, null]`. Oracle: recompute expected effective list in the test (iterate corrections in ledger order, walk each target chain to its terminal id, last writer wins) and compare to `resolve_effective` output including patched `data` equality.

## Predicted red/green

- **CERTAIN-green** (from code): chain flattening — `apply_effective_data` recurses through correction-valued targets carrying the newest payload (src/ledger.rs:480-489); corrections are applied in ledger order (src/ledger.rs:441-456), so the last correction wins at the terminal record; the final filter keeps all non-correction events (src/ledger.rs:457-467), so within the well-formed scope no observation can vanish — the output filter drops an event iff its id has non-empty `supersede_targets` (src/ledger.rs:431-439).
- **Termination characterization (CERTAIN, all three silent — no `LedgerError`, no `Recovery` report anywhere in this path; these are the excluded arms, ticketed):**
  - *Cycles* (A supersedes B, B supersedes A): terminate — `visiting` guard returns silently at src/ledger.rs:478-479; nothing is patched; **both records are then filtered from output** (src/ledger.rs:459) because both are in the corrections map. Reachable through the public API: `append` validates only line-parse, byte-dup, and id-reuse — it never checks that `supersedes` targets exist (src/ledger.rs:164-213), and `load_inner` doesn't either (src/ledger.rs:218-272). **HYPOTHESIZED-red**: total silent observation loss; failure input sketch: `append(A{data.supersedes:"B"})`, then `append(B{data.supersedes:"A"})`, then `model::build` sees an empty graph with exit code 0.
  - *Self-reference* (`A{supersedes:"A"}`): terminates immediately via the same guard; the event is both correction and observation, and is filtered out of `resolve_effective` output (src/ledger.rs:459) — payload vanishes. Note `resolve` (src/ledger.rs:381-393) *keeps* it: the `superseded.contains` check at src/ledger.rs:383-386 runs before the target insert, so a lone self-loop survives `resolve` but dies in `resolve_effective` — resolver divergence (also flagged in the catalog's open question; `resolve` has no production callers, only tests/ledger.rs:138,310).
  - *Dangling* (`A{supersedes:"ghost"}`): terminates; payload parked under the ghost id in `effective_data` (src/ledger.rs:488) which is never read (final step iterates real events only, src/ledger.rs:457-467); A itself filtered out. The observation survives; the correction silently becomes a no-op. **HYPOTHESIZED-red** (silent, unrecoverable by any report).
- **HYPOTHESIZED-red, ordering (excluded, ticket kaya-8k5):** "last correction wins" is *iteration-order* semantics, not recency semantics. A correction may be appended before its target (nothing forbids it): ledger `[A{sup B}, B{sup C}, C]` — iteration applies A first (C := A's payload), then B (C := B's payload), so the *earlier-in-ledger* correction B wins even though A is later and both name their chain. Unreachable only by append discipline, not by enforcement. Failure input sketch: exactly the three-line ledger above; expected effective C-data = B's payload, arguably should be A's.
- **Observation (not red):** recursion depth of `apply_effective_data` is O(chain length) with no cap (src/ledger.rs:480-485); a ~10⁶-long chain could overflow the stack, unreachable at realistic ledger sizes.

## recovery-on-read (open q7 — final answer, placed here by convention)

**Answer: a complete-but-unterminated final line is PRESERVED — never truncated, never refused — on every read path. Truncation happens only for a tail that cannot parse as one event, and only via the library repair path.**

- **Library path** (`read_recovered` src/ledger.rs:90-97, `read_verified` src/ledger.rs:103-110, and implicitly every `append` via `load_optional` src/ledger.rs:218-222): `recover` checks the final line (src/ledger.rs:275-287); if it is UTF-8 and `parse_line` succeeds, the missing `\n` is appended **to the file on disk** (src/ledger.rs:289-301) and pushed in-memory (src/ledger.rs:298), reported as `Recovery { truncated_bytes: 0, appended_newline: true }` (src/ledger.rs:299-301). Only an unparsable tail is truncated to the last line boundary, on disk via `set_len` (src/ledger.rs:303-313). The truncated branch is covered by tests (tests/ledger.rs:228-272); the *preserve* branch has **no test** — that is exactly the q7 gap (tests/ledger.rs writes only a non-JSON partial prefix, tests/ledger.rs:244-247). That regression belongs to ticket kaya-1y9, not to the nx4 property suite.
- **Build path** (`read_build_ledger` src/main.rs:915-957) does **not** repair: it parses raw bytes with `parse_ledger_bytes`, whose `split_inclusive`/`strip_suffix(b"\n").unwrap_or(raw)` accepts and parses a final unterminated line (src/main.rs:1031-1043) — so the event is served, not lost. However the git-history gate requires the committed HEAD copy to be newline-terminated (`committed.ends_with(b"\n")`, src/main.rs:999) and all growth to end in `\n` (src/main.rs:1002); an unterminated final line is therefore refused as `kaya.ledger.uncommitted_history` unless `--expect-digest` (digest over raw bytes, src/main.rs:945-956) or `--allow-uncommitted` (src/main.rs:930-941) is passed.
- So: library reads *repair-and-preserve*; the build path *serves-without-repair* but may refuse the unverified tail under the git gate. Both answers are "preserved". The catalog's conditional ("the answer decides which side is wrong") resolves to: neither side truncates a complete event; the only lossy path is a genuinely non-JSON tail, and only via the library repair.
- This read path feeds `resolve_effective` in production via `read_build_ledger` (build) and `read_recovered` (promote/acknowledge).

## Open questions

- None that change the invariant in the scoped domain; the excluded arms are ticketed (kaya-cl6, kaya-8k5), and `ledger::resolve`'s status (public contract vs dead API) is folded into kaya-cl6.
