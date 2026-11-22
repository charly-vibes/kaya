# Proposal: add-agent-session-capture

## Decision

**Distilled.** Agent sessions enter kaya as *distilled turu whisper bundles*,
observed through the turu adapter that already ships
(`capture.entry.observed`), carrying bounded, redacted attributes only —
never transcript text. Raw session transcripts (pi/terapia transcripts as
`capture.entry` events) are **rejected for now**, with the concrete
conditions that would unlock them recorded below.

The ownership rule as the code states it decides the surface: no adapter
copies its owner's *text* (`src/adapters/turu.rs:41` — "Entry text is never
copied into an event"), and a bounded content-derived attribute is allowed
only where the owner keeps the full text and the attribute has a declared
bound (`src/adapters/wai.rs:7` — "bounded content-derived attribute != text
ownership; wai keeps full text"). A raw transcript would break that rule:
it is a design-boundary change, not a tweak, and it needs its own spec
decision, its own redaction guarantee, and its own dedup and size rules
before any byte of a session may cross into the ledger.

The distilled surface is not a consolation prize: a whisper bundle carries
the session's structure — entry id, timestamp, topic, scope, plus the bundle
digest — which is which session, when, about what. It also carries a
`freeform` section that the adapter explicitly never reads and never emits
(`src/adapters/turu.rs:8-9`); what a bundle omits from the adapter's
contract is the entry text and that freeform section, not merely "the
transcript body".

**Precondition (the falsifier).** Distilled is admissible only if
agent-session material is published as turu whisper bundles. If a session
lives only in pi/terapia, distilled has no carrier and either a bundle writer
or the raw path must be reconsidered. This proposal assumes the carrier
exists; the follow-up ticket must verify it before implementing.

**What would change this decision.** A raw surface becomes admissible only
when all four hold: (1) a spec delta names the session store as a text
owner, parallel to wai and turu; (2) a redaction layer is proven by tests
over real transcripts; (3) size bounds and dedup are defined over
transcript bytes; (4) a new event kind in the closed capture vocabulary
carries a bounded excerpt, never the whole transcript.

## Why

An external evaluation called agent sessions the richest available source of
research material — an **unverified claim, treated here as motivation, not
evidence**. Today kaya captures none of that material directly: the turu
adapter observes whisper bundles, and nothing names an agent session as a
first-class source or links a decision to the session that produced it.
Capturing agent sessions as they already travel (whisper bundles) is the
only surface that takes that richness without breaking the ownership rule:
the session store keeps the text, kaya records the observation.

"MVP-2" here means the follow-up generation work that has no generator
caller yet (the `cap_proposals` library API) — the surface this proposal
feeds, not a shipped feature.

## What Changes

No production code in this change. This proposal fixes the surface, the
target event shape, and the commitments the follow-up implementation must
honour.

### Target event shape (what the follow-up must emit)

| field | value | bound | redaction |
|-------|-------|-------|-----------|
| `r#type` | `capture.entry.observed` (closed capture vocabulary, `src/model.rs:31`) | — | — |
| `subject.id` | `<project>/Source/entry-<entry id>` | — | — |
| `actor` | `ActorType::Tool`, name = the adapter (`turu`) — see the note below | — | — |
| `data.entry` | the bundle's entry id | ≤ 64 chars | not session-derived text |
| `data.topic` | the bundle's topic | ≤ 80 chars, cut on a character boundary | **required** — session-derived free text |
| `data.scope` | the bundle's scope | ≤ 64 chars | **required** |
| `data.repo_key` | the bundle's repo key | ≤ 64 chars | not session-derived text |
| `data.bundle_digest` | full sha256 of the bundle bytes, **64 hex** as shipped (`src/adapters/turu.rs:64`) | 64 hex | — |
| `provenance.reference` | the bundle digest, as shipped | 64 hex | — |

The bundle digest stays 64 hex and stays out of the dedup key (see below);
shortening it would be a schema-visible change to shipped event bytes and is
not proposed here.

**Actor typing.** `ActorType::Tool` ("a program emitted the observation",
`src/event.rs:108-115`) stays: the adapter program emitted the observation,
not the session's agent. The closed set already contains `Agent`
(`src/event.rs:114`) — no set change is needed; the open question is which
observations are agent-authored rather than adapter-observed, and none are
today. If a future surface needs `Agent`, it must bump `SOURCE_VERSION` so
the ids change, because the event id does not include the actor
(`src/adapters/turu.rs:218-222`): changing the actor bytes under a stable id
makes the ledger refuse the re-import with `kaya.ledger.id_reuse`
(`src/ledger.rs:250-256`), and the correction path is a
`data.supersedes` event, not an edit.

1. **Secret redaction.** Every session-derived string attribute — bounded or
   not — passes the redaction filter before it is copied. The filter
   **replaces the matched secret with a fixed marker** (`[redacted]`),
   preserving the surrounding attribute; it never replaces the whole
   attribute and never truncates silently. Starting pattern list (to be
   validated against real transcripts by the implementation ticket):
   `sk-[A-Za-z0-9_-]{16,}` (covers `sk-proj-…`), `ghp_[A-Za-z0-9]{36}`,
   `github_pat_[A-Za-z0-9_]{22,}`, `AKIA[0-9A-Z]{16}`, `ASIA[0-9A-Z]{16}`,
   `xox[baprs]-[A-Za-z0-9-]{10,}`, JWT
   `eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}`,
   `-----BEGIN [A-Z ]*PRIVATE KEY-----`,
   `-----BEGIN PGP PRIVATE KEY BLOCK-----`,
   `PuTTY-User-Key-File`, and
   `(?i)(api[_-]?key|secret|password|token)\s*[:=]\s*\S{8,}`. A high-entropy
   value (a base64 or hex run of ≥ 32 characters) is treated as secret by
   default, so an unlisted key family cannot leak silently. Required tests:
   one positive fixture per pattern, one negative fixture per pattern (a
   non-secret that must survive), a fixture proving the marker replaces the
   match rather than the attribute, and a `topic` fixture (the attribute that
   can carry a pasted secret today).

2. **Transcript size bounds.** Bounded attributes only, with the per-field
   bounds in the table above: `topic` ≤ 80 characters cut on a character
   boundary, `scope` and `entry` ≤ 64 characters, `bundle_digest` 64 hex. No
   transcript body is ever read into an event. The per-import entry cap is
   **not fixed by this proposal**: the implementation ticket must derive it
   from the ledger's per-line size budget and state the number before
   shipping. The failure mode is fixed now — a bundle over the cap is
   refused atomically with the adapter's existing
   `kaya.adapters.failure_atomic` label, naming the bundle; never a partial
   capture.

3. **Incremental import dedup by content hash.** The dedup key is defined
   per entry and never over the whole bundle file:

   ```
   dedup content = sha256 over the UTF-8 bytes of
       tool \x1f source_version \x1f project \x1f <normalised ts> \x1f
       <entry id> \x1f <topic> \x1f <scope>
   ```

   The bundle digest travels as provenance and as `data.bundle_digest` only;
   it never enters the key. This is what makes the import incremental: the
   shipped turu id folds in the whole-bundle digest
   (`src/adapters/turu.rs:218-222`), so appending one entry to a session
   changes the digest and re-ids every earlier entry. Required test: import
   a bundle with entries A and B, append C, re-import — assert exactly one
   new event and unchanged ids for A and B.

4. **Event kind and actor typing.** Kind `capture.entry.observed` with
   subject `<project>/Source/entry-<entry id>`, as shipped; actor
   `ActorType::Tool` with the adapter's name, as shipped, per the note above.

5. **Hooks for MVP-2 relationship linking.** Today the only link from a
   session capture into the graph is `cites` (Decision → Source, exact pair,
   `src/model.rs:189`) pointing at the Source node
   `<project>/Source/entry-<entry id>`. That Source node carries only the
   `entry` attribute (`src/model.rs:439-441`), so the entry id is the join
   key; topic, timestamp and scope live in the event, not in the graph node.
   Linking a Finding to a session is **not** expressible today: `influences`
   is Decision → Finding and `derives` is Finding → Observation
   (`src/model.rs:200`), and there is no Finding → Source edge. That gap
   needs a relationship-set delta filed with the MVP-2 ticket.

## Impact

- **This change**: an openspec proposal only. No Rust, no spec delta
  (`.openspec.yaml` declares `skip_specs: true`), no behaviour change.
- **Follow-up tickets to file**: (a) implement the distilled session adapter
  with the target event shape, the redaction filter, the per-entry dedup key
  and the append-then-re-import test; (b) derive and state the per-import
  entry cap from the ledger's line budget; (c) validate the redaction list
  against a real transcript corpus; (d) verify the carrier precondition
  (agent sessions actually publish whisper bundles); (e) revisit raw capture
  only when the four conditions above hold.
- **Schema-visible change to ship with (a)**: the shipped turu event id
  folds in the whole-bundle digest; moving to a per-entry key re-ids every
  existing turu event, so it needs a `SOURCE_VERSION` bump and a migration
  note for ledgers that already hold turu events.
- **Surfaces touched later**: `src/adapters/` (a new adapter, or an
  extension of the turu adapter), `src/model.rs` (the event vocabulary), the
  kaya.adapters and kaya.model specs, and the understanding projection's
  evidence rendering.
- **Risks**, each with its trigger and detection signal:
  - the distilled surface may be too thin for the questions the evaluation
    implied — measured by asking three questions a reader must answer from a
    distilled record; if two of three fail, revisit the surface;
  - the redaction list is unproven against real data — detected by the
    corpus run in the validation ticket;
  - bundle-digest churn duplicating every entry on append if the per-entry
    key is not adopted — detected by the append-then-re-import test;
  - the ledger refusing a re-import with `kaya.ledger.id_reuse` whenever
    bytes change under a stable id (any id or actor change without a
    version bump) — detected by re-importing an unchanged bundle;
  - an unbounded, unredacted `topic` reaching the ledger — detected by the
    `topic` redaction fixture and by the per-field bounds.
- **Ownership**: the session store keeps the text. Kaya records observations
  and bounded, redacted attributes only.
