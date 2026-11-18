> *"Let me tell you about something*
> *Bigger than a big bamboo (Big bamboo)"*
> — Sumo, "Kaya"

# kaya

[![tracked with wai](https://img.shields.io/badge/tracked%20with-wai-blue)](https://github.com/charly-vibes/wai)
[![Release](https://github.com/charly-vibes/kaya/actions/workflows/release.yml/badge.svg)](https://github.com/charly-vibes/kaya/actions/workflows/release.yml)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

> A living record of exploration — kaya turns the artifacts a project already
> produces into continuously generated, human-owned documentation.
> **Status:** [experimental](docs/src/status.md) · MVP-1 CLI implemented

Part of the [charly-vibes](https://github.com/charly-vibes) toolshed.

The overview — what kaya is, why it exists, and when to use it — is a
[compiled page](docs/public/overview.md) emitted from this repository's own
record that `kaya` compiles and verifies. Motivation, architecture and design
decisions live in the [book](docs/src/index.md).

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
kaya check                         # verify projections without writing
kaya check --index                 # verify the staged tree a hook would commit
kaya acknowledge <document> --block <name> --events <id>... --actor <name>
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
  when its kinds have no records. The research map's trailing count reports
  record events; `verification.*` bookkeeping is excluded so replaying an
  acknowledgment changes no bytes.
- `kaya check` recomputes both the research and publication trees without
  writing and exits 0 clean, 1 violations, 2 usage or I/O. `--index` reads the
  staged tree instead of the worktree; `--root`, `--out`, and `--publish-out`
  set the invocation and projection roots. The envelope reports `events`,
  `ledger_digest`, `last_capture_commit`, `newest_event_at`,
  `events_by_source`, `emitted_kinds`, and `omitted_kinds`; kaya evaluates no
  thresholds. `--require-kind <family.kind>`, `--require-source
  <wai|git|test-runner|model>`, and `--fail-on-stale-authored` turn those
  findings into exit 1.
- Authored prose may cite ledger evidence with a
  `<!-- kaya:cites <event-id> -->` marker inside a `kaya:authored` block.
  Every cited id must exist in the ledger. When a correction supersedes a
  cited id, the block is reported stale; a human clears it with
  `kaya acknowledge <document> --block <name> --events <id>...`, which records
  one `verification.acknowledged` event and never edits authored bytes.
- Promotion is human-approved and pull-based: `promote` and `challenge`
  require `--actor`, and pending counts surface in `status`/`build` output
  instead of as prompts. Events are not conclusions.
- Every executed command prints exactly one JSON envelope to stdout; the
  envelope contract is described in [Status](docs/src/status.md).

### Record gates

- `just record-verify` runs the read-only worktree check.
- `just record-check [warn|fail]` composes the verifier, release claims, built
  book links, and the lag gate; pull requests warn, main and releases fail.
- `just record-lag [warn|fail] [bound]` reports commits since
  `last_capture_commit` touching `src`, `docs`, `openspec`, and `.wai`
  (default bound 25).
- `just links` builds the book and fails on unresolved relative links;
  `just release-claims` checks that `docs/src/status.md` agrees with
  `Cargo.toml`.
- `just record-capture` observes the wai tree and recent commits, then rebuilds
  the products; the nightly Record workflow commits the snapshot on main and
  skips its own `chore(record):` commits.

The behavioral specification remains the openspec change
[`add-core-research-record`](openspec/changes/add-core-research-record/proposal.md)
(event, ledger, model, projection, promotion, adapters). The document taxonomy
is archived and deployed as
[`kaya-documents`](openspec/specs/kaya-documents/spec.md) and
[`kaya-publications`](openspec/specs/kaya-publications/spec.md) — see
[Documentation taxonomy](docs/src/document-taxonomy.md).
Remaining milestones are tracked in the
[change tasks](openspec/changes/add-core-research-record/tasks.md) and in
beads (`bd ready` in a clone). This repository already dogfoods the compiler:
[docs/research/](docs/research/index.md) is kaya's own compiled dossier.

## Install

The CLI is installed from this repository with a stable Rust toolchain
(1.88 or newer):

```sh
cargo install --git https://github.com/charly-vibes/kaya kaya-vibes
```

Publishing to crates.io is wired but runs only after maintainer authorization;
see [RELEASING.md](RELEASING.md) for the token, tag and verification steps.
The crates.io package is `kaya-vibes` (the bare name `kaya` belongs to an
unrelated crate); the installed command is still `kaya`.

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
- [Releasing](RELEASING.md)
- Apache-2.0 — see [LICENSE](LICENSE)
