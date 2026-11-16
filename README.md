> *"Got to have kaya now, for the rain is falling"*
> — Bob Marley, "Kaya" (1978)

# kaya

> A living record of exploration — kaya turns the artifacts a project already
> produces into continuously generated, human-owned documentation.
> **Why:** documentation drifts because it is written separately from the work
> that makes it true. Kaya captures events, decisions, evidence and artifacts
> from the tools used during development (wai, dont, whisper, specodelic, git,
> test runners) and compiles them into documentation projections — so the
> record grows out of the work instead of stopping it.
> **Status:** [experimental](docs/src/status.md) · design phase, pre-MVP · [Motivation & design](docs/src/index.md)

Part of the [charly-vibes](https://github.com/charly-vibes) toolshed.

## When to use

- Use kaya when a repo's documentation must stay grounded in what actually
  happened: decisions taken, experiments run, evidence collected.
- Use kaya when onboarding (human or agent) should reconstruct *why* software
  is the way it is, not just *what* it is.

**Don't use it when** you need workflow management (`wai`), claim grounding
(`dont`), a knowledge workspace (`whisper`/turu), or formal spec verification
(`specodelic`) — kaya is a consumer and integrator of those tools' outputs,
not a replacement for any of them. It assumes the tools it reads already own
their own authoritative records.

## The idea

Kaya is an event-driven documentation system with four subsystems:

1. **Capture** — adapters that translate existing tool outputs into a small
   normalized event format (`kaya.event/v0`). Tools emit; adapters translate;
   nobody re-enters information.
2. **Preserve** — an append-only JSONL research ledger: immutable evidence,
   stable ids, provenance, corrections.
3. **Understand** — a research model connecting the vocabulary that matters:
   Project, Question, Idea, Hypothesis, Model, Experiment, Run, Artifact,
   Observation, Claim, Finding, Decision, Source.
4. **Reveal** — documentation projections (research index, experiment reports,
   evidence dossiers) generated deterministically from the ledger, with
   generated and authored sections kept apart:

```markdown
<!-- kaya:begin generated:run-summary -->
18 tests passed, 2 failed. See RUN-003.
<!-- kaya:end generated:run-summary -->

<!-- kaya:authored -->
The failures show structural checks cannot detect this
category of semantic conflict.
<!-- kaya:end authored -->
```

One design principle rules everything: **events are not conclusions**. Tools
record that two tests failed; only a human (or an approved proposal) may
conclude that a hypothesis was falsified. Generated sections never overwrite
authored interpretation.

For the full motivation, architecture, milestones and design decisions, read
[Motivation & design](docs/src/index.md).

## Status

Design phase — the architecture and event format are specified, the CLI is
not implemented yet. The roadmap lives in [beads](https://github.com/charly-vibes/kaya)
(`bd ready` in a clone) and starts with MVP 1: a static research compiler that
reconstructs a useful dossier from existing wai/git/test artifacts.

## Development

```sh
just ci        # gates: ah check, pretender, spk lint (via lefthook)
ddl status     # fleet tooling health
ddl doctor     # detailed diagnostics
```

## Docs / status / license

- [Motivation & design](docs/src/index.md)
- [Status](docs/src/status.md)
- Apache-2.0 — see [LICENSE](LICENSE)
