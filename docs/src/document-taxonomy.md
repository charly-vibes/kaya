# Documentation taxonomy

kaya manages two things at once: the research record, and the client-facing
documentation compiled from it. This page records the document taxonomy that
separates them — eleven record-facing document kinds in four families, and
twelve reader-facing products in three tiers — with what exists today and
what is deferred.

## Where this came from

The taxonomy was worked out in kaya's origin conversation (chatgpt, November
2026, transcript `6afa5381-da7c-83e9-8f6b-9dfb71a97d9d`) and then lost: the
in-repo distillation
(`.wai/projects/kaya/research/2026-11-16-origin-conversation-event-driven-research-docum.md`)
kept six architectural decisions and pointed at this book, which named four
projection outputs and specified none. This page recovers the taxonomy from
the transcript; the behavioral contract lives in
[`add-document-taxonomy`](../../openspec/changes/add-document-taxonomy/proposal.md).

| Revision in the conversation | What it added |
|---|---|
| "What kinds of documents exist?" | Eleven document kinds in four families — orientation, exploration, understanding, reuse and handoff (the conversation called its numbered list "seven document families") |
| "Think in user questions, not document types" | kaya as an information access layer; reader questions as the organizing axis |
| Diátaxis validation (Kubernetes, Python, Next.js, Rust, MDN) | Eight primary types, plus the types-versus-sections-versus-collections distinction |
| LLM-authored documentation revision (DITA, Good Docs, docs-as-product) | Twelve reader-facing products, cross-cutting collections, publication packages, and five independent dimensions: audience, purpose, genre, relationships, epistemic status |

The conversation's own conclusion: retain the four Diátaxis modes as writing
principles, but do not adopt them as the taxonomy.

## Two layers

- **Record layer** — `kaya.documents`: projections over the ledger and model.
  These answer what we intended, what happened, what we learned, and what we
  chose.
- **Publication layer** — `kaya.publications`: client-facing products composed
  from record documents. These answer a visitor's questions.

A product is never a document kind, and a collection is never a product.

## Record layer: four families, eleven kinds

| Family | Kind | Reader question | Source records |
|---|---|---|---|
| orientation | `project-overview` | What is this, why does it exist, what state is it in? | Project, Question, current Findings |
| orientation | `research-map` | What is the shape of the investigation? | every admitted entity and typed relationship |
| exploration | `exploration-brief` | What triggered this question and what would change our mind? | Question, Idea, Hypothesis |
| exploration | `experiment-specification` | How will we investigate it? | Experiment, Hypothesis, intent relationships |
| exploration | `experiment-record` | What actually happened? | Run, Artifact, observation events |
| understanding | `findings-and-analysis` | What does the evidence tell us? | Finding, Claim, Observation |
| understanding | `decision-record` | Why did we choose this direction? | Decision, Source, Finding |
| reuse-handoff | `technical-reference` | What exactly is this interface or constraint? | Source references (specodelic specs, model) |
| reuse-handoff | `reproduction-package` | How do I reproduce this run? | Run revision, commands, expected outputs |
| reuse-handoff | `knowledge-brief` | What is reusable, and where? | Source references (turu entries) |
| reuse-handoff | `handoff` | What is the state and what comes next? | Source references (wai handoffs) |

The distinction that matters most: **what we intended to do is not the same as
what we actually did.** `experiment-specification` and `experiment-record` are
separate projections; neither is derived from the other.

Status today: `orientation.research-map` renders every declared relationship as
a typed edge naming both endpoints and lists admitted nodes that carry no
edge, inside the `orientation.research-map` block of `docs/research/index.md`.
The report implements the specification/record split as two generated blocks:
it compiles `exploration.experiment-specification` from declared intent and
`exploration.experiment-record` from runs, observations, findings, and
interpretations, omitting any kind with no records. The project-level
`docs/research/understanding.md` implements both understanding kinds:
`understanding.findings-and-analysis` renders Finding, Observation, and Claim
records (runs and artifacts never enter it), and
`understanding.decision-record` renders decisions plus their evidence — source
revisions and influenced findings — with a Source row only when both `path` and
`commit` are present and non-blank. Findings canonically precede the decision
record, and a document left holding neither block is deleted rather than
written empty. The project-level `docs/research/overview.md` implements
`orientation.project-overview`: Project, Question, and Finding records in three
sections in a fixed order — Project, Questions, Findings — with the rows of
each section id-sorted, empty sections omitted, and no document at all when
none of the three groups has a record. `exploration-brief`,
`technical-reference`, and `knowledge-brief` are not generated yet;
`reproduction-package` and `handoff` render reference rows from the ledger.

## Publication layer: twelve products

| Product | Reader question | Composed from |
|---|---|---|
| `overview` | What is this and why does it matter? | `orientation.project-overview` |
| `getting-started` | How do I reach a first result? | `orientation.project-overview`, `reuse-handoff.reproduction-package` |
| `tutorial` | Can you teach me through a guided outcome? | `exploration.experiment-record`, `exploration.exploration-brief` |
| `how-to` | How do I accomplish this task? | `reuse-handoff.technical-reference`, `reuse-handoff.reproduction-package` |
| `concept` | Why does it work this way? | `understanding.decision-record`, `understanding.findings-and-analysis` |
| `reference` | What exactly is this? | `reuse-handoff.technical-reference`, `orientation.research-map` |
| `example` | What can I do with it? | `exploration.experiment-record` |
| `troubleshooting` | Why is this broken and how do I fix it? | `understanding.findings-and-analysis` |
| `faq` | What do people keep asking? | `understanding.findings-and-analysis`, `understanding.decision-record` |
| `status-compatibility` | What can I rely on? | `orientation.project-overview`, `exploration.experiment-record` |
| `release-migration` | What changed and what must I do? | `orientation.project-overview`, `understanding.decision-record` |
| `architecture-design` | How is it structured and why? | `understanding.decision-record`, `orientation.research-map` |

Status today: `kaya publish` reads `index.md`, `overview.md`,
`understanding.md`, `reproduction-package.md`, `handoff.md`, and the experiment
dossiers in that fixed order, keeping only generated blocks in the document
vocabulary — so the overview and understanding kinds can back any product that
admits them. Today's declarations are the eleven products `overview`,
`reference`, `getting-started`, `how-to`, `concept`, `example`,
`troubleshooting`, `faq`, `status-compatibility`, `release-migration`, and
`architecture-design` — each including exactly its admission row — plus two
packages: `project-site` over `overview` and `reference`, and
`developer-guide` over the six developer products. The committed ledger emits
`overview`, `reference`, `getting-started`, `how-to`, `status-compatibility`,
`release-migration`, and `architecture-design`: `getting-started` renders
`orientation.project-overview` (the orientation slice of the ledger) and
`reuse-handoff.reproduction-package`, and `architecture-design` renders
`orientation.research-map`; `overview`, `how-to`, `reference`,
`status-compatibility`, and `release-migration` cross-reference the page that
owns their shared body — `getting-started` for the overview and
reproduction-package kinds, `architecture-design` for the research map —
instead of repeating it. An include with no producer is skipped rather than
stubbed, so `reference` loses `reuse-handoff.technical-reference`,
`architecture-design` loses `understanding.decision-record`, and
`status-compatibility` and `release-migration` lose
`exploration.experiment-record` and `understanding.decision-record`.
`concept`, `example`, `troubleshooting`, and `faq` are omitted with reason
`no-backing-document`, because the committed ledger holds no Finding, Claim,
Observation, Decision, or experiment record, so no `understanding.md` is
compiled; `developer-guide` lists those members as unavailable and never stubs
them. `tutorial` stays undeclared until its exploration documents have
producers.

## Collections and packages

Collections group related information; they are not page types:
`glossary`, `example-gallery`, `api-catalogue`, `capability-catalogue`,
`compatibility-matrix`, `feature-index`, `knowledge-index`,
`related-projects`, `topic-map`.

Packages bundle products for one purpose; they list products and never embed
their content: `project-site`, `developer-guide`, `user-guide`,
`api-documentation`, `integration-guide`, `research-background`,
`ai-knowledge-package`, `offline-bundle`.

Common labels that look like types but are not: an FAQ is a question-oriented
presentation, troubleshooting is a how-to, a changelog is a release-history
document, a glossary is a reference collection.

## Rules that keep the taxonomy honest

1. **Omission over placeholders.** A kind with no records is omitted and
   reported, never rendered as an empty section.
2. **Composition over duplication.** Products compose document references;
   packages reference products; each fact appears once and is cross-referenced
   elsewhere.
3. **References, not copies.** The reuse and handoff families point at their
   authoritative sources (specodelic, git, turu/whisper, wai) and create no
   ledger entities. kaya is not a second knowledge store.
4. **Evidence before assertion.** Every generated fact traces to event ids;
   interpretation stays authored or human-approved.
5. **Justified emission.** A publication tree contains no stub page for a
   product with no backing information.

## Deferred

The same conversation went further than a document list: structured DocIR as
kaya's central artifact, doctests as first-class objects with runners and
expected output, documentation impact analysis as a commit gate
(change → impact set → required dispositions → validation → gate), and Org as
an authoring frontend with `KAYA_*` properties. Those change kaya's central
artifact rather than its document list, so they are parked as one design
ticket (kaya-aaz) until the taxonomy lands.

Also deferred, and load-bearing for the reuse families: the `dont` (kaya-dt2),
turu/whisper (kaya-syx), and specodelic (kaya-24c) adapters.