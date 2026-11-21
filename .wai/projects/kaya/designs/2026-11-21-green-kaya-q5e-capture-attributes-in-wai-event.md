---
tags: [pipeline-run:tdd-ro5u-2026-11-21-kaya-q5e-capture-attributes, pipeline-step:green]
---

# GREEN kaya-q5e — capture attributes in wai events

Run: tdd-ro5u-2026-11-21-kaya-q5e-capture-attributes (step 4/9 green).

## Shipped
- `src/adapters/wai.rs`:
  - `pub const TITLE_MAX_LEN: usize = 80` (character bound, char-boundary-safe cut).
  - `TITLE_DIGEST_LEN: usize = 16` (private).
  - `pub fn capture_attributes(&str) -> Option<(String, String)>`: first line starting
    with `"# "` → title = remainder trimmed + truncated to 80 chars via
    `chars().take()`; digest = first 16 hex of `sha256_hex(title bytes)`. `None`
    when empty file or no `# ` heading (covers heading-less documents).
  - `capture_file` gains optional `title` / `title_digest` keys in the open `data`
    map of `capture.artifact.observed` events, only when derivable.
  - **Event-id independence**: id is computed over the identity payload
    `{kind, path}` only — content-derived attributes are inserted after hashing,
    so ids never move on heading edits (anti-goal respected; pin test
    `renamed_path_event_id_keeps_stem_digest_stability` passes by construction).
  - Module doc header rewritten to the amended ownership rationale:
    "bounded content-derived attribute != text ownership; wai keeps full text".
- `src/model.rs`: no change — `capture_from_event` whitelist and the nine-field
  `kaya.event/v0` envelope untouched; attributes ride the open `data` map.
- `openspec/changes/add-capture-attributes/`: proposal.md, tasks.md,
  specs/kaya-adapters/spec.md (MODIFIED rationale row, ownership decision verbatim
  + five scenarios), specs/kaya-model/spec.md (ADDED attribute rows), dual-format
  layer.md sidecars with the `capture_attribute_bound` constraint row.
- `.espectacular/`: 5 kaya-adapters + 3 kaya-model binding tomls naming the new
  test fns.
- `tests/wai.rs`: five red tests now green; sanctioned edits: `data.len()` 2→4 in
  the existing contract test (fixtures now carry the two new keys); multibyte
  fixture `repeat(30)`→`repeat(90)` (original never reached the 80-char bound, so
  its assert was unsatisfiable by any implementation).

## Verification (orchestrator)
- Pre-refactor: `cargo test` full suite — all targets ok, no failures.
- Post-refactor: `cargo test --test wai` + `cargo clippy --all-targets -- -D warnings`.

## Deviations from plan
- Multibyte red-test fixture length fixed as above (red-test bug, assertion intent unchanged).
- `src/model.rs` untouched — attribute vocabulary needed no model-side field; the
  kaya.model delta documents the attribute semantics on the open `data` map.

