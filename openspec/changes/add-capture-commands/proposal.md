# Proposal: add-capture-commands

## Why

The book promises hybrid capture: passive adapters *plus* lightweight
explicit capture — `kaya note/question/decision/finding`, "seconds, not
forms" (docs/src/index.md, Design decision 2). The adapter half shipped;
the explicit half did not. Today the only doors into the research graph
are bulk adapter imports and hand-written `model.*` seed files
(`kaya import model`), and the only human doors into the record are
`promote`/`challenge`/`acknowledge`/`supersede-doctest`/`disposition`.
A researcher who just
made a decision, answered a question, or learned a fact has no
seconds-sized way to record it, so the graph silently misses exactly the
human-authored material no adapter can observe. This is the
"continuous capture" MVP-2 seam named in docs/src/status.md, starting
here as an openspec proposal.

## What Changes

No production code in this change. This proposal fixes the command
surface, the target event shape, and the commitments the follow-up
implementation must honour.

### Four CLI subcommands, one shape

`kaya note|question|decision|finding <TEXT>` with shared flags:
`--actor NAME` (required, CLI convention), `--actor-type human|agent`
(default `human`; `tool` refused), `--key SLUG` (stable node key,
optional), `--project NAME` (default `kaya`, mirroring `import`).
Each invocation appends at most one event and prints the standard JSON
envelope naming the outcome — `appended`, `duplicate`, or `corrected` —
and the event id. A missing ledger is refused with `missing-ledger`
pointing at `kaya init`; nothing is auto-created.

### Target event shape (what the follow-up must emit)

| field | value | note |
|-------|-------|------|
| `type` | `model.entity.created` | the shipped family, `src/model.rs:21`; no new event kind |
| `subject.type` | `Observation` for `note`; `Question`, `Decision`, `Finding` for the others | closed thirteen-entity set, unchanged |
| `subject.id` | `<project>/<TYPE>/<key>` | the shipped namespace rule |
| `actor` | `{type: human\|agent, name: --actor}` | `tool` refused, `kaya.capture.actor_required` |
| `data.text` | the command text, redacted then bound-checked | the only free-text attribute |
| `data.key` | the node key | `data` carries exactly these two keys |
| `timestamp` | RFC 3339 UTC at invocation, seconds precision | |
| `provenance` | tool `kaya`, CLI version, reference `<subcommand>/<key>` | source-classified as `model` by the shipped facts rule |

`note` maps to the existing `Observation` entity: no new entity type and
**no kaya-model delta** (rationale in design.md). Because the graph
builder already consumes `model.entity.created`
(`src/model.rs`, `build`), captured nodes reach every projection with
zero `model.rs` changes, and the shipped verification facts classify the
events under source `model` with no new arm.

### Commitments the implementation must honour

1. **Secret redaction before anything else.** The command text passes
   the redaction filter inherited from add-agent-session-capture before
   bounding, id derivation, and storage: each matched secret is replaced
   with `[redacted]` in place — never the whole attribute, never silent
   truncation — including the default-secret rule that treats a
   base64/hex run of ≥ 32 characters as secret. Redaction is
   deterministic, so the same pasted secret always yields the same
   stored bytes and the same id.
2. **Bounds refuse, never truncate.** `data.text` over 2000 characters
   and a key over 64 characters (or outside `[a-z0-9-]`) are refused
   with `kaya.capture.bound_exceeded`, appending nothing. This is
   authored text; kaya never silently shortens what a human or agent
   wrote.
3. **Deterministic ids, idempotent replay.** Event ids are
   content-derived exactly as the adapters derive theirs (sha256 over a
   `\x1f`-joined identity payload, `evt-` + 26 hex). Re-capturing the
   same key with unchanged redacted text appends nothing and reports
   `duplicate`; the ledger's byte-exact dedup absorbs a byte-identical
   replay; `kaya.ledger.id_reuse` is never reachable from the command
   path.
4. **Edits supersede, never rewrite.** Re-capturing an existing key with
   changed text emits a correction — the same event shape carrying
   `data.supersedes` naming the prior effective id — on the ledger's
   shipped correction path (`resolve_effective`), with the
   generation-salted deterministic correction id the adapters already
   use (`src/adapters/mod.rs`). The node id never changes.
5. **Actor policy.** Only human and agent actors author captures; a
   `tool` actor is a labeled refusal. This does not bypass promotion:
   capture commands create vocabulary nodes, never conclusions; the
   promote/challenge doors and the promotion event family are untouched.

## Impact

- **This change**: an openspec proposal only. No Rust, no behaviour
  change.
- **Specs**: `kaya-capture` is a new capability (ADDED requirements
  only). No delta to any existing spec: `kaya-model` keeps its closed
  thirteen-entity set and shipped `model.entity.created` semantics;
  `kaya-event` keeps its schema — `model.entity.created` already passes
  the namespaced-type and observation-only guards, and command text
  uses `data.text`, never the refused interpretation keys;
  `kaya-ledger` keeps byte-exact dedup and supersedes corrections, both
  shipped.
- **Code (follow-up ticket kaya-w9t)**: four clap subcommands in
  `src/main.rs` sharing one handler; one new command-layer module
  implementing redact → bound → derive id → append or correct, reusing
  the wai id derivation scheme and
  `crate::adapters::{reobservation_correction, correction_generation}`
  (relocating those helpers to a neutral module if desired — no spec
  effect). Test expectations per the scenario names in the spec delta.
- **Docs**: none — the promise in docs/src/index.md already names these
  commands; the implementation ticket should drop a short usage note
  into the record's own docs when shipping.
- **Risks**, each with its trigger and detection signal:
  - the redaction list is tuned for transcripts, not interactive paste —
    detected by the corpus-validation follow-up ticket running real
    captured text through the filter;
  - the default key (`<date>-<digest8>`) collides for same-day
    same-text captures — intended (the second is a duplicate no-op), but
    if researchers expect distinct nodes, detected by dogfooding and
    fixed by making `--key` required instead;
  - command-captured Decisions cite no Source, so decision records
    render the ledger-global evidence with the staleness marker —
    already specified behaviour
    (kaya-model, Pre-relationship ledger attribution), detected by the
    dogfooding ticket if it surprises anyone.
