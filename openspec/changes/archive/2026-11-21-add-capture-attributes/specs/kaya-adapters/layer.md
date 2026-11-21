---
id: kaya.adapters.layer
kind: intent
statement: "WHEN a capture adapter runs, THE kaya.adapters capability SHALL translate source-owned records into validated kaya.event/v0 events carrying source provenance, treat the source's store as authoritative by writing nothing to it, emit observations rather than conclusions, copy no more of the source's content than a bounded first-heading title and its digest, and write no partial events when the run fails."
---

# kaya.adapters — dual-format layer (sidecar)

## Purpose

Kaya is a consumer and integrator, never a second place where the same
knowledge must be entered. Each adapter owns exactly one translation — its
source's records to `kaya.event/v0` — and every adapter names the
source-owned contract it consumes, so the source tool, not kaya, defines
the interface. The wai adapter adds a bounded, content-derived title and
digest to artifact observations: an attribute of the file's own structure,
never the body, never an interpretation.

## Constraints

| id                       | kind      | expr                                                                                                                                                             | traces_to |
|--------------------------|-----------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| source_authority         | invariant | `an adapter reads its source and never writes to the source's store, and never authors content duplicating what the source owns — a re-capture is acknowledged as a duplicate by source reference` | [[kaya.adapters.layer]] |
| translate_not_interpret  | invariant | `adapter events describe what happened at the source; no adapter event carries an interpretation or conclusion field`                                                | [[kaya.adapters.layer]] |
| failure_atomic           | invariant | `a failed adapter run ends with a labeled error naming source and record, and writes no events from that run to the ledger`                                          | [[kaya.adapters.layer]] |
| provenance_linked        | invariant | `every emitted event's provenance names the source tool, its version, and a source-specific reference (artifact path, claim id, bundle digest, commit sha, run output path)` | [[kaya.adapters.layer]] |
| run_failed               | effect    | `kaya.adapters.run_failed(detail)`                                                                                                                                  | [[kaya.adapters.layer]] |
| contract_reference       | invariant | `each adapter's registration names the source-owned contract it consumes: wai (.wai artifact tree), dont (claim store), turu (bundle per whisper shared-store), specodelic (spk parse IR envelope), git (commits and tags), test runner (machine-readable run output)` | [[kaya.adapters.layer]] |
| capture_attribute_bound  | invariant | `content-derived capture attributes are limited to a bounded first-heading title and its digest; absence when no heading exists; event id derivation unaffected`      | [[kaya.adapters.layer]] |

## Model

### States
- `registered`
- `walking`
- `emitted`
- `failed` (emits: [[kaya.adapters.layer.run_failed]])

### Transitions

| id      | from        | to       | guard                                                  |
|---------|-------------|----------|--------------------------------------------------------|
| walk    | registered  | walking  | `source reachable ∧ contract reference named`           |
| emit    | walking     | emitted  | [[kaya.adapters.layer.translate_not_interpret]] ∧ [[kaya.adapters.layer.provenance_linked]] ∧ [[kaya.adapters.layer.source_authority]] ∧ [[kaya.adapters.layer.capture_attribute_bound]] |
| abort   | walking     | failed   | `¬([[kaya.adapters.layer.translate_not_interpret]] ∧ [[kaya.adapters.layer.provenance_linked]] ∧ [[kaya.adapters.layer.source_authority]] ∧ [[kaya.adapters.layer.capture_attribute_bound]])` |

## Properties

| id                           | kind | derives_from                              | generator                              | predicate                                                  |
|------------------------------|------|-------------------------------------------|----------------------------------------|-------------------------------------------------------------|
| no_source_writes             | unit | [[kaya.adapters.layer.source_authority]]        | `fixture_workspace_all_sources()`      | `∀ source: tree byte-identical after run`                    |
| facts_only_events            | unit | [[kaya.adapters.layer.translate_not_interpret]] | `fixture_capture_run()`                | `no emitted event carries an interpretation field`           |
| atomic_failure               | unit | [[kaya.adapters.layer.failure_atomic]]          | `mid_walk_invalid_record()`            | `labeled error ∧ zero events from the run`                   |
| provenance_per_adapter       | unit | [[kaya.adapters.layer.provenance_linked]]       | `first_event_per_adapter()`            | `provenance names tool ∧ version ∧ reference`                |
| contract_reference_recorded  | unit | [[kaya.adapters.layer.contract_reference]]      | `adapter_registry_fixture()`           | `registration names the consumed source contract`            |
| atomic_failure_labeled       | unit | [[kaya.adapters.layer.failure_atomic]]    | `mid_walk_invalid_record()`            | `labeled error names source ∧ record ∧ zero events`          |
| run_failed_label_asserted    | unit | [[kaya.adapters.layer.run_failed]]        | `adapter_run_failed_raised()`          | `error_label == "kaya.adapters.run_failed"`                  |
| first_hash_heading_title     | unit | [[kaya.adapters.layer.capture_attribute_bound]] | `fixture_with_hash_heading()`    | `data.title == heading remainder ∧ data.title_digest == sha256(title)[..16]` |
| multibyte_title_truncated_safely | unit | [[kaya.adapters.layer.capture_attribute_bound]] | `fixture_with_multibyte_heading()` | `title cut on a char boundary ∧ chars().count() <= 80`     |
| title_absent_without_heading | unit | [[kaya.adapters.layer.capture_attribute_bound]] | `fixture_without_heading()`      | `data omits title ∧ title_digest`                            |
| long_title_truncated_to_bound | unit | [[kaya.adapters.layer.capture_attribute_bound]] | `fixture_with_long_heading()`   | `title.chars().count() == 80`                                |
| event_id_independent_of_content_attributes | unit | [[kaya.adapters.layer.capture_attribute_bound]] | `same_path_edited_heading()` | `event id unchanged across heading edits`                |

## Requirements
### Requirement: Layer sidecar contract

This sidecar is the dual-format layer for the `kaya.adapters` capability
and SHALL be deployed verbatim by the archive companion; the plain delta in
`spec.md` beside it is its openspec face.

#### Scenario: Archive deploys the layer

- **WHEN** the change `add-capture-attributes` is archived after its
  scenarios have executing contract tests
- **THEN** the capability deploys with this layer attached fail-closed
