# Rule-of-5 review — kaya-d0n agent-session capture proposal

**Verdict: FAIL — CRITICAL count: 2** (High 3, Medium 6, Low 4)

Reviewed read-only: `openspec/changes/add-agent-session-capture/proposal.md`,
`tasks.md`, `.openspec.yaml`; `/tmp/kaya-d0n/{orient,plan,red}.md`; and the
facts the proposal rests on (`src/adapters/turu.rs`, `src/adapters/wai.rs`,
`src/model.rs`, `src/event.rs`, `src/ledger.rs`, `src/adapters/mod.rs`).
No files in the repo were modified; `git diff --stat src/` is empty and the
change directory contains only `proposal.md`, `tasks.md`, `.openspec.yaml`,
so the "no production code" constraint is respected.

The *decision* (distilled over raw) is defensible and its four unlock
conditions are a good shape. The failures below are in the **commitments**:
the proposal describes the attribute set of a surface it did not choose and
states a dedup rule that the shipped adapter contradicts. Both defects sit in
areas the ticket explicitly requires to be covered concretely, so the
proposal is not yet implementable as written.

---

## Critical

### C1 — The chosen surface's real attributes are unbounded and unredacted; the bounds/redaction in the proposal describe a different surface (wai's)

**Location:** `openspec/changes/add-agent-session-capture/proposal.md:12-14`,
`:21-24`, `:50-69`; facts at `src/adapters/turu.rs:123-129`,
`src/adapters/wai.rs:7`, `src/adapters/wai.rs:44`, `src/adapters/wai.rs:47`.

Quoted evidence — proposal:

> "The ownership rule is total and decides the surface: no adapter copies
> content into an event (`src/adapters/turu.rs:41` — "Entry text is never
> copied into an event"; the wai adapter copies only a bounded title plus a
> digest, `src/adapters/wai.rs:1-8`)."

> "2. **Transcript size bounds.** Bounded attributes only: a derived title is at
>    most 80 characters (the `TITLE_MAX_LEN` precedent) cut on a character
>    boundary, a digest is 16 hex characters of sha256 …"

> "1. **Secret redaction.** Every bounded attribute derived from session
>    material passes a redaction filter before it is copied."

Counter-evidence — the shipped turu event:

```rust
// src/adapters/turu.rs:123-129
let data = json!({
    "entry": entry_id,
    "topic": topic,          // <- unbounded, session-derived free text
    "scope": scope,
    "repo_key": repo_key,
    "bundle_digest": digest, // <- full 64-hex sha256, not 16
});
```

Three concrete problems:

1. **`topic` is unbounded and session-derived.** The turu adapter copies it
   verbatim with no length cap and no redaction. The proposal's rules are
   scoped to "bounded attribute", so on the surface it chose, the one
   attribute that can plausibly carry a pasted secret (`topic`) falls outside
   both the redaction filter and the size bounds. The proposal's own summary
   — "Ownership: the session store keeps the text. Kaya records observations
   and bounded, redacted attributes only" — is therefore false of the carrier
   it selected.
2. **The bounds are wai's attribute set, not turu's.** `TITLE_MAX_LEN = 80`
   and `TITLE_DIGEST_LEN = 16` are the *wai* artifact title and the digest
   **of that title** (`src/adapters/wai.rs:44,47,152-153`). The turu event
   has no title and its digest is the full 64-hex `sha256_hex` of the bundle
   bytes (`src/adapters/turu.rs:64`, `src/ledger.rs:404-406`). The proposal
   neither says it intends to change `bundle_digest` (a schema-visible change
   to shipped event bytes, which it lists only as a risk for the *actor* switch)
   nor specifies where a "derived title" comes from in a bundle that has none.
3. **"No adapter copies content into an event" overstates the quoted rule.**
   `src/adapters/turu.rs:41` says "Entry text is never copied into an event" —
   a narrower claim about the entry's *text* field. `topic` is content-derived
   and is copied. Since wai's own rationale is "bounded content-derived
   attribute != text ownership; wai keeps full text"
   (`src/adapters/wai.rs:7`), the rule as the code states it **permits a
   bounded content-derived attribute from any owner**, which materially weakens
   the proposal's central argument that only a whisper bundle can be
   distilled.

**Concrete fix:** (a) enumerate the exact target `data` map for the distilled
session event and its per-field bounds — e.g. `entry` (id, ≤64 chars),
`topic` (≤80 chars, cut on a char boundary, redacted), `scope` (≤64 chars),
`repo_key` (≤64 chars), `bundle_digest` (state 64 hex as shipped, or declare
the deliberate change and its migration); (b) state the redaction rule as
"every session-derived string attribute, bounded or not, passes the filter
before it is copied", and add `topic`/`scope` to the pattern fixtures;
(c) replace "no adapter copies content into an event" with the accurate rule
("no adapter copies the owner's text; bounded content-derived attributes are
allowed only where the owner keeps the full text and the attribute has a
declared bound") and cite `src/adapters/turu.rs:41` for what it actually says.

### C2 — The dedup key is not well-defined, and it contradicts the shipped adapter's event id and the incremental-import use case it exists to serve

**Location:** `proposal.md:71-75` (and the same claim in `/tmp/kaya-d0n/plan.md`
"sha256 over (bundle bytes digest, entry id, topic, ts)"); facts at
`src/adapters/turu.rs:64`, `:123-129`, `:218-222`; `src/ledger.rs:207-256`.

Quoted evidence — proposal:

> "3. **Incremental import dedup by content hash.** The dedup key is the sha256
>    over the source bytes digest plus the entry identity (entry id, topic,
>    ts). Because the event id is derived from that content, re-importing an
>    unchanged session produces the same ids and therefore no new events;
>    incremental imports need no separate bookkeeping."

Counter-evidence — the actual event id:

```rust
// src/adapters/turu.rs:218-222
fn event_id(project: &str, timestamp: &str, reference: &str, data: &serde_json::Value) -> String {
    let identity =
        format!("{TOOL}\x1f{SOURCE_VERSION}\x1f{project}\x1f{timestamp}\x1f{reference}\x1f{data}");
```

Concretely:

1. **The id is not derived from the stated key.** It also folds in tool,
   tool version, project, the *normalised* timestamp, `reference` (the bundle
   digest) and the whole serialized `data` map (`scope`, `repo_key`,
   `bundle_digest` included). The proposal's key omits them, so "the event id
   is derived from that content" is false as written.
2. **The key is ambiguous.** "sha256 over the source bytes digest plus the
   entry identity" does not say whether "source bytes" is the whole bundle
   file or the entry's own bytes, nor how the digest and the three fields are
   combined (separator, order, encoding). Three implementers will produce
   three different keys.
3. **The bundle-wide digest breaks incremental import.** `reference` is the
   digest of the *entire bundle file* (`src/adapters/turu.rs:64`), and it is
   inside both the id and `data`. Appending one entry to a session therefore
   changes the digest, changes every prior entry's id, and re-import emits a
   fresh event for every previously captured entry — the opposite of the
   stated goal. The claim "re-importing an unchanged session produces the same
   ids" is true only for a byte-identical bundle, which is not what
   "incremental import" means. The wai adapter deliberately avoids this by
   hashing "the path-identity payload only, so content never moves the id"
   (`src/adapters/wai.rs:7`, `:212-217`).
4. **The dedup consequence is not stated.** The ledger dedupes byte-exactly
   and refuses the same id with different bytes
   (`src/ledger.rs:207-215`, `:250-256`), so a re-import that changes content
   under a stable id is not merely a duplicate — it is a hard refusal.

**Concrete fix:** define the dedup key explicitly and per-entry, e.g.
"dedup/id content = sha256 over the UTF-8 bytes of `tool \x1f version \x1f
project \x1f <normalised ts> \x1f <entry id> \x1f <topic> \x1f <scope>` —
never over the whole bundle file; the bundle digest travels as provenance
only." Then state the append case as a required test: "import a bundle with
entries A,B; append C; re-import; assert exactly one new event and unchanged
ids for A and B." Add the `kaya.ledger.id_reuse` behaviour to the risks
section for any key change.

---

## High

### H1 — Actor typing contradicts the closed set's semantics and will break replay against the ledger

**Location:** `proposal.md:77-83`; facts at `src/event.rs:108-115`,
`src/adapters/turu.rs:136`, `:218-222`, `src/ledger.rs:250-256`.

> "Actor: the closed `ActorType` set gains its intended use — agent-session
> captures are `Agent` (today's turu adapter emits `Tool`), with the actor
> name naming the session tool. Changing a shipped adapter's actor bytes is a
> deliberate follow-up ticket, not part of this proposal."

`ActorType` is documented by meaning, not by availability
(`src/event.rs:108-114`): `Tool` = "A program emitted the observation",
`Human` = "A person emitted the observation", `Agent` = "A model-driven agent
emitted the observation". A capture observation is emitted by the *adapter
program* reading a file; the session's agent did not emit it. Typing it
`Agent` attributes the observation to the agent, contradicting the closed
set's own semantics — and the set needs no change at all for that, so
"the closed `ActorType` set gains its intended use" is inaccurate.

The migration consequence is also mis-stated. `event_id` does not include the
actor (`src/adapters/turu.rs:218-222`), so switching `Tool` → `Agent` leaves
the id unchanged while changing the bytes; the ledger then refuses the
re-import with `kaya.ledger.id_reuse` ("ids are never reused — append a
correction event naming … in `data.supersedes`", `src/ledger.rs:250-256`).
The proposal mentions "a migration story" only as a risk bullet, without this
mechanism.

**Concrete fix:** keep `ActorType::Tool` for adapter-emitted capture events
and record the session tool in `provenance.tool` (which already names the
adapter) or in a dedicated `data` field; if `Agent` is genuinely wanted, say
which event is agent-authored rather than adapter-observed, require a
`SOURCE_VERSION` bump so ids change, and specify the `data.supersedes`
correction path for already-stored events.

### H2 — The linking hooks overstate what the closed relationship set can express

**Location:** `proposal.md:85-89`; facts at `src/model.rs:200-202`,
`:214-220`, `:439-441`.

> "5. **Hooks for MVP-2 relationship linking.** … With the `cites`
>    relationship shipped by kaya-q6s, a Decision can cite the session Source,
>    and Findings attach through the influence edge; MVP-2 generation reads
>    those links rather than the transcript."

`cites` is `Decision → Source` and `requires_exact_pair` is true for it
(`src/model.rs:202`, `:214-220`), so the Decision→Source half is real. The
Finding half is not: `influences` is `Decision → Finding`
(`src/model.rs:200`), `derives` is `Finding → Observation`, and there is no
`Finding → Source` edge in the closed set (`src/model.rs:34`). A Finding
cannot currently point at a session Source at all.

Additionally, the Source node created from `capture.entry.observed` carries
only the `entry` attribute (`src/model.rs:439-441`, fields `&["entry"]`), so
the "join key" the proposal advertises — entry id, topic, timestamp, scope —
exists in the *event*, not in the *graph node*; a projection can join only on
the subject id unless the projection reads raw events.

**Concrete fix:** state the hook precisely: "today the only link from a
session capture into the graph is `cites` from a Decision to the Source node
`<project>/Source/entry-<entry id>`; linking a Finding to a session requires
a spec delta to the closed relationship set, filed with the MVP-2 ticket."
Either drop "Findings attach through the influence edge" or specify the new
relationship kind and its direction.

### H3 — The carrier precondition is asserted, not stated, and it is the falsifier the analysis named

**Location:** `proposal.md:21-24`, `:35-39`; `/tmp/kaya-d0n/red.md`
falsifier 3; `/tmp/kaya-d0n/orient.md` "Uncertainties".

> "The distilled surface is not a consolation prize: a whisper bundle already
> carries the session's structure (entry id, timestamp, topic, scope, bundle
> digest) — which session, when, about what — and the only thing it omits is
> the transcript body."

> "Meanwhile the turu adapter already observes whisper bundles — which is how
> this session's own handoffs travel — but nothing names an agent session as
> a first-class source"

`red.md` names the carrier gap as a falsifier: "if session material does not
flow through whisper bundles, 'distilled' has no carrier". The proposal never
states this as a precondition, never lists it in Risks, and simultaneously
asserts both that "kaya currently captures none of it" and that the turu
adapter already observes the bundles that carry the session structure. Those
two sentences cannot both be true unless the carrier assumption is false.

**Concrete fix:** add an explicit precondition to the Decision section —
"distilled is admissible only if agent-session material is published as turu
whisper bundles; if a session lives only in pi/terapia, distilled has no
carrier and the raw path (or a bundle writer) must be reconsidered" — and add
the same item to Risks with the measurement from `red.md` falsifier 1 (three
questions; if two of three fail, revisit).

---

## Medium

### M1 — Unverified and undefined terms are stated as fact

**Location:** `proposal.md:35-39`, `:41-43`, `:85-89`.

"The external evaluation called agent sessions the richest available source of
research material" is asserted as fact, while `orient.md` records it as "an
unverified claim; this proposal treats it as motivation, not evidence" — the
proposal does not carry that caveat. "MVP-2", "the external evaluation",
"the session store", "text owner", "raw surface" are used without definition
or citation; "MVP-2" in particular is meaningless to an implementer.
**Fix:** mark the evaluation claim as unverified motivation and define or
cite each remaining term in one line (or drop it).

### M2 — The redaction pattern list has plausible gaps and an ambiguous replacement semantics

**Location:** `proposal.md:50-62`.

`sk-[A-Za-z0-9]{16,}` misses OpenAI project keys (`sk-proj-…`, hyphenated)
and `sk-` keys of other lengths; `AKIA[0-9A-Z]{16}` covers only AWS *access
key IDs* (not `ASIA…` temporary credentials and not the 40-character secret
access key, which cannot be pattern-matched in general); there is no
`-----BEGIN PGP PRIVATE KEY BLOCK-----` or `PuTTY-User-Key-File` arm. More
importantly "A redacted attribute is replaced by a fixed placeholder" is
ambiguous between replacing *the matched secret* and replacing *the whole
attribute*, and the required fixture ("the placeholder replaces rather than
leaks") does not disambiguate.
**Fix:** state replace-the-match (with the match replaced by a fixed marker,
surrounding attribute preserved) versus replace-the-attribute explicitly and
pick one; add the missing key families; add a "cannot be pattern-matched —
treated as secret by default" clause for high-entropy values (base64/hex
strings ≥32 chars) so a missed family is not a silent leak.

### M3 — The 500-entry cap is an unjustified number and the wrong failure mode

**Location:** `proposal.md:64-69`.

"A bundle contributing more than 500 entries in one import is refused
atomically … naming the bundle." No source for 500, and refusing the whole
bundle discards a session's entire evidence whereas the risk being bounded
is only the *per-entry* size. The label itself is fine
(`src/adapters/mod.rs:19`: `kaya.adapters.failure_atomic`).
**Fix:** justify the number or replace it with a bound derived from the
event-size budget; state whether over-cap bundles are refused wholesale or
truncated at a stated boundary (and say which, with the consequence).

### M4 — The digest length in the proposal contradicts the shipped digest

**Location:** `proposal.md:64-66` vs `src/adapters/turu.rs:64`,
`src/adapters/turu.rs:128`.

"a digest is 16 hex characters of sha256" is the wai *title* digest
(`src/adapters/wai.rs:47`); the shipped bundle digest is the full 64-hex
sha256 of the bundle bytes. If the intent is to change `bundle_digest`, that
is a schema-visible change to shipped event bytes and must appear in Impact
with a migration story; if not, the sentence should say 64 hex.
**Fix:** pick one and say so explicitly.

### M5 — The risk list omits the two concrete failure modes found in this review

**Location:** `proposal.md:104-107`.

Risks cover thinness, unproven redaction, and actor bytes; they do not cover
(a) bundle-digest churn duplicating every entry on append (C2), (b) the
ledger's `id_reuse` refusal when bytes change under a stable id (C1/H1), or
(c) unbounded `topic` reaching the ledger unredacted (C1).
**Fix:** add those three risks with their triggers and detection signals.

### M6 — The proposal never states the target event shape, so the implementer must infer it

**Location:** `proposal.md:21-24`, `:45-69`.

No section gives the concrete `data` map, `subject` id form, and `provenance`
of the future distilled session event; the reader must reconstruct it from the
turu adapter. This is the direct cause of C1 and part of H2.
**Fix:** add a short "Target event shape" block listing type, subject, actor,
and each `data` key with its bound and its redaction obligation.

---

## Low

### L1 — "the closed `ActorType` set gains its intended use" is inaccurate

**Location:** `proposal.md:80-81`. `Agent` already exists
(`src/event.rs:114`); nothing is added to a closed set. **Fix:** "the closed
set already contains `Agent`; the question is which captured observations are
agent-authored and which are adapter-authored."

### L2 — External, unverifiable citations are stated as shipped facts

**Location:** `proposal.md:87` ("the `cites` relationship shipped by kaya-q6s"),
`proposal.md:16-17` ("the same classification the external evaluation gave
kaya-q5e"). `cites` does exist in this tree (`src/model.rs:189`), but the
ticket id and the external evaluation are not checkable here.
**Fix:** cite the in-repo location (`src/model.rs:189`) and mark the
external-evaluation comparison as an analogy.

### L3 — "the only thing it omits is the transcript body" overstates the bundle's content

**Location:** `proposal.md:21-24`. A bundle also carries `scope`,
`repo_key`, and a `freeform` section that the adapter explicitly never reads
(`src/adapters/turu.rs:8-9`, `:66-73`). **Fix:** say what is carried and what
is out of the adapter's contract.

### L4 — `tasks.md` mirrors the proposal's defects and adds no independent verification

**Location:** `openspec/changes/add-agent-session-capture/tasks.md:1.3`,
`3.1-3.4`. Task 1.3 claims the five areas are "fixed as design commitments",
but the commitments are the ones found defective above; the task list also
marks 3.1–3.4 as pending even though the proposal is otherwise final.
**Fix:** after the proposal is corrected, have 1.3 assert the *corrected*
commitments and add a task requiring an append-and-re-import dedup test and
a `topic` redaction fixture.

---

## Verdict block

- **overall**: FAIL
- **critical_count**: 2
- **counts**: Critical 2, High 3, Medium 6, Low 4
- **top fixes**:
  1. (C1) State the target distilled event's exact attribute set with per-field
     bounds, and apply redaction to every session-derived string (not only
     "bounded" ones); state the ownership rule as the code states it.
  2. (C2) Define the dedup/id content hash precisely, per entry, excluding the
     whole-bundle digest, and add the append-then-re-import test.
  3. (H1) Keep capture actors as `Tool` (or justify `Agent` against
     `src/event.rs:108-114`) and specify the version-bump/supersedes
     migration for the byte change.
  4. (H2) Narrow the linking claim to `cites` Decision→Source; say that
     Finding→Source needs a relationship-set delta.
  5. (H3) Add the carrier precondition and the `red.md` measurement to the
     Decision and Risks sections.
- **no-production-code**: respected (verified: `git diff --stat src/` empty;
  change dir holds only `proposal.md`, `tasks.md`, `.openspec.yaml`).
- **not run here** (per assignment): `openspec validate --changes --strict`,
  cargo test/clippy — orchestrator's checks.
