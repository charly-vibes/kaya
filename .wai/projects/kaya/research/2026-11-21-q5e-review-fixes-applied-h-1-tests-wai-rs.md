---
tags: [pipeline-run:tdd-ro5u-2026-11-21-kaya-q5e-capture-attributes, pipeline-step:fix-review]
---

# q5e review fixes applied

## H-1 — tests/wai.rs:436 (`renamed_path_event_id_keeps_stem_digest_stability`)
Dated the fixture filename: `.wai/projects/kaya/designs/renamed-stability.md` → `.wai/projects/kaya/designs/2026-04-05-stability.md`, so `timestamp_for` derives the fixed date from the filename instead of wall-clock mtime seconds; both captures now produce identical event ids regardless of timing. Test assertions unchanged (title changes across edit, id and subject id stable).

## H-2 — tests/wai.rs:419,426 (`capture_truncates_long_title_to_the_bounded_constant`)
Heading extended to 115 chars ("…and keeps going far past eighty characters total"), so truncation actually runs; assertion tightened from `<= 80` to `assert_eq!(title.chars().count(), wai::TITLE_MAX_LEN)`, matching the layer predicate and the `.espectacular/kaya-adapters/long-title-truncated-to-bound.toml` scenario. The `starts_with("a very long heading")` assertion retained.

## M-1 — openspec/changes/add-capture-attributes/specs/kaya-model/layer.md
- Line 4 (frontmatter statement): "carrying only bounded content-derived capture attributes on Artifact nodes" → "admitting capture events whose observations carry only bounded content-derived capture attributes" (event-level carriage; matches `src/model.rs` `capture_from_event(..., ["path","kind"])`).
- Purpose (lines 13–17): "Artifact observations carry…" → "Capture events carry…", with explicit note that title/digest ride the `capture.artifact.observed` event's data map and are never copied onto Artifact nodes, which keep only `path` and `kind`.

## L-3 — src/adapters/wai.rs:297
`event_id` doc comment "over every field the event writes" → "over the path-identity payload" (rest of the doc already enumerated the {kind, path} + project + timestamp inputs correctly).

## L-4 — both layer sidecars
- `traces_to` aligned to the archived 2026-11-18 pattern (all rows trace own layer): `capture_attribute_bound` in `specs/kaya-model/layer.md:29` → `[[kaya.model.layer]]`, in `specs/kaya-adapters/layer.md:29` → `[[kaya.adapters.layer]]` (was `[[kaya.model]]` in both).
- Abort guard in `specs/kaya-adapters/layer.md:45` now negates the full emit conjunction, adding `∧ [[kaya.adapters.layer.capture_attribute_bound]]`, mirroring the archived change's ¬-conjunction style.

## L-5 — openspec/changes/add-capture-attributes/proposal.md:23-26
"What Changes" bullet reworded: dropped the nonexistent "translate-not-interpret reconciliation" delta reference; now says `kaya-adapters` MODIFIED (Source authority retained, whose reconciliation sentence covers the bounded first-heading title as an observation of the file's own structure).

## Deferred (not fixed, per assignment)
L-1 (empty `# ` heading → empty title) and L-2 (BOM-prefixed heading uncaptured) — untouched.

## Deviations
None.

