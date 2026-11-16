> *"Kaya now, got to have kaya now"*
> — Bob Marley, "Kaya" (1978)

# kaya

[![tracked with wai](https://img.shields.io/badge/tracked%20with-wai-blue)](https://github.com/charly-vibes/wai)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

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

Kaya compiles what your tools already record into documentation: adapters
translate tool outputs into a small event format (`kaya.event/v0`), an
append-only ledger preserves them, a research model connects them, and
deterministic projections render them — with generated and human-authored
sections kept strictly apart:

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

The full motivation, architecture, design decisions and milestones live in
[Motivation & design](docs/src/index.md).

## Status

Design phase — the architecture and the six core capabilities are specified
in [`openspec/changes/add-core-research-record/`](openspec/changes/add-core-research-record/proposal.md)
(event, ledger, model, projection, promotion, adapters); the CLI is not
implemented yet. The roadmap: the
[change tasks](openspec/changes/add-core-research-record/tasks.md) plus the
beads tickets (`bd ready` in a clone), starting with MVP 1 — a static
research compiler that reconstructs a useful dossier from existing
wai/git/test artifacts.

## FAQ

### Why not extend wai instead?
wai owns the workflow (what to do next); kaya owns the record (what happened
and what it meant). They observe each other's outputs instead of merging —
see [Sources and authority](docs/src/index.md#sources-and-authority).

### Is kaya.event/v0 stable?
No — it is a draft schema. It grows only by explicit revision, and events
carry a schema field so old ledgers stay readable.

### What if my AI provider disappears?
Nothing breaks. Interpretation is human-owned; AI only writes visibly
flagged proposals; the ledger and projections are plain JSONL and Markdown
in git.

### How do I get my data out?
It is already out. The ledger is append-only JSONL, the docs are Markdown —
both readable without kaya. There is no export step because there is no
lock-in.

### What happens when evidence changes under an interpretation?
The authored section is flagged stale (naming the changed evidence) and its
bytes are never touched — acknowledgment is a human act.

## Development

```sh
just ci        # gates: ah check, pretender, spk lint (via lefthook)
ddl status     # fleet tooling health
ddl doctor     # detailed diagnostics
mdbook build   # docs (book.toml at root)
```

No install section yet — there is no binary; the standard install block
lands with the first release.

## Docs / status / license

- [Motivation & design](docs/src/index.md)
- [Status](docs/src/status.md)
- Apache-2.0 — see [LICENSE](LICENSE)
