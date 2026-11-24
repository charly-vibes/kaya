# Design: add-capture-commands

## Context

Every seam except the redaction filter is already shipped — the filter
exists only as the add-agent-session-capture commitments, implemented by
the follow-up — so the change is a front door, not new machinery:

- The graph consumes `model.entity.created` over the closed
  thirteen-entity set (`src/model.rs`, `build`); node ids are
  `<project>/<TYPE>/<key>`.
- The ledger dedups byte-exactly, refuses id reuse, and resolves
  corrections through `data.supersedes` (`src/ledger.rs`); the adapter
  path already mints deterministic corrections
  (`src/adapters/mod.rs`, `reobservation_correction`).
- The event schema pins `kaya.event/v0`, requires ≥ 3 dotted lowercase
  type segments, and refuses interpretation keys in any non-`promotion.`
  event (`src/event.rs`, `parse_data`) — command text must travel as
  `data.text`, never `data.verdict`/`data.conclusion`.
- The CLI's human-door commands (`promote`, `challenge`,
  `acknowledge`, `supersede-doctest`, `disposition`) already require
  `--actor` and refuse tool authorship; capture adds the first
  `human|agent` CLI door.
- The redaction commitment is fixed by add-agent-session-capture: match
  replaced with `[redacted]` in place, never whole-attribute, never
  silent truncation, high-entropy runs secret by default.

The pipeline for one invocation is therefore fixed by ordering alone:

```
text ──▶ redact ──▶ bound check (refuse, never cut)
                        │
                        ▼
        key ──────▶ subject `<project>/<TYPE>/<key>`
                        │
     effective text of  │  unchanged ──▶ duplicate (no append)
     the key from the   │  changed   ──▶ correction (data.supersedes)
     stored events ─────┤  absent    ──▶ first capture
                        ▼
        derive id ──▶ append ──▶ envelope {appended|duplicate|corrected}
```

## Decision log

### D1 — `note` maps to `Observation` (no new entity type)

The four commands need four entity types; three exist verbatim
(`Question`, `Decision`, `Finding`). For `note` there are three options:

- **Claim** — rejected. `Claim` is truth-bearing: in this vocabulary the
  assertion surface that feeds promotion's
  `proposed → established → challenged` lifecycle. A note is not an
  assertion with a truth value — it is often a link, a fragment, a
  hunch. Mapping notes onto `Claim` either drags every note into
  promotion semantics or litters the graph with `Claim` nodes that never
  promote. That is forms, not seconds.
- **New `Note` entity** — rejected. It grows the closed set
  (`EntityType`, `ALL`, `FromStr`, `ENTITY_TYPE_SET`, projection
  rendering, the kaya-model delta MODIFIED for `vocabulary_closed`, new
  layer property rows, specodelic bindings) — a design-boundary change
  priced like `cites`, for a word the vocabulary already expresses as
  `Observation`. The promise is lightweight; the cheapest honest mapping
  is the existing one.
- **`Observation`** — chosen. A note *is* a recorded observation made by
  a human or agent, which is exactly what the entity means. It also
  gives captured material a zero-delta forward path: `derives` is
  direction-fixed Finding → Observation, so a later `kaya finding` can
  be linked to the notes it derived from without any relationship-set
  change; and `Question`'s open-direction relationships (`investigates`,
  `supports`, `challenges`) reach Observations today.

**kaya-model delta forced: none.** The closed set stays at thirteen;
`build` already creates nodes of these types from
`model.entity.created`; no requirement text in kaya-model names event
kinds by family. This is the payoff of D1+D2: the only spec delta in
this change is the new `kaya-capture` capability.

### D2 — Event kind: reuse `model.entity.created`, not `capture.<x>.recorded`

Both spellings satisfy the schema's grammar (≥ 3 dotted lowercase
segments, `src/event.rs` `is_namespaced`), so the choice is semantic:

- **`capture.note.recorded` (+3 siblings)** — rejected. It adds a new
  family to the capture vocabulary whose graph semantics would be an
  exact duplicate of `model.entity.created`: `model.rs::build` would
  need four new match arms mapping kind → entity + fields
  (`capture_from_event`), verification facts would need a sixth source
  arm (`src/verification/facts.rs` classifies by type prefix), and the
  statement of which events create nodes would need a kaya-model delta.
  A second convention for "a node enters the graph" is exactly the drift
  the closed vocabulary exists to prevent.
- **`model.entity.created`** — chosen. The shipped seed-file door already
  uses it; the graph, corrections resolution, projections, and the
  source classification (`model.*` → source `model`) consume it today
  with zero changes. "Created" names the graph effect — the node enters
  the vocabulary — not a store mutation: the ledger stays append-only
  and edits still supersede. The command-vs-seed distinction travels in
  provenance (`tool: kaya`, reference `<subcommand>/<key>`) and in the
  actor, where it belongs; the wire type stays one family. The
  epistemic guard is unaffected and respected: command text uses
  `data.text`/`data.key`, never the five refused interpretation keys.

### D3 — Id derivation: the adapters' identity payload, correction ids generation-salted

First capture id (wai's derivation, command-flavoured payload):

```
id = "evt-" + sha256( tool \x1f cli_version \x1f project \x1f timestamp
                      \x1f subject.id \x1f serialized data )[:26 hex]
```

Tool is `kaya` (the provenance tool), timestamp is RFC 3339 UTC seconds
at invocation, data is the final redacted `{text, key}` object. Every
input that makes two capture acts distinct is in the payload, so two
acts never collide; every input of one act is in the payload, so a
replay of the same act is byte-identical and the ledger's byte-exact
dedup turns it into a no-op. Random ids (`event::new_id`, ULID — the
promotion family's choice) are rejected here: capture is
content-addressed like every adapter, and determinism is what makes
`id_reuse` unreachable from the command path.

Correction id: the shipped scheme from
`src/adapters/mod.rs::reobservation_correction` —
`evt-` + sha256(`correction \x1f <superseded id> \x1f <generation> \x1f
<data>`)[:26]. The generation salt (count of prior differing corrections
of the same id, `correction_generation`) keeps revisits collision-free.
The implementation may move these two helpers to a neutral module for
the command layer to share; the derivation is the contract, the module
placement is free.

Default key when `--key` is omitted: `<UTC date>-<sha256(redacted
text)[:8]>`, e.g. `2026-11-24-3f9a21bc` — deterministic, greppable, and
stable per (day, content), which makes an accidental same-day re-capture
of the same text a duplicate no-op instead of a fork.

### D4 — Redaction placement: command layer, first, before bounds and id derivation

The filter runs on the raw command text before anything else is computed.
Placement is not stylistic; it is load-bearing for replay:

- Ids are derived over the stored bytes. Redacting after hashing would
  make the id cover text the ledger never holds, so a re-run (or the
  corpus validation) would derive a different id over different bytes
  and either fork the node or trip `kaya.ledger.id_reuse`. Redact-first
  keeps stored bytes == hashed bytes == replayed bytes.
- Bounding after redaction means a text whose overage *is* the secret
  shrinks under the bound and is admitted safely; bounding first would
  refuse it and invite hand-trimming pasted secrets before capture —
  the exact failure the filter exists to prevent.
- The filter is deterministic (fixed marker, fixed pattern list), so the
  same pasted secret always yields the same id — asserted by
  `redacted_text_hashes_deterministically`.

Bounds are enforced on the redacted text: over-bound **refuses** with
`kaya.capture.bound_exceeded` — never truncates. Truncation is the one
silent rewrite this capability forbids; the marker replacement is not
silent (it is visible in the stored text) and never loses the
surrounding sentence.

### D5 — Actor policy: human default, agent opt-in, tool refused

`--actor NAME` is required (CLI convention in every human-door command);
`--actor-type` accepts `human` (default) or `agent`; `tool` is refused
with `kaya.capture.actor_required`. Rationale:

- Promotion's rule — tools may emit events, only human/agent actors
  author assertions, only humans approve (`src/promotion.rs`) — is about
  authorship, and a capture command is authored content. A tool that
  wants these events in the ledger belongs behind an adapter (or a seed
  file), where `ActorType::Tool` and the adapter's accountability rules
  apply.
- Agent authorship stays visible: the actor type is carried on the
  stored event. This does not bypass promotion — captures create
  vocabulary nodes, never `promotion.conclusion.established`; approval
  keeps its own door.

## Alternatives rejected (cross-cutting)

- **Auto-init the ledger on first capture** — refused: every existing
  command refuses with `missing-ledger` and points at `kaya init`;
  silently creating state in a wrong working directory scatters
  ledgers.
- **Burst capture / stdin batching (`kaya note -`)** — deferred: it
  multiplies the dedup/correction edge cases (partial batch refusal,
  per-line redaction reporting) for an MVP whose promise is one thought,
  one command. Filed as a follow-up candidate, not in scope.
- **Relationship flags (`--derives`, `--cites`) on capture** — deferred:
  the closed grammar already admits the pairs via seed files; flags add
  reference-validation UX the MVP does not need. Revisit after
  dogfooding (follow-up ticket).

## Risks

- **Pasted secrets in `--key` or `--project`.** The filter covers text
  only; keys are slug-validated (`[a-z0-9-]`), so a pasted token cannot
  survive as a key, and project names are namespace choices, not
  captured content. Detected by the slug-validation scenario.
- **Redaction list tuned for transcripts, not interactive paste.**
  Detection: the corpus-validation follow-up runs real captured text
  through the filter; misses extend the list, exactly as
  add-agent-session-capture prescribed.
- **Same-key captures racing.** The ledger lock serializes bytes, but
  two racing captures of one key with different texts leave two
  coexisting effective events for the subject id; the newest in ledger
  order is the key's effective text and the other stays in the record —
  visible, never lost. Detected by a concurrent-capture test in the
  implementation ticket.
- **Decision records surprising researchers with the staleness marker.**
  Command-captured decisions cite no Source, so they render
  ledger-global evidence plus the marker — specified behaviour, but new
  to interactive users. Detection: dogfooding ticket; mitigation would
  be docs, not code.
