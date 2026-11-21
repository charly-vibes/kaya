# Rule-of-5 Review — kaya-q5e capture attributes in wai events

**Work reviewed:** uncommitted working-tree diff — `src/adapters/wai.rs`, `tests/wai.rs`, `openspec/changes/add-capture-attributes/**` (proposal, tasks, 2 spec deltas, 2 layer sidecars), 8 new `.espectacular/*.toml` bindings.
**Method:** Rule-of-5 lenses (Draft/Correctness/Clarity/Edge cases/Excellence), single convergence pass — Stage 2 surfaced the substantive defects; Stages 3–5 added only wording/edge items. Converged at Stage 4 (0 new CRITICAL after Stage 2, new-issue rate < 10%).

**CRITICAL count: 0** (explicit)

## What was verified correct

- **Derivation** (`src/adapters/wai.rs:117-125`): first `# `-prefixed line, remainder trimmed, char-boundary-safe truncation via `chars().take(TITLE_MAX_LEN)`, digest over the *bounded* title's UTF-8 bytes — matches the kaya-model delta wording exactly.
- **Event-id stability**: `capture_file` computes `id` from `{kind, path}` payload *before* inserting `title`/`title_digest`; `event_id` itself is untouched (anti-goal respected). `data` is a free-form `serde_json::Value` (`src/event.rs:188`), so the `deny_unknown_fields` on the fixed `Event` fields does not reject the new keys, and `model::build`'s capture branch tolerates extra keys (it selects fields explicitly). No consumer breakage; `facts.rs` only counts events.
- **Edge cases handled**: CRLF (`str::lines()` strips `\r`), heading exactly at the 80-char boundary, multiple headings (first wins), `#` without space (not a `# ` heading → omitted), byte-safe `line[2..]` (both prefix bytes ASCII). `sha256_hex` is exported (`src/adapters/mod.rs:16`).
- **Spec delta format** conforms to the archived 2026-11-18 change: `## MODIFIED/ADDED Requirements`, `#### Scenario:` + bold WHEN/THEN; the MODIFIED "Source authority retained" keeps both pre-existing scenarios verbatim and replaces the whole requirement (openspec-safe). Both layer sidecars keep the archived section layout (frontmatter / Purpose / Constraints / Model / Properties / Requirements).
- **Bindings**: all 8 tomls match the existing schema (`tool_version`, `id`, `archetype = "PF"`, `[tests] cargo = [{flags = ...}]`, cf. `no-source-writes.toml`) and every `flags` value names a test fn that exists in `tests/wai.rs`.
- **Module doc header** rewritten with the ownership rationale, verbatim in the delta. `tasks.md` 2.3 assertion update (2→4) matches the fixtures (all five carry headings).

## Findings

### Critical

None.

### High

**H-1. `renamed_path_event_id_keeps_stem_digest_stability` is timing-flaky: undated record makes the id depend on wall-clock mtime seconds**
`tests/wai.rs:434-451` — the record `.wai/projects/kaya/designs/renamed-stability.md` has no leading date, so `timestamp_for` falls back to the file's modification time at *second* precision (`src/adapters/wai.rs:228-237`, `to_rfc3339_opts(SecondsFormat::Secs, true)`), and the timestamp is an input to `event_id` (`src/adapters/wai.rs:297-305`, identity includes `{timestamp}`). The test writes the file twice and asserts `before.id == after.id`; if the two `fs::write` calls straddle a second boundary (likely under CI load), the ids legitimately differ and the test fails without any id-derivation regression. The codebase already handles this hazard — `fixture()` pins `FIXED_MTIME_SECS` on `RENAMED` for exactly this reason ("the renamed design has no date") — but the new test omits both the dated filename and the mtime pin.
```suggestion
    let temp = tempfile::tempdir().unwrap();
    let record = ".wai/projects/kaya/designs/2026-04-05-stability.md";
```
(a dated filename makes `timestamp_for` return the fixed date `2026-04-05T00:00:00Z` for both captures; alternatively pin the mtime like `fixture()` does)

**H-2. `capture_truncates_long_title_to_the_bounded_constant` never exercises truncation — its fixture heading is 66 chars, below the 80 bound**
`tests/wai.rs:412-431` — the heading remainder `"a very long heading that goes well beyond the bounded title length"` is exactly 66 characters (measured: `wc -m` → 66), so `chars().take(80)` is a no-op and the assertion `title.chars().count() <= 80` passes even with truncation deleted, halved to `take(40)`, or off-by-N. The binding `.espectacular/kaya-adapters/long-title-truncated-to-bound.toml` describes this test as a "Scenario mirror" of the delta scenario "Long title truncated to the bounded constant" (WHEN heading exceeds the bound THEN title is the first 80 chars), and `layer.md:61` states the predicate as `title.chars().count() == 80` — neither holds for this fixture. (Actual truncation behavior is incidentally covered by the multibyte test's `== 80` over a 90-char heading, but this scenario/binding claims coverage it does not provide.)
```suggestion
    write_file(
        temp.path(),
        record,
        "# a very long heading that goes well beyond the bounded title length and keeps going far past eighty characters total\n",
    );

    let events = wai::capture(temp.path(), PROJECT).unwrap();
    let event = by_path(&events, record);

    let title = event.data["title"].as_str().expect("title present");
    assert_eq!(title.chars().count(), wai::TITLE_MAX_LEN);
```

### Medium

**M-1. kaya-model layer sidecar claims the attributes live "on Artifact nodes", but `model::build` never copies them to nodes**
`openspec/changes/add-capture-attributes/specs/kaya-model/layer.md:4` — the statement deployed verbatim by the archive companion says the capability carries "only bounded content-derived capture attributes **on Artifact nodes**", and the Purpose section (line 13) repeats that "Artifact observations carry a bounded, content-derived title and its digest". The code's consumer side does not do this: `src/model.rs:425` builds Artifact nodes with `capture_from_event(event, EntityType::Artifact, &["path", "kind"], …)` — only `path` and `kind` become node attributes; `title`/`title_digest` are dropped. The spec delta requirement itself is event-scoped ("A `capture.artifact.observed` event MAY carry…") and consistent with the implementation, so the sidecar wording overstates the change. Fix the wording (extending `fields` would be new scope beyond the ticket):
```suggestion
statement: "THE kaya.model capability SHALL connect admitted events into a research graph over a closed thirteen-entity vocabulary with a closed set of typed, direction-fixed relationships, admitting capture events whose observations carry only bounded content-derived capture attributes, refusing unknown entities, unknown relationships, dangling references, and influence cycles, with namespace-qualified node ids so ledgers compose across repositories."
```

### Low

**L-1. Degenerate `# ` heading yields `title: ""` plus digest-of-empty-string**
`src/adapters/wai.rs:117-125` — a file whose only heading line is `# ` or `#   ` passes `starts_with("# ")`, and `line[2..].trim()` yields `""`, so the event carries `title: ""` and `title_digest` = sha256 of the empty string. The delta only mandates absence for "empty or carries no `# ` heading", so this is spec-literal, but an empty title attribute is a poor observation state the spec never intended to represent. Skip whitespace-only remainders:
```suggestion
    let heading = contents
        .lines()
        .find(|line| line.starts_with("# ") && !line[2..].trim().is_empty())
        .map(|line| line[2..].trim())?;
```

**L-2. UTF-8 BOM before the heading silently defeats title capture**
`src/adapters/wai.rs:120` — for a BOM-prefixed file (`\u{FEFF}# heading`, common from Windows-authored markdown), the first line starts with U+FEFF, `starts_with("# ")` is false, and the attributes are absent despite a plainly visible heading. Spec-literal ("a line starting with `# `") but an unintended-looking UX gap; strip a leading BOM from the first line or document the exclusion in the delta.

**L-3. Stale doc on `event_id`: "over every field the event writes" is now false**
`src/adapters/wai.rs:297-301` — the patch deliberately excludes the content-derived attributes from the id payload, so the doc's first line contradicts the new behavior explained at the call site. Reword:
```suggestion
/// Deterministic event id over the path-identity payload.
```

**L-4. Layer sidecar conformance: `traces_to` deviation and an unreachable abort guard**
`openspec/changes/add-capture-attributes/specs/kaya-adapters/layer.md:29` and `specs/kaya-model/layer.md:27` — the new `capture_attribute_bound` rows trace to `[[kaya.model]]` while every other row in both files (and the archived 2026-11-18 sidecars) traces to the row's own layer (`[[kaya.adapters.layer]]` / `[[kaya.model.layer]]`). Additionally `kaya-adapters/layer.md:44-45`: `emit` now requires `capture_attribute_bound` but `abort` negates only the original three constraints, so a violation of just that constraint satisfies neither guard and the state machine is stuck in `walking`. Make `abort` negate the same conjunction as `emit`:
```suggestion
| abort   | walking     | failed   | `¬([[kaya.adapters.layer.translate_not_interpret]] ∧ [[kaya.adapters.layer.provenance_linked]] ∧ [[kaya.adapters.layer.source_authority]] ∧ [[kaya.adapters.layer.capture_attribute_bound]])` |
```

**L-5. Proposal overstates the kaya-adapters delta scope**
`openspec/changes/add-capture-attributes/proposal.md` ("What Changes") says "kaya-adapters MODIFIED (Source authority retained **+ translate-not-interpret reconciliation**)", but the delta modifies only "Source authority retained"; the reconciliation sentence ("A bounded first-heading title is an observation of the file's own structure, not an interpretation.") lives inside that requirement. Align the proposal wording or move the sentence.

## Non-issues checked (so nobody re-litigates)

- Leading whitespace before a heading (`  # title`) is not captured — spec-literal ("a line starting with `# `"), intentional per the delta.
- Digest over the *bounded* title means two headings sharing 80 prefix chars collide — deliberate, stated in both the code doc and the delta.
- Exactly-80-char heading passes through whole (boundary correctness); CRLF handled by `lines()`; `unparsable_record_aborts_atomically` still pins the invalid-UTF-8 path since `read_to_string` precedes attribute extraction.
- Cross-capability duplicate bindings (`multibyte-title-truncated-safely` in both kaya-adapters and kaya-model) mirror the duplicated scenarios across the two deltas — consistent with the archived pattern of per-capability scenario sets.

## Verdict block (for `wai add review`)

- overall: pass (conditional — land H-1 and H-2 before merge; no production-code blockers)
- critical_count: 0
- high_count: 2
- medium_count: 1
- low_count: 5
- top_fixes:
  1. H-1 `tests/wai.rs:434` — date the stability-test record (or pin its mtime); undated record makes the asserted id equality depend on wall-clock seconds and will flake CI.
  2. H-2 `tests/wai.rs:412` — make the long-title fixture exceed 80 chars and assert `chars().count() == wai::TITLE_MAX_LEN`; the current 66-char fixture never truncates, so the scenario mirror claimed by `.espectacular/kaya-adapters/long-title-truncated-to-bound.toml` is untested.
  3. M-1 `specs/kaya-model/layer.md:4` — reword "on Artifact nodes" to event-level carriage; `model::build` (`src/model.rs:425`) copies only `path`/`kind` into node attributes.
