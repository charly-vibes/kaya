# Proposal: add-document-ir-commit-gate

## Why

kaya-aaz recovered four ideas from the origin conversation's later turns
(chatgpt `6afa5381`): a structured document IR (DocIR) as kaya's central
artifact, doctests as first-class objects, documentation impact analysis as a
commit gate, and Org as an authoring frontend with `KAYA_*` properties. The
document taxonomy deliberately parked them: they change kaya's central
artifact, not its document list. The taxonomy has landed; this proposal makes
the scope decision.

Today the compiler renders documents straight from ledger and model into
markdown managed blocks. The only evidence-driven loop is authored staleness
(`kaya:cites` → supersede → stale → acknowledge); the build tracks
recomputed-versus-unchanged per document, but nothing answers which documents
a change affects. Three consequences follow. Structural facts — which records exist, what each
depends on, which validations hold — live only implicitly in the renderer and
must be re-derived by reparsing rendered markdown. Nothing can answer "which
documentation does this change affect?" mechanically. And an example in a
document is inert text: nothing distinguishes a runnable contract from prose,
so documented commands rot silently.

## Central-artifact decision

**Adopt a scoped DocIR as kaya's central documentation artifact — derived,
never authoritative.**

- The append-only ledger stays kaya's epistemic source of truth and the
  research model stays the knowledge graph. The origin conversation's own
  distinction holds: the knowledge graph and the build/dependency graph have
  different semantics; kaya needs the second to validate and recompile, and
  DocIR is what connects them.
- DocIR is the typed intermediate the compiler materializes between
  (ledger, model, authored citations, document sources) and every consumer of
  documentation structure: renderers, the doctest harness, the impact gate.
  Records carry stable ids, content hashes, event-id provenance, and
  dependency edges; validation state (current / stale / invalid / unverified /
  waived) lives on records.
- Rendered markdown stays the human surface. The deployed projection
  guarantees — byte-preservation, managed grammar, deterministic generation,
  staleness scope, scoped acknowledgment — are unchanged; the IR mechanizes
  them instead of replacing them. Authored prose remains authoritative in the
  document files: an authored-block IR record references its location
  (document, block), carries a region hash and its `kaya:cites` dependencies,
  and never copies the bytes.
- Deliberate deviation from the origin sketch, which made DocIR
  "authoritative for curated documentation content": deployed kaya postdates
  that sketch and made authored markdown the authored-truth surface (managed
  blocks, acknowledgment events). DocIR is authoritative for the
  documentation *model* — records, dependencies, validation state — not for
  authored bytes.

## What Changes

- `kaya-projection` — ADDED requirement: the compiler SHALL materialize the
  document IR (stable ids, content hashes, event-id provenance per record,
  authored blocks by reference, dependency edges), deterministically, and
  render from it, so structural analysis never reparses rendered markdown.
- `kaya-documents` — ADDED requirement: doctest execution contracts. An
  example MAY declare runner, steps, and expected output; a declared doctest
  becomes a first-class IR record; expected output changes only through a
  human-recorded supersede, never to make a failing run pass; runs execute
  through the controlled test-runner source in isolation and are recorded as
  test-runner facts (kind `doctest`), so `kaya.verification` counts them
  under `test-runner` with no vocabulary change.
- NEW capability `kaya-impact` — the documentation commit gate:
  change set → impact set over IR dependency edges under explicit project
  policy → required dispositions (closed vocabulary: `updated`, `validated`,
  `reviewed`, `waived`) → mechanical validation → gate result (`pass`,
  `review`, `fail`). The report is deterministic and machine-readable, naming
  the rules that fired, the checks that ran, and every disposition and waiver.
  Hard blockers are reproducible results only; `reviewed` and `waived` are
  human-recorded events under the `verification` family, mirroring
  `verification.acknowledged`; an unacknowledged stale authored block is an
  unmet obligation. `kaya impact --staged | --range <revset>` exits nonzero
  exactly on `fail`, so a lefthook gate can wire it alongside `ah check`,
  pretender, and `spk lint` (hook wiring is repo policy, decided at
  implementation).

Consequences for kaya.projection: every existing invariant is preserved.
`incremental_scope` is strengthened — invalidation is computed at record
granularity from dependency edges, so a rebuild recomputes exactly the
records the frontier affected. `source_map` is mechanized — the emitted map
derives from record provenance instead of the tracing currently coupled to
the render walk. `kaya:cites` markers become dependency declarations feeding
the IR dependency graph — the model's `cites` attribution relationship stays
as it is. Staleness and acknowledgment keep their trigger scope and clearing
event; the stale flag becomes IR validation state.

Consequences for kaya.documents: the family/kind vocabulary, the omission
rules, and the reuse-reference rule are unchanged; they are evaluated on IR
records before rendering. Doctests are the one new content surface, and their
results stay facts, not policy — `kaya.verification` still evaluates no
threshold or allowlist; the gate owns the decision.

Consequences for kaya.publications: publication pages compose from record
documents, so their impact arrives through the same dependency edges; no
delta in this change.

## Impact

- Specs: kaya-projection ADDED (document IR materialization); kaya-documents
  ADDED (doctest execution contracts); kaya-impact ADDED (new capability:
  impact mapping, dispositions, gate result contract, with layer sidecar).
- Code (implementation tickets, not this proposal): IR materialization in
  `src/projection/`, doctest records and the controlled runner, the gate in a
  new `src/impact` module, `kaya impact` CLI subcommand.
- Docs: `docs/src/document-taxonomy.md` Deferred section shrinks to the Org
  rejection with its revisit conditions; book regenerated.
- Tickets: kaya-aaz (this proposal).

## Out of Scope

- **Org authoring frontend with `KAYA_*` properties — rejected for now.** No
  Org authoring user exists; markdown with managed markers is the deployed
  authoring surface; a parser frontend before the IR interchange has a stable
  form would be speculative surface area — the origin conversation's own
  warning against building a new DITA applies. Revisit when (a) the IR
  serialization has a stable, versioned form and (b) a real Org authoring
  need appears. The origin's Babel lesson is recorded: authoring conveniences
  must not become the execution engine; doctests run through the controlled
  harness.
- **Semantic and LLM-assisted impact checks** (contradictions, audience
  adequacy, terminology drift): deferred. The origin conversation is explicit
  that an LLM asserting documentation is correct is not evidence, and that
  false positives train developers to bypass the gate. The MVP hard gate is
  deterministic; advisory findings produce `review` results without blocking.
- **A persisted IR interchange format as an artifact of record:** the IR is
  materialized deterministically from the ledger. This proposal pins
  determinism, hashing, and provenance — not a file format. A serialization
  may exist for tests, `kaya ir` inspection, and tooling without becoming
  authoritative.
- **Generic markdown linting** (link checkers, style rules): out of scope.
  The gate operates on the change → impact relationship over the IR, not as a
  markdown linter.
- **Two-stage local/CI deployment:** one gate command is specified; how
  checks split between a staged local pass and a full CI pass is wiring
  policy for the implementation tickets.
