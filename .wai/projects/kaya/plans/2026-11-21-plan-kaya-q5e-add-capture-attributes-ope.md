---
tags: [pipeline-run:tdd-ro5u-2026-11-21-kaya-q5e-capture-attributes, pipeline-step:plan]
---

# PLAN — kaya-q5e (add-capture-attributes)

## Openspec change dir

`openspec/changes/add-capture-attributes/` (verb-first name matches the archive
convention `add-core-research-record`). Layout mirrors
`openspec/changes/archive/2026-11-18-add-core-research-record/`:

```
openspec/changes/add-capture-attributes/
  proposal.md
  tasks.md
  specs/kaya-adapters/spec.md    (## MODIFIED Requirements)
  specs/kaya-adapters/layer.md   (dual-format sidecar, constraints retyped)
  specs/kaya-model/spec.md       (## MODIFIED Requirements)
  specs/kaya-model/layer.md
```

## File-by-file change list (implementer's green work)

1. `src/adapters/wai.rs`
   - `pub const TITLE_MAX_LEN: usize = 80;` — bounded constant (doc: kept on a
     char boundary; UTF-8-safe).
   - `const TITLE_DIGEST_LEN: usize = 16;`
   - `pub fn capture_attributes(contents: &str) -> Option<(String, String)>` —
     first line starting with `"# "`; title = rest trimmed, then
     `title.chars().take(TITLE_MAX_LEN).collect()` (char-safe); returns
     `Some((title, sha256_hex(title.as_bytes())[..TITLE_DIGEST_LEN].to_owned()))`;
     `None` when no heading (covers empty file).
   - `capture_file` (wai.rs:107-112): use the already-read string (currently
     discarded) to derive attributes; `data` becomes
     `{ kind, path, title?, title_digest? }` — keys present only when
     `capture_attributes` returns `Some`. `json!` built via a `Map` to keep
     key order irrelevant (serde_json Map is BTree-backed, replay stable).
   - Module doc rewrite (lines 1-8), sketch:

     ```
     //! wai adapter: `.wai/` markdown artifacts to `capture.artifact.observed`.
     //!
     //! Purpose: record that a research, design, plan, or handoff artifact
     //! exists in the wai artifact tree, with a bounded, content-derived
     //! title digest for stable identification. Responsibilities: walk the
     //! artifact tree read-only, derive a deterministic event id per
     //! artifact, and abort atomically when a record cannot be read.
     //! Rationale: ownership decision — bounded content-derived attribute
     //! != text ownership; wai keeps full text. kaya copies only the
     //! bounded first-heading title and its digest, never the artifact
     //! body, and never interprets it.
     ```
2. `src/model.rs` — no code change expected (`data` is opaque to the graph
   builder); the file is listed in the ticket because the vocabulary constants
   (src/model.rs:21-27) are the event vocabulary the delta amends. Touch only
   if the implementer finds a doc-comment naming the two-key payload.
3. `tests/wai.rs` — at green, update `capture_emits_one_event_per_markdown_artifact`
   `data.len() == 2` to expect 2 or 4 keys and drop the red-only assertions if
   superseded; keep the new determinism tests.
4. `.espectacular/kaya-adapters/first-hash-heading-title-observed.toml`,
   `multibyte-title-truncated-safely.toml`,
   `title-absent-without-heading.toml`,
   `long-title-truncated-to-bound.toml`,
   `event-id-independent-of-content-attributes.toml` — same shape as existing
   tomls (`tool_version = "0.12.2"`, `archetype = "PF"`, `status = "active"`,
   `[tests] cargo = [{flags = "<test fn>"}]`).

## Test names (exact) → criteria

In `tests/wai.rs`, reusing `write_file`, `by_path`:

| criterion                       | test fn                                            |
|---------------------------------|----------------------------------------------------|
| (a) first `# ` heading → title  | `capture_extracts_first_hash_space_heading_as_title` |
| (b) UTF-8-safe truncation       | `capture_truncates_multibyte_title_without_char_boundary_panic` |
| (c) absent when empty/heading-less | `capture_omits_title_attributes_when_file_empty_or_heading_less` |
| (d) long-title truncation       | `capture_truncates_long_title_to_the_bounded_constant` |
| (e) id stability vs content     | `renamed_path_event_id_keeps_stem_digest_stability` |

## Draft delta rows

### specs/kaya-adapters/spec.md — MODIFIED

Requirement **Source authority retained** — new closing paragraph on the
requirement text:

> A bounded content-derived attribute != text ownership; wai keeps full text.
> The wai adapter MAY copy only a bounded first-heading title and its digest
> from a record's own structure; it SHALL never copy the artifact body or any
> arbitrary text.

New scenario under it:

```
#### Scenario: Bounded title attribute is not text ownership

- **WHEN** the wai adapter captures an artifact whose first line is a
  `# ` heading
- **THEN** the event carries only the bounded title and a short digest of
  it, the artifact body stays wai's truth, and the ownership decision
  "bounded content-derived attribute != text ownership; wai keeps full
  text" governs
```

Scenario mirroring test (d):

```
#### Scenario: Long title truncates to the bounded constant

- **WHEN** the first heading's text exceeds the bounded title length
- **THEN** the captured title is the first bound-many characters, cut on a
  character boundary, and never panics on multibyte text
```

Scenario for (c):

```
#### Scenario: No heading, no title attribute

- **WHEN** an artifact is empty or carries no `# ` heading line
- **THEN** the event's data omits the title attributes entirely
```

### specs/kaya-model/spec.md — MODIFIED

Requirement (amend the capture-vocabulary statement; kaya-model currently has
no artifact-attribute row, so this is **ADDED** Requirements):

```
### Requirement: Bounded capture attributes in artifact observations

A `capture.artifact.observed` event MAY carry two content-derived data
attributes: `title` (the artifact's first `# ` heading, bounded and cut on
a character boundary) and `title_digest` (a short digest of the bounded
title). Both SHALL be absent when no heading exists, and the event id
SHALL remain derived from path inputs only — stem and digest — never from
file content. Ownership decision: bounded content-derived attribute !=
text ownership; wai keeps full text.

#### Scenario: Title and digest derive deterministically

- **WHEN** the same artifact is captured twice
- **THEN** the title and title_digest attributes are byte-identical

#### Scenario: Content edit does not move the event id

- **WHEN** only an artifact's heading text changes and it is re-captured
- **THEN** the event id is unchanged — id derivation stays path stem +
  digest
```

Constraint table addition (both spec.md and layer.md), row:

| id | kind | expr | traces_to |
|----|------|------|-----------|
| capture_attribute_bound | invariant | `content-derived capture attributes are limited to a bounded first-heading title and its digest; absence when no heading exists; event id derivation unaffected` | [[kaya.model]] |

## Deployment steps (mirror add-core-research-record)

1. Create the change dir + files above; keep proposal.md Why/What/Impact
   sections, tasks.md red→green→refactor checklist with the bd ticket id.
2. `openspec validate add-capture-attributes --strict` (orchestrator).
3. `ah check --changes add-capture-attributes` for binding validation, then
   `ah check` clean after deployment into `openspec/specs/` at archive.
4. Commit order: red tests → spec delta + tomls → green implementation +
   module doc rewrite.

## Risks

- Existing contract test `capture_emits_one_event_per_markdown_artifact`
  hard-asserts `data.len() == 2` — breaks at green; must be updated in the
  same commit (documented above).
- Adding `title`/`title_digest` to `data` changes `event_id`'s data input;
  events captured pre-green differ post-green (one-time id churn on
  re-capture). Acceptable: derivation formula unchanged; noted in proposal.
- Truncation must use `chars()`, not byte slicing — pinned by test (b).

