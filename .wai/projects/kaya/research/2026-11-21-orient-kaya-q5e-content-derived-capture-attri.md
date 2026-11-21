---
tags: [pipeline-run:tdd-ro5u-2026-11-21-kaya-q5e-capture-attributes, pipeline-step:orient]
---

# ORIENT — kaya-q5e (Content-derived capture attributes in wai events)

Pipeline run tdd-ro5u-2026-11-21-kaya-q5e-capture-attributes. Base commit fbec965.

## Confirmed metadata

- Lane: afk; complexity: l. Files: `src/adapters/wai.rs`, `src/model.rs`.
- My scope: orient + plan artifacts + RED tests only (tests/ only, never src/).

## Acceptance restated as testable statements

- openspec change dir exists (suggest `openspec/changes/add-capture-attributes/`) and
  `openspec validate --changes --strict` passes (orchestrator runs; I only write the deltas).
- `ah check` clean (binding tomls naming new test fns are part of the change).
- RED tests pin deterministic derivation:
  1. first `# ` heading line becomes the `title` attribute;
  2. truncation is UTF-8-safe (no char-boundary panic on multibyte);
  3. attribute absent when file is empty or has no `# ` heading;
  4. long title truncates to the bounded constant;
  5. event id stays derived from path stem+digest inputs only — editing the
     derived title does not change the event id (anti-goal: no id-derivation change).
- Spec delta states the ownership decision verbatim: "bounded content-derived
  attribute != text ownership; wai keeps full text".
- `src/adapters/wai.rs` module doc (Purpose/Responsibilities/Rationale) is rewritten
  to the amended ownership rationale at green time (orchestrator/implementer).

## Current wai ownership rationale row (kaya-adapters spec, verbatim quotes)

Invariant `source_authority` (openspec/specs/kaya-adapters/spec.md:21):

> `an adapter reads its source and never writes to the source's store, and never authors content duplicating what the source owns — a re-capture is acknowledged as a duplicate by source reference`

Requirement "Source authority retained" (spec.md:58-63):

> An adapter SHALL read its source's records and SHALL never write to the source's store, and SHALL never author content that duplicates what the source already owns — kaya holds projections and interpretation, the source holds its own truth.

Requirement "Translate, never interpret" (spec.md:77-88) — the constraint the new
attributes must be reconciled with: no adapter event carries an interpretation or
conclusion field. A bounded first-heading title is an observation of the file's own
structure, not an interpretation.

Module doc today (src/adapters/wai.rs:1-8): "Rationale: wai owns the artifact text;
kaya only needs a stable, replayable observation naming where the artifact lives."

Current payload (wai.rs:115): `data = { kind, path }` — exactly two keys; the existing
contract test `capture_emits_one_event_per_markdown_artifact` asserts `data.len() == 2`
(tests/wai.rs). At green this assertion must move to the attribute-aware shape.

## How title/digest fit the existing event model

- Event vocabulary: `capture.artifact.observed` is `ARTIFACT_OBSERVED_EVENT`
  (src/model.rs:24) among five event-type constants; the model consumes it as a
  capture event type (src/model.rs:411). Attributes ride inside `data`
  (`serde_json::Value`), one of the nine closed `kaya.event/v0` fields — adding
  `title` / `title_digest` keys to `data` needs no schema revision.
- The nine-field rule is a kaya.event constraint (`kaya.event/v0` closed field set:
  id, schema, type, timestamp, actor, subject, data, provenance, artifacts — see
  archived delta specs/kaya-event/spec.md). `data` is open content; the closed
  vocabulary that matters here is kaya.model's thirteen EntityType set — unchanged.
- Planned derivation (deterministic): scan lines for the first starting with
  `# `; title = remainder trimmed, truncated to `TITLE_MAX_LEN` (80) on a char
  boundary; `title_digest` = first 16 hex of `sha256_hex(title bytes)`
  (`kaya::adapters::sha256_hex` is pub re-exported, src/adapters/mod.rs:16).

## Uncertainties

- `openspec/AGENTS.md` referenced by root AGENTS.md but absent in the repo; change-dir
  conventions mirrored from `openspec/changes/archive/2026-11-18-add-core-research-record/`
  (proposal.md, tasks.md, specs/<cap>/spec.md with `## ADDED`/`## MODIFIED Requirements`,
  plus dual-format `layer.md` sidecars).
- `openspec/changes/` currently holds only `archive/` — the referenced
  `add-core-research-record` lives under archive; deployment = mirror its layout.
- Tests (c) attribute-absent and (e) id-stability pin behavior that already holds
  pre-green; they are regression pins, expected to pass red. The other three fail
  on assertions.
- Boundary: kaya-8f4 (publication block vocabulary) deliberately untouched.

