# Proposal: add-capture-attributes

## Why

A captured artifact is currently named only by its path (`kind` + `path` in
the event's `data`), so a dossier reader cannot tell what an artifact is
without leaving kaya and opening the wai file. A bounded, content-derived
title and its digest are an observation of the file's own structure — not an
interpretation — and are safe to copy without violating the ownership
decision: bounded content-derived attribute != text ownership; wai keeps
full text.

## What Changes

- `src/adapters/wai.rs`: add `TITLE_MAX_LEN` (80) and
  `capture_attributes(&str) -> Option<(String, String)>`; the wai capture
  event's `data` gains optional `title` and `title_digest` keys, present
  only when the artifact carries a `# ` heading.
- Event ids stay derived from tool, version, project, timestamp, reference,
  and the path-identity payload (`kind` + `path`) only — never from file
  content. Editing a heading changes the title attribute, never the id.
- Module doc header amended to the ownership rationale above.
- Spec deltas: `kaya-adapters` MODIFIED (Source authority retained, whose
  reconciliation sentence now covers the bounded first-heading title as an
  observation of the file's own structure), `kaya-model` ADDED (bounded
  capture attributes vocabulary).
- `.espectacular` bindings for the five new contract tests.

## Impact

- Specs: kaya-adapters MODIFIED, kaya-model ADDED requirement.
- Code: `src/adapters/wai.rs` only (plus one assertion update in
  `tests/wai.rs`, documented in tasks.md).
- Tickets: kaya-q5e.
