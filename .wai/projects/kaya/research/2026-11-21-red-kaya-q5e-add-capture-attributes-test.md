---
tags: [pipeline-run:tdd-ro5u-2026-11-21-kaya-q5e-capture-attributes, pipeline-step:red]
---

# RED — kaya-q5e (add-capture-attributes)

## Tests added (all in `tests/wai.rs`, appended after `registry_names_the_wai_source_contract`)

1. `capture_extracts_first_hash_space_heading_as_title` — criterion (a)
2. `capture_truncates_multibyte_title_without_char_boundary_panic` — criterion (b)
3. `capture_omits_title_attributes_when_file_empty_or_heading_less` — criterion (c)
4. `capture_truncates_long_title_to_the_bounded_constant` — criterion (d)
5. `renamed_path_event_id_keeps_stem_digest_stability` — criterion (e)

All compile against current public APIs only (`wai::capture`, `by_path`,
`write_file`, pub re-export `kaya::adapters::sha256_hex`). No planned-API
symbol (`TITLE_MAX_LEN`, `capture_attributes`) is referenced, so there are no
expected compile failures — every red failure is an assertion failure.

## Orchestrator command

```
cargo test --test wai
```

Narrower filter per test: `cargo test --test wai capture_extracts_first_hash_space_heading_as_title` etc.

## Expected failures pre-green

- Tests 1, 2, 4: FAIL — `capture.artifact.observed` data today carries only
  `kind`/`path` (src/adapters/wai.rs:115), so `data["title"]` is `Null`
  (1: expected-title assert), (2: no title present), (4: bounded-title assert).
- Test 3: PASSES (pin) — absence of title attributes already holds today; it
  guards the green implementation against emitting empty-title keys.
- Test 5: PASSES (pin) — event id today already derives only from
  tool+version+project+timestamp+reference+data, and title edits change
  `data` only after green; pre-green it pins that content does not affect the
  id. Post-green it pins the anti-goal "no change to event-id derivation":
  note that `data` feeds `event_id` (wai.rs:265-270), so the green commit MUST
  keep id derivation content-independent (e.g. hash only `kind`+`path` from
  data) or this pin breaks — that is the acceptance criterion.
- The existing test `capture_emits_one_event_per_markdown_artifact` asserts
  `data.len() == 2` and will FAIL at green until updated (per plan.md).

## src/ untouched

Only `tests/wai.rs` edited; `src/adapters/wai.rs` and `src/model.rs` untouched.
Artifacts: `/tmp/kaya-q5e/orient.md`, `/tmp/kaya-q5e/plan.md`, this file.

