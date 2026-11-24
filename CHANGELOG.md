# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2026-11-24

### Added
- Collections tier end to end: `kaya publish` declares and renders all five
  implemented collections — `glossary`, `topic-map` (per-topic rows from the
  research map, cross-referencing the page that embeds it), `knowledge-index`
  (per-type census of the named node ids), `example-gallery` (rows from
  experiment records), and `api-catalogue` (rows from technical references).
  A collection whose producer rows are empty omits with
  `no-backing-document` instead of writing a page of empty sections; the
  export mirrors pages with `emitted` plus the omission reason; declaring any
  other collection kind refuses naming the implemented set
  (`src/publication/collection.rs`).
- Read-list parity: `technical-reference.md` joins both fixed document lists
  — publish's read order (`src/publication/source.rs`) and check's
  `RESEARCH_READ_ORDER` (`src/verification/mod.rs`) — in the same change.

### Changed
- Capture refusals carry per-field labels: `kaya.capture.invalid_key` and
  `kaya.capture.invalid_project` replace the overloaded
  `kaya.capture.bound_exceeded`, and `--project` now validates against the
  same `[a-z0-9-]` slug alphabet as `--key`, so a format violation is
  distinguishable from an over-bound text payload and a malformed project can
  no longer reach the subject id (`src/capture.rs`).
- A corrected capture's envelope names the outcome chain: `data.supersedes`
  carries the id of the superseded event alongside `data.event`, matching the
  superseded field the stored correction already records (`src/capture.rs`).

### Fixed
- A capture racing a twin that mints the same generation-salted correction id
  now re-resolves against the stored ledger once: matching text settles as a
  plain duplicate, and one bounded retry supersedes the twin's freshly stored
  correction under its bumped generation instead of failing outright
  (`src/capture.rs`).
- Incremental-build cache invalidation is mechanical: `build.rs` hashes the
  `src/projection` sources into the generator string
  (`<version>+render-<fnv1a>`), so a render-affecting edit invalidates stale
  state without a manual bump to remember; the const documents the scheme.
- The documents fingerprint distinguishes an unreadable file
  (`<unreadable>`) from an absent one (`<absent>`): both still only ever
  force a rebuild, but a permission problem no longer masquerades as a
  deleted document (`src/projection/state.rs`).
- The incremental-build temp file is pid-unique
  (`build-state.json.<pid>.tmp`), so two concurrent builds can no longer
  share one temp path and rename a torn file into place
  (`src/projection/state.rs`).
- Read-list parity is enforced by a unit test binding
  `publication::source::READER_FILES` to `verification`'s
  `RESEARCH_READ_ORDER`, so the two fixed orders can no longer drift — the
  failure class behind the brief.md publication-before-check gap.

## [0.3.0] - 2026-11-24

### Added
- Capture commands: `kaya note`, `kaya question`, `kaya decision`, and
  `kaya finding` append `model.entity.created` events with Observation,
  Question, Decision, and Finding subjects. One capture appends exactly one
  ledger line; provenance records `kaya` plus the CLI version and the
  `<subcommand>/<key>` reference; captured text runs the inherited
  secret-redaction pattern list (`src/capture.rs`).
- Incremental build regeneration: `kaya build` skips documents whose ledger
  digest, generator version, and on-disk bytes match the last successful
  build (state in `.kaya/build-state.json`, git-ignored), restoring their
  summary contribution instead of re-rendering; citation validation and
  staleness flagging still run on every build. `kaya build --instrument` adds
  `data.render_invocations` counting render constructor invocations.
- Glossary collection tier end to end: `kaya build` and `kaya publish`
  compose the `glossary` collection page from its declaration; the other
  eight collection kinds still refuse at declaration (`src/publication/`).
- Exploration-brief generator: the `exploration.exploration-brief` record
  kind now renders into the dossier — the last record kind without a
  renderer (`src/projection/`).
- Tutorial product declaration: `kaya publish` declares all twelve
  vocabulary products; `tutorial` publishes as the spec-pinned
  `no-backing-document` omission until exploration documents carry records
  (`src/publication/decls.rs`).

### Fixed
- `kaya import wai` re-runs are idempotent: a re-observed record with
  identical content skips, and a re-observed record with changed content
  appends a superseding correction event instead of refusing with
  `kaya.ledger.id_reuse` (`src/adapters/`, `src/ledger.rs`).
- `kaya status` caps pending proposals at the session budget instead of
  listing unbounded promotion queues (`src/projection/`).

## [0.2.0] - 2026-11-23

### Added
- Repository bootstrapped on the dulce-de-leche fleet standard: wai tracking,
  beads issue tracking (no-db, tracked export), OpenSpec, lefthook gates
  (`ah check`, `pretender`, `spk lint`), LICENSE (Apache-2.0), and fleet README.
- Six capability specs (`add-core-research-record`: event, ledger, model,
  projection, promotion, adapters) — dual-format openspec deltas + specodelic
  layers, lint-clean, strict-validated.
- mdBook scaffolding (`book.toml`, `theme/charly.css`, `SUMMARY.md`).
- Rust CLI `kaya` (clap derive): `init`, `status`, `import wai|git|test|model`,
  `build [--out] [--expect-digest] [--allow-uncommitted]`, `promote`,
  `challenge`; each executed command prints one genesis-style JSON envelope
  and failures carry a labeled class.
- Event layer: `kaya.event/v0` parsing, the closed 9-field schema, actor and
  provenance handling (`src/event.rs`).
- Ledger: append-only JSONL with SHA-256 digest anchors, verified reads,
  crash-truncated tail recovery, stored-line dedupe, and supersede/correction
  resolution (`src/ledger.rs`).
- Research model: 13 entity types, 9 typed direction-fixed relationships,
  graph validation (`src/model.rs`).
- Projections: research index and experiment reports, generated-block
  ownership, verbatim preservation of foreign blocks, stale authored-section
  detection (`src/projection/`).
- Promotion: proposal/approval/challenge replay with pending counts; CLI
  promotion commands require a named human `--actor` (`src/promotion.rs`).
- turu adapter: `import turu --bundle <PATH> [--project NAME]` reads one turu
  whisper bundle per whisper's shared-store contract and records each bundle
  entry as a `capture.entry.observed` Source node (entry attribute feeds the
  knowledge-brief projection); entry text stays in turu, and a malformed
  bundle appends nothing (`src/adapters/turu.rs`).
- Adapters for the `.wai/` artifact tree, git history, and libtest JSONL run
  reports (`src/adapters/`).
- Verification facts census: `turu` joins the canonical source list, so a
  whisper-bundle entry is counted under `turu` in `events_by_source` and
  `--require-source turu` is a valid request that fails (exit 1, `no-events`)
  when the record holds no turu event (`src/verification/facts.rs`).
- Test report input on a stable toolchain: `scripts/junit-to-libtest.py`
  converts the JUnit XML nextest emits into the libtest JSONL
  `kaya import test --report` consumes, so a stable Rust toolchain can feed
  the test-runner adapter without libtest's nightly-only `--format json`.
- `tdd-ro5u` wai pipeline config (`orient` → `plan` → `red` → `green` →
  `refactor` → `ro5u-review` → `fix-review` → `quality-ledger` →
  `ship-close`), gated on passing tests and human approval.
- CI workflows: `ci.yml` runs `just ci` with pinned tools; `docs.yml` builds
  the mdBook and deploys it to GitHub Pages; `justfile` recipes for
  build/test/lint/docs/spec gates.
- `release.yml`: tag-triggered five-target build (linux/darwin amd64+arm64,
  windows amd64), canonical `kaya_{VERSION}_{os}_{arch}` archives,
  `checksums.txt`, and `provenance.json`; gated on the tagged commit's tests;
  updates the homebrew-charly tap and scoop-charly bucket through
  `scripts/update-homebrew.py` and `scripts/update-scoop.py` (idempotent,
  retried, reported). The crates.io channel is token-gated and disabled until
  publish authorization (`kaya-dgz`).
- Dogfooded dossier: `.kaya/ledger.jsonl` plus the `docs/research/`
  projections (`index.md`, `experiments/add-core-research-record.md`) are
  checked in and rebuilt by `kaya build`.

### Changed
- Docs review pass (Rule of 5, TypeSafe-verified findings): README de-
  duplicated in favor of the design doc, spec suite linked from README/status,
  projection grammar corrected (two block forms, proposals are a status),
  capture→preserve→understand→reveal labels unified across docs, FAQ added,
  data-outlives-kaya guarantee and agent-session-recording rejection stated,
  MVP-1 milestone tense aligned with the spec change.
- README and status now describe the shipped MVP-1 CLI (command surface,
  `.kaya/ledger.jsonl`, `docs/research/` projections, human-approved
  promotion) instead of a design-only phase; install section added for
  `cargo install --git`.
