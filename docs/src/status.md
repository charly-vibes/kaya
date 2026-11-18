# Status

**kaya** is **experimental**: exploratory; may be renamed or sunset.

The behavioral specification is the openspec change
[`add-core-research-record`](../../openspec/changes/add-core-research-record/proposal.md)
— six capabilities (event, ledger, model, projection, promotion, adapters),
each with lint-clean specodelic layers and scenario-level red-test shapes.
MVP-1 implements those capabilities end to end as a static research compiler.

| Area | Status | Notes |
|---|---|---|
| Architecture | implemented (MVP-1) | Four subsystems: capture adapters (`src/adapters`), research ledger (`src/ledger.rs`), research model (`src/model.rs`), documentation projections (`src/projection`) |
| Event format | [implemented (draft)](../../openspec/changes/add-core-research-record/specs/kaya-event/spec.md) | `kaya.event/v0` — closed 9-field schema, append-only JSONL, provenance-preserving, observation-only |
| Research vocabulary | [implemented](../../openspec/changes/add-core-research-record/specs/kaya-model/spec.md) | 13 entities, 9 typed direction-fixed relationships; relationships over entity count |
| CLI | implemented | `kaya init`, `kaya status`, `kaya import` (wai, git, test), `kaya build [--out] [--expect-digest]`, `kaya promote`, `kaya challenge` |
| Integrations | wai, git, test runner implemented; dont, whisper (turu), specodelic [specified](../../openspec/changes/add-core-research-record/specs/kaya-adapters/spec.md), not implemented | Adapters translate tool outputs into `kaya.event/v0`; the remaining sources have encoded specs but no adapter |
| Document taxonomy | implemented and deployed (eleven record kinds, publications) | [Record layer: four families, eleven kinds; publication layer: twelve products, collections, packages](../../openspec/specs/kaya-documents/spec.md) ([publications](../../openspec/specs/kaya-publications/spec.md)). Implemented: `research-map`, `project-overview`, `experiment-specification`, `experiment-record`, `findings-and-analysis`, `decision-record`, `technical-reference`, `reproduction-package`, `knowledge-brief`, `handoff`; `kaya publish` declares all eleven non-tutorial products (`overview`, `reference`, `getting-started`, `how-to`, `concept`, `example`, `troubleshooting`, `faq`, `status-compatibility`, `release-migration`, `architecture-design`), emits the backed ones and both the `project-site` and `developer-guide` packages, and reports unbacked products as omissions instead of stubbing them. `exploration-brief`, `technical-reference`, `knowledge-brief` remain specified. See [Documentation taxonomy](document-taxonomy.md) |
| Docs | buildable, deploy workflow present | `book.toml` + `theme/charly.css` at root; `.github/workflows/docs.yml` builds the book and deploys it to GitHub Pages on push to `main` |
| CI | implemented | `just ci` (cargo test, fmt + clippy, mdbook build, `ah check`, `spk lint`, `openspec validate --strict`) run by `.github/workflows/ci.yml`; lefthook gates `ah check`, `spk lint`, and `pretender` on commit |
| Release | workflow implemented, no release tagged | `.github/workflows/release.yml` builds the five standard targets on a `v*` tag, publishes the canonical archives plus `checksums.txt` and `provenance.json`, and updates the homebrew-charly tap and scoop-charly bucket. Deviation (§1): the crates.io channel is token-gated and currently disabled — publishing requires explicit authorization (`kaya-dgz`), and no GitHub Release exists yet, so prebuilt binaries are not the install path until the first tag. Until then the only install path is building from source |

## Implemented

- `kaya init` — creates `.kaya/ledger.jsonl`, refusing a path that is not a
  regular file.
- `kaya status` — reports event count, byte size, digest, and pending
  proposals; a missing ledger fails as `missing-ledger`.
- `kaya import wai|git|test|model` — captures observations from the `.wai/`
  artifact tree, recent repository commits, one libtest JSONL run, and a
  JSONL file of research-structure events (validated `model.*` records only).
- `kaya build [--out DIR] [--expect-digest HEX] [--allow-uncommitted]` —
  compiles `docs/research/index.md`, one
  `docs/research/experiments/<key>.md` per experiment, the project-level
  `docs/research/overview.md` and `docs/research/understanding.md`, and the
  reuse-handoff reference documents; only `kaya:`-owned blocks are rewritten,
  a document whose kinds have no records stays absent, the ledger is verified
  against its committed history, and `--expect-digest` refuses to build from
  bytes other than the recorded ones.
- `kaya promote <assertion> --actor <name>` and
  `kaya challenge <conclusion> --events <ids...> --actor <name>` — the
  human-only door into the record: proposals become conclusions, conclusions
  gain counter-evidence, and neither happens automatically.
- Library modules: event parsing, append-only ledger with digest anchoring,
  crash-truncated tail recovery and supersede resolution, the research model,
  deterministic projections with authored-block preservation and stale
  detection, and promotion state replay.
- Every executed command prints exactly one JSON envelope to stdout, success
  or failure; `--help` and `--version` print clap's own text.

## In progress

MVP-2 seams — continuous capture, agent traces, deduplication, relationship
linking, missing-evidence detection, and incremental regeneration. The
`dont`, whisper (turu), and specodelic adapters are specified but not
written.

## Dogfooding

- kaya's own dossier is compiled by kaya: the ledger at
  `.kaya/ledger.jsonl` and the projections under `docs/research/`
  (`index.md`, `overview.md`, `experiments/add-core-research-record.md`) are
  checked in and rebuilt by `kaya build`.
- Reverse dogfooding: kaya's own specs are written in specodelic format and
  gate-checked by `spk lint` on every commit, and the change validates under
  `openspec validate --strict` — kaya is specified with the tools it reads.

## Notes

- Install: `cargo install --git https://github.com/charly-vibes/kaya kaya-vibes`.
  crates.io publication is deferred.