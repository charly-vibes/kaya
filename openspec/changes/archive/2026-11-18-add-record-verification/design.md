# Design: add-record-verification

## Context

Three contracts already exist and must not be duplicated: the ledger's
committed-history verification (`build`), the managed-block grammar
(`kaya.projection`), and the stale report. `check` is a second reader of the
same three, with one difference that drives the whole design: it must not
write.

Measured cost on the committed record: `kaya build` 11 ms, `kaya publish`
3 ms. A read-only pass is bounded by the same work, so the command is viable in
a pre-commit hook without a cache.

## Architecture

```
                 ┌──────────────┐
   worktree ────▶│  ledger read │──▶ events, digest, facts
   index (:path) │              │
                 └──────┬───────┘
                        ▼
                 ┌──────────────┐
                 │ model + proj │──▶ expected bytes (in memory)
                 └──────┬───────┘
                        ▼
   worktree/index ─────▶│ compare │──▶ drift[], stale[], missing[], facts
                        └──────────┘
```

`build` and `check` share the "expected bytes" stage; only `build` proceeds to
write. Sharing is mandatory: a second renderer would drift from the compiler.

## Index semantics

`--index` resolves every relevant path through `git show :<path>`:

- `.kaya/ledger.jsonl` — the staged ledger, not necessarily the worktree one.
- `docs/research/**` and `docs/public/**` — the staged documents.

Committed-history verification uses the same rule as `build` (HEAD bytes must
prefix the candidate bytes), with one index-only exception: when HEAD carries
no ledger, the staged ledger is accepted as empty history, because the first
commit has nothing to extend.

Paths that are not staged and do not exist in the index are treated as absent,
which is how a staged deletion of a generated document is detected. A path that
exists in the index but cannot be read (`git show :<path>` fails, or the entry
is unmerged) is an I/O failure: exit 2, `kaya.verification.check_io`, naming the
path. There is never a silent fallback to the worktree.

## Finding model

| area | meaning | violation kind |
|---|---|---|
| `drift` | a generated block's bytes differ from the projection | `changed` |
| `missing` | the projection requires a document the index/worktree lacks | `missing` |
| `extra` | a managed document exists that the projection would not emit | `extra` |
| `stale` | an authored block cites evidence whose effective data changed | `stale` |
| `required` | a required kind or source is absent | `required` |

Exit 1 when any of these is present *and* the corresponding flag/condition
applies; plain `check` exits 1 only for `drift`, `missing`, and `extra`, which
are always defects.

## Facts

`last_capture_commit` is the newest `data.sha` among `capture.commit.observed`
events; `null` when the record has none, and reported as recorded when the
commit is no longer reachable after a rebase. `newest_event_at` is the maximum
timestamp.

Sources are canonical, not actor-derived:

| source | derived from |
|---|---|
| `wai` | `capture.artifact.observed` emitted by the wai adapter |
| `git` | `capture.commit.observed` |
| `test-runner` | `experiment.run.completed` |
| `model` | `model.*` events (seed imports) |

The actor name cannot be the key: the test adapter's actor is `test-runner`,
and seed events carry a human actor such as `charly` with `provenance.tool: kaya`.

`emitted_kinds` and `omitted_kinds` are dotted `family.kind` names. The build
summary splits omitted entries into `family` + `kind`; `check` joins them so
both lists share one spelling and `--require-kind` has a single grammar.

Kaya evaluates no threshold. A repository that wants "at most 25 commits since
capture" computes it from the fact plus its own git call.

## Envelope

```json
{
  "ok": true,
  "envelope_version": "0.1",
  "envelope_kind": "check",
  "data": {
    "drift": [{ "path": "docs/research/index.md", "block": "orientation.research-map", "kind": "changed" }],
    "stale": [],
    "events": 80,
    "ledger_digest": "005f99…",
    "last_capture_commit": "e5cf768…",
    "newest_event_at": "2026-11-16T18:53:24Z",
    "events_by_source": { "wai": 47, "git": 20, "test-runner": 0, "model": 13 },
    "emitted_kinds": ["orientation.project-overview", "orientation.research-map"],
    "omitted_kinds": ["understanding.decision-record"]
  }
}
```


## Alternatives rejected

- **`build --check`** — keeps one command but couples "verify HEAD" to
  "compare staged bytes", and keeps a writer one flag away from a hook.
- **`git diff` after `build`** — mutates the tree, cannot run in pre-commit,
  and reports file-level noise instead of block-level findings.
- **A repository script that parses the ledger** — a second parser of the
  event schema, which the project's own adapter rule forbids.

## Risks

- **Index reads and git worktree plumbing.** Name the failure
  (`kaya.verification.check_io`) when `git show :<path>` fails or the entry is
  unmerged; never fall back to the worktree.
- **Byte comparison across platforms.** CRLF checkouts would make every block
  look drifted; the repository pins `* text=auto eol=lf` for docs and ledger
  before the gate lands (kaya-3kg).
- **Two readers drifting.** The expected-bytes stage must be extracted, not
  copied; a test builds once and checks once over the same fixture.
- **Hook/CI parity.** Both must invoke `just record-check`; a divergent CI
  command makes the local gate theatre.