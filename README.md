> *"Let me tell you about something*
> *Bigger than a big bamboo (Big bamboo)"*
> — Sumo, "Kaya"

# kaya

[![tracked with wai](https://img.shields.io/badge/tracked%20with-wai-blue)](https://github.com/charly-vibes/wai)
[![Release](https://github.com/charly-vibes/kaya/actions/workflows/release.yml/badge.svg)](https://github.com/charly-vibes/kaya/actions/workflows/release.yml)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

> A living record of exploration — kaya turns the artifacts a project already
> produces into continuously generated, human-owned documentation.
> **Why:** documentation drifts because it is written separately from the work
> that makes it true. Kaya captures events, decisions, evidence and artifacts
> from the tools used during development (wai, dont, whisper, specodelic, git,
> test runners) and compiles them into documentation projections — so the
> record grows out of the work instead of stopping it.
> **Status:** [experimental](docs/src/status.md) · MVP-1 CLI implemented · [Motivation & design](docs/src/index.md)

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

MVP-1, the static research compiler, is implemented: kaya imports observations
from artifacts a repository already has, appends them to an append-only
ledger, and recompiles a research dossier from that ledger.

```sh
kaya init                          # create .kaya/ledger.jsonl if absent
kaya status                        # event count, byte size, digest, pending proposals
kaya import wai                    # observe the .wai/ artifact tree
kaya import git                    # observe recent commits
kaya import test --report <path>   # observe one libtest JSONL run
kaya import model --file <seed>    # record research-structure events from JSONL
kaya build                         # compile docs/research (index + experiment reports)
kaya build --expect-digest <hex>   # refuse to build unless the ledger matches
kaya promote <assertion> --actor <name>
kaya challenge <conclusion> --events <id>... --actor <name>
```

- The ledger lives at `.kaya/ledger.jsonl`: one `kaya.event/v0` record per
  line, append-only, plain JSONL that survives in git.
- `kaya build` compiles `docs/research/index.md`, one
  `docs/research/experiments/<key>.md` per experiment, the project-level
  `docs/research/overview.md` and `docs/research/understanding.md`, and the
  reuse-handoff reference documents; it rewrites only its own generated
  blocks, preserves authored bytes verbatim, and leaves a document absent
  when its kinds have no records.
- Promotion is human-approved and pull-based: `promote` and `challenge`
  require `--actor`, and pending counts surface in `status`/`build` output
  instead of as prompts. Events are not conclusions.
- Every executed command prints exactly one JSON envelope to stdout, success
  or failure; `--help` and `--version` print clap's own text.

The behavioral specification remains the openspec change
[`add-core-research-record`](openspec/changes/add-core-research-record/proposal.md)
(event, ledger, model, projection, promotion, adapters), plus
[`add-document-taxonomy`](openspec/changes/add-document-taxonomy/proposal.md)
(the record-facing document kinds and the client-facing publications composed
from them — see [Documentation taxonomy](docs/src/document-taxonomy.md)).
Remaining milestones are tracked in the
[change tasks](openspec/changes/add-core-research-record/tasks.md) and in
beads (`bd ready` in a clone). This repository already dogfoods the compiler:
[docs/research/](docs/research/index.md) is kaya's own compiled dossier.

## Install

The CLI is installed from this repository with a stable Rust toolchain
(1.88 or newer):

```sh
cargo install --git https://github.com/charly-vibes/kaya kaya
```

Publication to crates.io is deferred; install from git until then.

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
just ci        # cargo test, fmt + clippy, mdbook build, ah check, spk lint, openspec validate
just build     # cargo build
just test      # cargo test
mdbook build   # docs only (book.toml at root)
ddl status     # fleet tooling health
ddl doctor     # detailed diagnostics
```

`just ci` is the single source of truth: `.github/workflows/ci.yml` runs that
exact command, and lefthook gates the same spec checks on commit and push.

## Docs / status / license

- [Motivation & design](docs/src/index.md)
- [Status](docs/src/status.md)
- Apache-2.0 — see [LICENSE](LICENSE)
