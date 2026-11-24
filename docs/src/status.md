# Status

**kaya** is **experimental**: exploratory; may be renamed or sunset.

The behavioral specification is deployed as capability specs under
[`openspec/specs/`](https://github.com/charly-vibes/kaya/tree/main/openspec/specs),
specified by the archived changes
[`add-core-research-record`](https://github.com/charly-vibes/kaya/blob/main/openspec/changes/archive/2026-11-18-add-core-research-record/proposal.md)
and
[`add-record-verification`](https://github.com/charly-vibes/kaya/blob/main/openspec/changes/archive/2026-11-18-add-record-verification/proposal.md)
— seven capabilities (event, ledger, model, projection, promotion, adapters, verification),
each with lint-clean specodelic layers and scenario-level red-test shapes.
MVP-1 implements those capabilities end to end as a static research compiler.

| Area | Status | Notes |
|---|---|---|
| Architecture | implemented (MVP-1) | Four subsystems: capture adapters (`src/adapters`), research ledger (`src/ledger.rs`), research model (`src/model.rs`), documentation projections (`src/projection`) |
| Event format | [implemented (draft)](https://github.com/charly-vibes/kaya/blob/main/openspec/specs/kaya-event/spec.md) | `kaya.event/v0` — closed 9-field schema, append-only JSONL, provenance-preserving, observation-only |
| Research vocabulary | [implemented](https://github.com/charly-vibes/kaya/blob/main/openspec/specs/kaya-model/spec.md) | 13 entities, 9 typed direction-fixed relationships; relationships over entity count |
| CLI | implemented | `kaya init`, `kaya status`, `kaya import` (wai, git, test, turu, model), `kaya build [--out] [--expect-digest] [--allow-uncommitted]`, `kaya ir`, `kaya publish [--out]`, `kaya check [--index --require-kind --require-source --fail-on-stale-authored]`, `kaya acknowledge`, `kaya promote`, `kaya challenge`, `kaya doctest`, `kaya supersede-doctest`, `kaya impact [--staged --range]`, `kaya disposition` — `build` and `publish` both compose the publication tree, including the `glossary` collection page; the other eight collections still refuse at declaration |
| Integrations | wai, git, test runner, whisper (turu) implemented; dont, specodelic [specified](https://github.com/charly-vibes/kaya/blob/main/openspec/specs/kaya-adapters/spec.md) (contracts under `.espectacular/kaya-adapters/`), not implemented | Adapters translate tool outputs into `kaya.event/v0`; the remaining sources have encoded specs but no adapter |
| Document taxonomy | implemented and deployed (eleven record kinds, publications) | [Record layer: four families, eleven kinds; publication layer: twelve products, collections, packages](specs/kaya-documents.md) ([publications](specs/kaya-publications.md)). Implemented: `research-map`, `project-overview`, `experiment-specification`, `experiment-record`, `findings-and-analysis`, `decision-record`, `technical-reference`, `reproduction-package`, `knowledge-brief`, `handoff`; `kaya publish` declares all twelve vocabulary products (`overview`, `reference`, `getting-started`, `tutorial`, `how-to`, `concept`, `example`, `troubleshooting`, `faq`, `status-compatibility`, `release-migration`, `architecture-design`; `tutorial` publishes as the `no-backing-document` omission until exploration records reach the publication reader), emits the backed ones and both the `project-site` and `developer-guide` packages, and reports unbacked products as omissions instead of stubbing them. `exploration-brief` remains specified. See [Documentation taxonomy](document-taxonomy.md) |
| Docs | buildable, deploy workflow present | `book.toml` + `theme/charly.css` at root; `.github/workflows/docs.yml` builds the book and deploys it to GitHub Pages on push to `main` |
| CI | implemented | `just ci` (cargo test, fmt + clippy, mdbook build, `ah check`, `spk lint`, `openspec validate --strict`) run by `.github/workflows/ci.yml`; lefthook gates `ah check`, `spk lint`, and `pretender` on commit |
| Release | v0.1.0 and v0.2.0 released | `.github/workflows/release.yml` builds the five standard targets on a `v*` tag and publishes the canonical archives, `checksums.txt`, and `provenance.json`. GitHub Releases v0.1.0 and v0.2.0 and crates.io `kaya-vibes` 0.2.0 are published; homebrew and scoop remain token-gated. Install: `cargo install --git https://github.com/charly-vibes/kaya kaya-vibes` |

## Implemented

- `kaya init` — creates `.kaya/ledger.jsonl`, refusing a path that is not a
  regular file.
- `kaya status` — reports event count, byte size, digest, and pending
  proposals; a missing ledger fails as `missing-ledger`.
- `kaya import wai|git|test|turu|model` — captures observations from the `.wai/`
  artifact tree, recent repository commits, one libtest JSONL run, and a
  JSONL file of research-structure events (validated `model.*` records only).
- `kaya build [--out DIR] [--expect-digest HEX] [--allow-uncommitted]` —
  compiles `docs/research/index.md`, one
  `docs/research/experiments/<key>.md` per experiment, the project-level
  `docs/research/overview.md`, `docs/research/brief.md`, and
  `docs/research/understanding.md`, and the
  reuse-handoff reference documents; only `kaya:`-owned blocks are rewritten,
  a document whose kinds have no records stays absent, the ledger is verified
  against its committed history, and `--expect-digest` refuses to build from
  bytes other than the recorded ones.
- `kaya promote <assertion> --actor <name>` and
  `kaya challenge <conclusion> --events <ids...> --actor <name>` — the
  human-only door into the record: proposals become conclusions, conclusions
  gain counter-evidence, and neither happens automatically.
- Document-IR tooling: `kaya ir` prints the record's document IR;
  `kaya publish` composes the client-facing publication tree;
  `kaya doctest` / `kaya supersede-doctest` run and disposition declared
  doctest contracts; `kaya impact` / `kaya disposition` gate a change set over
  the document IR into reviewed/waived record obligations.
- Library modules: event parsing, append-only ledger with digest anchoring,
  crash-truncated tail recovery and supersede resolution, the research model,
  deterministic projections with authored-block preservation and stale
  detection, and promotion state replay.
- Every executed command prints exactly one JSON envelope to stdout, success
  or failure; `--help` and `--version` print clap's own text.

## Test report input

`kaya import test --report <path>` reads a libtest JSONL report. The stable
Rust toolchain cannot produce one: libtest's `--format json` is nightly-only,
and stable libtest rejects `-Z unstable-options` outright. The repository
therefore converts the JUnit XML that nextest does emit, which needs no
nightly:

```toml
# .config/nextest.toml
[profile.ci.junit]
path = "junit.xml"
```

```sh
cargo nextest run --profile ci            # writes target/nextest/ci/junit.xml
python3 scripts/junit-to-libtest.py \
  target/nextest/ci/junit.xml cargo-test-results.jsonl
kaya import test --report cargo-test-results.jsonl
```

The converter names each test and the outcome its element states — a
`<failure>` or `<error>` fails, a `<skipped>` is ignored, anything else
passed — and refuses a report it cannot read without writing anything. The
adapter is unchanged: the converted file is the libtest JSONL it always
consumed.

The alternative, documenting the nightly invocation
(`cargo +nightly test -Z unstable-options --format json`), was rejected: it
requires a nightly toolchain and an upstream format that is still unstable.

## In progress

The `dont` and specodelic adapters are specified but not written (deferred,
tracked as kaya-dt2 and kaya-24c); the turu adapter shipped. The MVP-2 seams —
continuous capture, agent traces, deduplication, missing-evidence detection —
are named but not yet specified; they start as
openspec proposals when picked up. Incremental regeneration is implemented on
the build side: a document whose ledger digest, generator version, and on-disk
bytes all match the last build (recorded in `.kaya/build-state.json`) is
skipped instead of re-rendered, and `kaya build --instrument` reports the
render-invocation count.

## Dogfooding

- kaya's own dossier is compiled by kaya: the ledger at
  `.kaya/ledger.jsonl` and the projections under `docs/research/`
  (`index.md`, `overview.md`, `brief.md`,
  `experiments/add-core-research-record.md`) are
  checked in and rebuilt by `kaya build`.
- Reverse dogfooding: kaya's own specs are written in specodelic format and
  gate-checked by `spk lint` on every commit, and the change validates under
  `openspec validate --strict` — kaya is specified with the tools it reads.

## Notes

- Install: `cargo install --git https://github.com/charly-vibes/kaya kaya-vibes`.
  crates.io `kaya-vibes` 0.2.0 is published; the binary remains `kaya`.