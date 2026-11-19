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

# Property catalog — kaya single-crate deep pass

Evidence: [`properties/{slug}.md`](properties/) (one file per property below). All line numbers verified against kaya @ `cb8be97696b59f99909ba7f39a10daa2819294bc`.

"Predicted status at pin" is the pre-test expectation from the dossiers: **green** (CERTAIN-green from code reading), **red** (HYPOTHESIZED-red with a failure input sketch), or **red-leads** (green as scoped, but predicted reds exist in adjacent arms / boundary classes that were deliberately scoped out to tickets).

| Slug | Title | Type | Check mode | Priority | Predicted at pin | Disposition |
|---|---|---|---|---|---|---|
| correction-totality | Corrections resolve totally; no observation silently disappears | Safety | Pure in-process | P1 | green (scoped) | `tests/ledger_properties.rs` (nx4) |
| one-event-one-line | Event serialization is line-atomic and round-trips | Safety | Pure in-process | P2 | green (scoped) | `tests/event_properties.rs` (nx4) |
| canonical-line-convergence | Any JSONL spelling converges to the same canonical line | Safety | Pure in-process | P2 | green (scoped) | `tests/event_properties.rs` (nx4) |
| append-byte-monotonicity | Ledger grows only by whole canonical lines at the end | Safety | In-process, tempdir state machine | P2 | green (scoped) | `tests/ledger_properties.rs` (nx4) |
| status-build-parity | Status and build digest the same bytes and agree on the record | Safety (envelope parity) | Subprocess state machine | P3 | green (scoped; events-accounting reds → tickets) | `tests/status_build_properties.rs` (nx4) |
| envelope-exit-code-agreement | Every invocation emits one envelope; exit code ⇔ `ok` | Safety (CLI contract) | Subprocess (free rider on parity machine) | P3 | green (scoped) | asserted in `tests/status_build_properties.rs` (nx4) |
| append-id-uniqueness | No id reuse, no torn or poisoned lines under concurrent append | Safety / concurrency | (design assessment — not test material) | P4 | red under injection ( Races 1–3) | ticket kaya-1y9 |

## correction-totality — P1

Evidence: [`properties/correction-totality.md`](properties/correction-totality.md). For any ledger built by `Ledger::append` where every correction's `data.supersedes` names ids already present, `resolve_effective` returns every non-correction event exactly once, each superseded event carrying the payload of the last correction in ledger order that transitively reaches it (chains flatten, `supersedes` key stripped, all other fields preserved), and never returns a correction event. Scoped to the well-formed domain: cycles/self-reference/dangling/malformed shapes/correction-before-target ordering are excluded (tickets kaya-cl6, kaya-8k5). File: `properties/correction-totality.md`; the recovery-on-read answer (open q7) is recorded there as a final section.

## one-event-one-line — P2

Evidence: [`properties/one-event-one-line.md`](properties/one-event-one-line.md). For every `Event` accepted by `parse_line`, `parse_line(to_line(e)) == e` exactly, and `to_line` contains exactly one `\n`, in field-declaration order. Scoped to depth ≤ 127 and finite floats; the depth>127 asymmetry and non-finite-float silent `null` mutation are tickets (kaya-8tn). File: `properties/one-event-one-line.md`.

## canonical-line-convergence — P2

Evidence: [`properties/canonical-line-convergence.md`](properties/canonical-line-convergence.md). Any two JSONL spellings of the same record (key permutation at any depth, whitespace variation, unterminated final line) either both parse to the same `Event` with byte-identical canonical `to_line`, or both are refused with the same label. The duplicate-key last-wins arm is excluded (ticket kaya-8tn). File: `properties/canonical-line-convergence.md`.

## append-byte-monotonicity — P2

Evidence: [`properties/append-byte-monotonicity.md`](properties/append-byte-monotonicity.md). For any sequence of `append` calls, the file only changes by whole canonical lines appended at the end; every outcome is exactly `Appended` (bytes grew by `to_line(e)`), `Duplicate` (unchanged), or a labeled refusal (bytes bit-identical). The cosmetic-replay edge (byte dedupe refusing semantically-equal replays of hand-edited lines) is excluded (ticket kaya-sh6). File: `properties/append-byte-monotonicity.md`.

## status-build-parity — P3

Evidence: [`properties/status-build-parity.md`](properties/status-build-parity.md). For a fixed ledger byte sequence in a fixed git state, one `kaya status` and one `kaya build --out <tmp>` over the same bytes either both accept — then `status.data.digest == build.data.ledger_digest`, `status.data.bytes == |B|`, `status.data.pending_proposals == build.data.pending_proposals` — or build refuses with a labeled error. Scoped by the build-acceptance classifier; neither run may mutate the ledger or write outside `--out` (ledger-immutability oracle included). The `status.events` accounting cases and build-envelope-gap are tickets kaya-g11 / kaya-8yw / kaya-7ma. File: `properties/status-build-parity.md`.

## envelope-exit-code-agreement — P3

Evidence: [`properties/envelope-exit-code-agreement.md`](properties/envelope-exit-code-agreement.md). For every invocation routed through `emit`, stdout is exactly one pretty-JSON envelope carrying the fixed field set, error envelopes carry a non-empty `error.label`, and `process.exit == 0 ⇔ ok == true`. `check` (which uses `emit_code` and returns exit 1/2 with `ok: true`) is deliberately out of scope. File: `properties/envelope-exit-code-agreement.md`.

## append-id-uniqueness — P4 (design assessment, not a test)

Evidence: [`properties/append-id-uniqueness.md`](properties/append-id-uniqueness.md). Id generation is not the risk (ULIDs, content-derived adapter ids); the hazards are unlocked check-then-append races and destructive recovery on an in-flight append: duplicate-line poisoning (permanent `id_reuse` refusal), torn-line continuation, and recovery truncating/splitting a writer's line. Not test material at this pin — design ticket kaya-1y9 (`flock`/lockfile around the load+append+recover critical section; recovery refuses rather than truncates without the lock). File: `properties/append-id-uniqueness.md`.

## Ticket index

Bug/design tickets filed from this pass: kaya-cl6 (supersedes cycles/self/dangling silently erase records), kaya-8k5 (malformed supersedes shapes + ordering semantics), kaya-8tn (codec boundary arms: depth>127, non-finite floats, duplicate keys), kaya-1y9 (ledger file lock + non-destructive recovery; includes the preserve-branch regression test), kaya-g11 (status.events newline-count accounting), kaya-8yw (build serves crash tail the library refuses), kaya-7ma (build envelope lacks event count), kaya-sh6 (cosmetic-replay dedupe policy).
