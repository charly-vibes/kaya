# Tasks: add-capture-attributes

Red→green per the kaya-q5e ticket; each scenario below is a contract-test
shape bound in `.espectacular`.

## 1. Red — failing tests first (kaya-q5e)

- [x] 1.1 `capture_extracts_first_hash_space_heading_as_title` — first
      `# ` heading line becomes `title`; `title_digest` is the first 16 hex
      of sha256 over the title bytes.
- [x] 1.2 `capture_truncates_multibyte_title_without_char_boundary_panic`
      — truncation cuts on a character boundary.
- [x] 1.3 `capture_omits_title_attributes_when_file_empty_or_heading_less`
      — regression pin: attributes absent without a heading.
- [x] 1.4 `capture_truncates_long_title_to_the_bounded_constant` — the
      bounded constant is `TITLE_MAX_LEN` (80).
- [x] 1.5 `renamed_path_event_id_keeps_stem_digest_stability` — pin: the
      event id never depends on content-derived attributes.

## 2. Green — make the tests pass

- [x] 2.1 `src/adapters/wai.rs`: `TITLE_MAX_LEN`, `capture_attributes`,
      wiring into `capture_file` with a content-independent id payload.
- [x] 2.2 Module doc header rewritten to the ownership rationale.
- [x] 2.3 Update `capture_emits_one_event_per_markdown_artifact`
      `data.len()` 2 → 4 (all fixtures carry a heading) in the same change.
- [x] 2.4 Spec deltas (kaya-adapters MODIFIED, kaya-model ADDED) +
      layer sidecars + `.espectacular` bindings.

## 3. Verify (orchestrator)

- [ ] `cargo test --test wai` green.
- [ ] `openspec validate add-capture-attributes --strict` passes.
- [ ] `ah check` clean.
