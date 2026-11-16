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
              │  (adapters capture what already exists)
              ▼
        research ledger         append-only JSONL, kaya.event/v0,
              │                 provenance + stable ids
              ▼
        research model          Project · Question · Idea · Hypothesis ·
              │                 Model · Experiment · Run · Artifact ·
              │                 Observation · Claim · Finding · Decision · Source
              ▼
   documentation projections    research index, experiment reports,
                                evidence dossiers, reproduction guides
```

Ledger, model and projections are deliberately separate: the ledger preserves
what happened, the model says what it means, the projections present that
meaning to different audiences.

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
Documents mix three ownership modes, never mixed inside one section:

- **derived** — generated inside `kaya:begin/end` managed blocks;
- **authored** — human interpretation inside `kaya:authored` blocks;
- **proposed** — AI drafts awaiting approval, always visibly flagged.

## Design decisions

1. **Storage:** git is the canonical archive (Markdown + JSONL); any database
   is at most a rebuildable query index.
2. **Capture:** hybrid — passive adapters plus lightweight explicit capture
   (`kaya note/question/decision/finding` — seconds, not forms).
3. **Narrative authority:** human-owned interpretation, generated factual
   sections, proposed drafts kept distinct.
4. **Scope:** workspace-first; experiment identities are namespace-qualified
   from day one so multiple repositories can connect later.

## Deliberately not built

Another workflow manager (wai exists), a general graph database (query
patterns unknown), automatic paper generation (unsupported narrative), a
universal event bus (file-based integration suffices), automatic conclusions,
a proprietary documentation format, a complex web application (the viewer
comes last, if ever).

## Milestones

1. **MVP 1 — static research compilation.** Core entities, event format, wai
   import, git history + test results import, research index, experiment
   report with authored sections preserved. *Validation: a useful dossier from
   existing project information, without rewriting anything.*
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

