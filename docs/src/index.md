> *"Let me tell you about something*
> *Bigger than a big bamboo (Big bamboo)"*
> — Sumo, "Kaya"

# kaya — motivation & design

> Documentation that grows out of the work, rather than work that stops to
> produce documentation.

## Why

Software is a medium for thinking, but the thinking is usually lost: decisions
dissolve into chat logs, experiments leave no trace, READMEs drift from reality
the week they are written. The charly-vibes ecosystem already records most of
the raw material — wai keeps research and decisions, dont keeps grounded
claims, whisper keeps knowledge, specodelic keeps formal specs — but nobody
connects those records into a coherent, inspectable research narrative.

Kaya is the integrator: a common research-information model plus a
documentation compiler over it. It does not duplicate what those tools own.

## Architecture

```
  wai · dont · whisper · specodelic · git · test runners
              │  [capture — adapters translate, never interpret]
              ▼
        research ledger         append-only JSONL, kaya.event/v0,
              │                 provenance + stable ids
              ▼  [preserve — immutable evidence]
        research model          Project · Question · Idea · Hypothesis ·
              │                 Model · Experiment · Run · Artifact ·
              │                 Observation · Claim · Finding · Decision · Source
              ▼  [understand — typed, direction-fixed relationships]
   document projections        record layer: four families, eleven kinds
              │                 (orientation · exploration · understanding · reuse-handoff)
              ▼  [compose]
   publications                client layer: twelve reader-facing products,
              ▼                 collections and packages
   [reveal — deterministic generation, authored sections preserved]
```

Ledger, model and projections are deliberately separate: the ledger preserves
what happened, the model says what it means, and the projections present that
meaning to different audiences. The record layer answers what we intended,
what happened, what we learned and what we chose; the publication layer
answers a visitor's questions. See [Documentation taxonomy](document-taxonomy.md)
for the kinds, the products, their mapping, and what is deferred.

## Sources and authority

| Information | Authoritative source | Kaya's role |
|---|---|---|
| Research rationale, decisions | wai | organize, present, cross-link |
| Claims and evidence | dont | expose support and doubt |
| Knowledge and context | whisper (turu) | connect and contextualize |
| Formal constraints, specs | specodelic | explain and verify |
| Implementation history | git | connect versions to experiments |
| Interpretation | human (AI-assisted) | preserve and publish |

Kaya is a consumer of knowledge, never a second place where the same knowledge
must be entered. There are no competing copies of authoritative information.

## Events, assertions, conclusions

| Category | Example | Producer |
|---|---|---|
| Event | "test suite completed: 18 passed, 2 failed" | tool |
| Assertion | "this result contradicts H-001" | human or AI proposal |
| Conclusion | "H-001 is partially supported" | human-approved |

An AI may suggest interpretation; it may never silently promote a proposal
into an established finding. This is the defining design principle.

## Documentation compilation

Generation is a compiler: normalize → validate → link semantics → project.
Deterministic, incremental, with source maps back to original evidence.
Documents mix two block forms, never mixed inside one block:

- **generated** — compiler-owned, inside `kaya:begin/end generated:<name>`
  markers;
- **authored** — human-owned, inside `kaya:authored` markers.

Proposals are not a block form but a record status: AI-drafted
interpretations land as `proposed` assertions, always visibly flagged,
until a human approves them (see [Events, assertions, conclusions](#events-assertions-conclusions)).

## Design decisions

1. **Storage:** git is the canonical archive (Markdown + JSONL); any database
   is at most a rebuildable query index.
2. **Capture:** hybrid — passive adapters plus lightweight explicit capture
   (`kaya note/question/decision/finding` — seconds, not forms).
3. **Narrative authority:** human-owned interpretation, generated factual
   sections, proposed drafts kept distinct.
4. **Scope:** workspace-first; experiment identities are namespace-qualified
   from day one so multiple repositories can connect later (connect-later
   semantics — merge and conflict resolution — are deliberately unspecified
   until a second workspace actually exists).

## Your data outlives kaya

The ledger is append-only JSONL; the projections are Markdown; both live in
git as plain files. Every artifact kaya produces is readable — by a human or
another tool — without kaya running. There is no export step because there
is no lock-in; if kaya is sunset, the record survives it.

## Deliberately not built

Another workflow manager (wai exists), a general graph database (query
patterns unknown), automatic paper generation (unsupported narrative), a
universal event bus (file-based integration suffices), automatic conclusions,
a proprietary documentation format, a complex web application (the viewer
comes last, if ever).

## Milestones

1. **MVP 1 — static research compilation.** Core entities and event format
   *(implemented — [add-core-research-record](../../openspec/changes/add-core-research-record/proposal.md))*,
   wai import, git history + test results import, research index, experiment
   report with authored sections preserved. *Validation: a useful dossier from
   existing project information, without rewriting anything — proven by the
   dogfooded [research dossier](../research/index.md).*
2. **MVP 2 — continuous capture.** CLI capture, agent traces, deduplication,
   relationship linking, missing-evidence detection, incremental
   regeneration.
3. **MVP 3 — interactive research environment.** Graph exploration, run
   comparison, dossier export. *Validation: a stranger can continue an
   exploration without its author.*

## Provenance

The design originates in a research conversation on event-driven research
documentation (chatgpt, November 2026 — "Kaya" naming and the four-source
architecture: wai, dont, whisper, specodelic). Key prior ideas in the org:
wai preserves the *why*; dont forces epistemic discipline; specodelic makes
specs machine-checkable. Kaya is the piece that makes the whole development
process inspectable as an evolving body of knowledge.
