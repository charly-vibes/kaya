# Design: add-collection-kinds

## Context

The publication layer already draws a hard boundary the proposal must
respect: `cmd_publish` reads `docs/research` through `read_documents`
(fixed list: index.md, overview.md, understanding.md, reproduction-package.md,
handoff.md, then experiments/*.md), resolves the default declarations, and
builds — it never touches the ledger (`read_build_ledger` is reachable only
from build/ir). Two precedents prove what crosses that boundary:

- the glossary composes from compile-time vocabularies
  (`EntityType::ALL`, `Relationship::ALL`, `DOCUMENT_KINDS`) — data that
  needs no ledger because it pins the grammar;
- the research-map block compiles per-ledger rows (relationships,
  artifacts, runs, sources) into `docs/research/index.md` at `kaya build`
  time — per-ledger data, made ledger-free by the time publish reads it.

Today the collection arm of `partition_declarations` refuses every name but
glossary with `kaya.publications.tier_unimplemented`, and
`render_collection_page` debug-asserts the glossary name. The compiled
documents already carry, per block, their `kaya/...` node ids — parsed for
event attribution — and the reuse and report projections already produce
`reuse-handoff.technical-reference` and `exploration.experiment-record`
blocks, though the dogfood ledger yields rows for neither. Both publication
doors read the same fixed five-file list today (check filters its
expected-IR map by the identical order), so the moment a tree carries
technical-reference.md or brief.md the projection has emitted blocks the
publication layer of either door cannot compose — and extending one list
without the other would make the doors disagree, which the parity task
prevents.

The design is therefore a routing problem: for each of the eight refused
kinds, decide render (with which ledger-free source and row shape) or
refuse (with which recorded justification).

## Decision log

### D1 — topic-map renders from the research-map block, under fact-once

Every declared relationship is already rendered in
`orientation.research-map`; the invariant `relationship_rendered` pins
that. The same file is in publish's read list, so the topic-map needs no
new producer and no read-list change. But the `reference` — and, in the
dogfood tree, `architecture-design` — products' admission rows also admit
the research-map (composition assigns the body to the alphabetically
first emitted product, so architecture-design owns it today), and
composition without duplication says each fact body appears once: a
topic-map that re-embedded the edge statements would duplicate them
wherever either product embeds the block body. The renderer therefore
treats the research-map body as owned:

- rows are per topic — one per distinct node id in the block, with its
  entity type and the relationship types it participates in;
- the edge statements themselves are cross-referenced to the page that
  embeds the body, and embedded by the topic-map only when no page in the
  tree composes the research-map (then the topic-map is their only
  rendering).

This generalizes the shipped ownership mechanism from products to the
whole tree without touching the admission table: the topic-map is a map of
the graph's shape — topics, types, participation, counts — not a second
copy of the edges.

### D2 — knowledge-index indexes the published corpus, not the ledger

A knowledge-index needs a per-type census; per-type node rows are
per-ledger data. Two ledger-free routes existed:

- a new compiled census block (a `kaya build` producer writing a new
  document kind) — rejected: it adds a document kind, forcing a
  `kaya-documents` delta, a new producer, and a read-list entry, for data
  the compiled blocks already carry;
- **chosen**: the census of node ids the compiled blocks already carry.
  `read_documents` parses every block body's `kaya/...` ids for event
  attribution; the renderer groups them by entity type under the closed
  thirteen-entity headers with a count per type and reports none for
  empty types. Ids whose type segment is outside the closed entity set
  are not census rows.

The scope sentence matters and is pinned: the index covers the topics the
published tree names, exactly as a book index covers the book. It is
complete for its scope by construction and it never implies ledger
coverage. In dogfood the page appears immediately — Question, Hypothesis,
Model, Experiment, Run, Source, and Artifact ids all appear in the
compiled blocks; the six types with no rows report none.

### D3 — example-gallery is producer-backed and omission-tolerant

The `exploration.experiment-record` producer exists
(`src/projection/report.rs`, `RECORD_BLOCK`) and experiments/*.md are
already read; the dogfood dossier carries only the experiment
specification, no record block. The renderer ships against the block's
shape (one row per recorded experiment with its run, artifacts, and
observations); with no admissible rows the declared gallery is omitted
with `no-backing-document` — the `tutorial` precedent, which composes the
same kind. Implementing now is honest because the producer-backed path is
defined; refusing would pin "unimplemented" onto a kind whose only
missing piece is dogfood rows, and the same rows un-omit `tutorial`
(follow-up ticket).

### D4 — api-catalogue rides the producer that exists; the read list grows with it

`reuse-handoff.technical-reference` renders spec ids only
(`src/projection/reuse.rs:391-392` — entry ids belong to the separate
`reuse-handoff.knowledge-brief` block, `reuse.rs:396-401`) into
technical-reference.md — a file publish never reads today. The catalogue's
rows are exactly that block's rows (one per spec id with its resolvable
reference). The change that adds the renderer extends the read list — publish's fixed list in
`src/publication/source.rs` and check's `RESEARCH_READ_ORDER` in
`src/verification/mod.rs`, together, so the doors cannot diverge again.
This also lets `how-to` and `reference`, whose admission rows already
name the kind, compose a block they could never see before; the dogfood
tree carries no technical-reference.md, so no existing behaviour changes
there. The scout separately flagged brief.md missing from both lists;
that gap changes `tutorial` emission and is filed as its own ticket, not
smuggled into this one.

### D5 — the four refusals are recorded, each with its reason

- **capability-catalogue** — there is no Capability entity in the closed
  thirteen-entity set, no admissible rows, no producer. A renderer would
  emit a page because the taxonomy says so: the exact failure mode the
  capability exists to prevent.
- **compatibility-matrix** — nothing compiled carries per-item status.
  The facts live in authored `docs/src/status.md` (outside
  `docs/research`, the human's bytes) and in ledger verification facts;
  compiling them is possible, but the `status-compatibility` product
  already answers "what can I rely on" from its pinned admission row, so
  the collection would either duplicate that product or open a second
  source class publish does not own. Refuse.
- **feature-index** — no Feature entity and no feature section in any
  compiled block. Nothing to index. Refuse.
- **related-projects** — every node id in the dogfood record is
  `kaya/…`; the handoff block lists handoff artifacts, not projects. A
  renderer would render an empty page. Refuse.

The distinction pinned by the delta: **refusal** for "no source can exist
yet" (undeclarable, labeled), **omission** for "the source exists but this
tree has no rows" (declarable, reported). capability-catalogue,
compatibility-matrix, feature-index, and related-projects are refusals;
example-gallery and api-catalogue are omissions until their producers
yield rows.

### D6 — the gate becomes a dispatch; the refusal names what exists

The glossary-only gate (`decl.name != "glossary"`) becomes membership in
the implemented set; the `tier_unimplemented` message changes from "only
the glossary collection is implemented" to naming the declared kind and
the implemented kinds, so the refusal is self-describing. Everything
around the gate stays: the tier-fields refusal ahead of it, the block
vocabulary and pre-write validation, the plan-then-write phases, the sort
order, and the byte-untouched refusal guarantee.

### D7 — export rows mirror pages

Collection export rows are hardcoded `{emitted: true, name}`. With
omission now reachable, the rows carry the same emitted flag and reason
the summary reports, projected from the same source map — the
one-source-many-formats requirement applies to collections like it does
to products.

### D8 — collections stay field-free and question-free

Declarations keep no `includes`/`members` for collections (the stray-field
refusal stays); each renderer binds its own pinned source in code, exactly
as the glossary binds its compile-time vocabularies. No reader question is
added — `reader_question` stays product-only, and the glossary page has
never carried one. Widening the admission table to collections would add
a second binding mechanism for eight names; the per-kind contract in the
spec delta is the binding, and the renderer match is its one
implementation.

## Alternatives rejected

- **A1 — publish-time ledger queries.** Rejected outright: it breaks the
  boundary that makes publish a pure function of `docs/research`, and it
  would make publish and check read different inputs. The research-map
  precedent already shows per-ledger data crossing the boundary by
  compilation, which is where any future ledger-derived collection must
  cross it too.
- **A2 — a new compiled census block for knowledge-index.** Rejected by
  D2: a new document kind plus producer plus read-list entry to move data
  the blocks already carry.
- **A3 — reading authored `docs/src/**` (status.md) for
  compatibility-matrix.** Rejected: the publication layer reads compiled
  research documents; authored trees are the human's bytes and a second
  source class would need its own vocabulary, checks, and attribution
  rules. If the matrix is ever wanted, its route is a compiled block with
  a producer, priced like the research-map.
- **A4 — declaring the four refused kinds and omitting them every build.**
  Rejected by D5's distinction: an omission row every build reads like
  breakage, and the tier vocabulary would promise pages no release can
  produce. A refusal says "not this release"; an omission says "not this
  tree".
- **A5 — implementing all eight now.** Rejected: four have no admissible
  source, and a stub renderer for them is the stub-page failure mode in
  its purest form.

## Risks

- **Scope confusion on the knowledge-index.** Trigger: a reader expects
  ledger-global coverage. Signal: dogfood review; mitigation is the scope
  sentence on the page, never a ledger read.
- **Fact-once ownership order.** Trigger: `reference` and
  `architecture-design` (both admit the research-map; allocation assigns
  the body to the alphabetically first emitted) and `topic-map` all
  composing the research-map. Signal: the
  `topic_map_cross_references_embedded_edges` scenario test pins the
  allocation; without it, ownership silently depends on tier render
  order.
- **Read-list drift between publish and check.** Trigger: one list
  extended without the other. Signal: the two lists are in different
  modules; the implementation ticket changes both in one commit and the
  check pipeline fails loudly on divergence.
- **Omission fatigue.** Trigger: example-gallery and api-catalogue omit
  on every dogfood build until rows exist. Signal: the build summary
  names them with `no-backing-document`; the follow-up dogfooding tickets
  are the fix, and the omitted rows double as their acceptance check.
