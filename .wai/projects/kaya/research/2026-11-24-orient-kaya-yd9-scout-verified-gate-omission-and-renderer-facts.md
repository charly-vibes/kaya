---
tags: [pipeline-run:tdd-ro5u-2026-11-24-kaya-yd9-implement-add-collection-kinds, pipeline-step:orient]
---

# Research: kaya-yd9 orient — implement add-collection-kinds

Scout-verified facts (file:line from working tree @ main, post-v0.3.0).

## The refusal seam

- `src/publication/plan.rs:182-193` — collection arm: refuses stray `includes`/`members` via `tier_fields`, then `if decl.name != "glossary" { return Err(PublicationError::tier_unimplemented(&decl.name)); }`. Refusal happens before any write (`plan()` calls `partition_declarations` at plan.rs:97; "phase one computes every byte before any write").
- `src/error.rs:113-122` — `tier_unimplemented` message today: `collection "{name}" cannot be declared: only the glossary collection is implemented, so this release refuses every other collection`. Label `kaya.publications.tier_unimplemented` (vocabulary.rs:67). Must grow to name the implemented set.
- `src/publication/vocabulary.rs:32-41` — `COLLECTIONS: [&str; 9]` closed vocabulary: glossary, example-gallery, api-catalogue, capability-catalogue, compatibility-matrix, feature-index, knowledge-index, related-projects, topic-map. Implemented five = glossary, topic-map, knowledge-index, example-gallery, api-catalogue; refused four = capability-catalogue, compatibility-matrix, feature-index, related-projects (design D5).
- `src/publication/decls.rs:66` — `DEFAULT_COLLECTIONS: [&str; 1] = ["glossary"]` → grows to the five implemented kinds.

## Omission machinery (product-only today)

- `src/publication/render.rs:24-48` `compose_products` — no populated backing doc ⇒ `Omitted { tier: Tier::Product.as_str(), name, reason: REASON_NO_BACKING_DOCUMENT }` (vocabulary.rs:78, `"no-backing-document"`); sorted by name.
- `src/publication/build.rs:23-31` `Omitted { tier: &'static str, name: String, reason: &'static str }`; `Built.omitted` at build.rs:51. CLI envelope `data.omitted` rows `{"tier","name","reason"}` at main.rs:1204-1211 (build) and 1477-1483 (publish).
- `plan.rs` — `Plan { pages, removals, export, export_path, export_previous, omitted }` (plan.rs:56-81); `plan_removals(&omitted, out, read_text)` plans strip/unlink of an omitted publication's page. Collections never omitted today; export rows hardcoded `{"emitted": true, "name"}` (render.rs:195-204 in `export_json`).

## Renderers and ownership

- `render.rs:74-104` `render_pages` — products → packages → `render_collection_page(collection)` per surviving collection; results sorted by name.
- `render.rs:106-129` `render_collection_page` — `debug_assert_eq!(collection.name, "glossary")`; glossary composes `EntityType::ALL`, `ENTITY_TYPE_SET`, `Relationship::ALL`, `DOCUMENT_KINDS` (compile-time only).
- `render.rs:131-161` `render_product_page` — embeds `docs.0.get(kind)` body when product is `owners[kind]` (first emitted holder, `assign_owners` render.rs:66-82: declaration-ordered first insert wins), else `See [{holder}]({holder}.md#{anchor}).`; anchor = `heading_anchor(kind)` (render.rs:285-297, `orientation.research-map` → `orientationresearch-map`).
- `export_json` (render.rs:184-241): product rows `{"name","question","emitted","documents":[{kind,event_ids}]}`; package rows `{"name","members":[{name,emitted}]}`; collection rows the hardcoded pair above. `export.rs` test pins collection row keys exactly `["emitted","name"]`.

## Sources the renderers pin

- `source.rs:64-77` `read_documents` — fixed list index.md, overview.md, understanding.md, reproduction-package.md, handoff.md + sorted `experiments/*.md`. **`technical-reference.md` absent** (producer const `TECHNICAL_REFERENCE_FILE` reuse.rs:47). Must join; `verification/mod.rs:64-70` `RESEARCH_READ_ORDER: [&str; 5]` is the identical list feeding check — both change together (tasks 1.14; design D4).
- `source.rs:146-167` `collect_documents` — only `DOCUMENT_KINDS` blocks, non-empty, first-populated-wins, duplicate-block refusal; `DocumentRef { kind, body, event_ids }`.
- `source.rs:184-196` `node_ids` — tokens starting `kaya/` (split on non-`[alnum / - _ . @]`), sorted deduped. Id format: `kaya/<TYPE>/rest` — TYPE segment = `EntityType` variant name (model.rs:100-102 "used as the `TYPE` segment of a node id"). `@` kept inside token (Source refs `kaya/Source/path@commit`).
- `model.rs:35-36` `ENTITY_TYPE_SET` string; `model.rs:78-95` `EntityType::ALL` (13, vocabulary order); `Relationship::ALL` ten.

## Pinned block text shapes

- `orientation.research-map` (index.md; `RESEARCH_MAP_BLOCK` index.rs:26): `### Relationships` rows `- <from> -[<rel>]-> <to>` sorted/deduped (index.rs:226-247); `### Isolated` rows `- <id> — <EntityType> — <title-or-key>` (index.rs:250-277, skips Artifact/Run/Source); `### Artifacts` `- <id> — <label>`; `### Runs` `- <id> — passed N, failed N, ignored N`; `### Sources` `- <id> — path@commit`; trailing `_N nodes from E events._`.
- `exploration.experiment-record` (`RECORD_BLOCK` report.rs:38; `render_record` report.rs:741-822): sections `### Runs`, `### Observations`, `### Findings`, `### Interpretations`; run/observation/finding ids as `- <id> — <title>` bullets (`id_lines`); **no artifacts section** — the gallery row carries exactly the artifact refs the record names (spec: "the run, artifacts, and observations the record names"; collection_source_bound).
- `reuse-handoff.technical-reference` (reuse.rs:392-396 `render_flat(&inputs.specs, "Specs")`): rows `- <node id> — <spec id>`. Entry ids live in the separate `reuse-handoff.knowledge-brief` block (reuse.rs:398) — never a catalogue source.

## Write path

- `build.rs:63-103` `build` → `plan()` → `write_pages` (atomic per changed page + export) → `apply_removals` (plan.rs removals). Publication pages only ever written by `cmd_publish` (main.rs:1448-1454).
- `kaya check` replays `plan()` with no-op readers — the expected tree; read-list parity keeps both doors identical.

## Test pins to flip (task 1.16)

- `tests/publications/build.rs:252-282` — `non_glossary_collection_still_refused_as_unimplemented` declares **topic-map** (now implemented) → narrows to a refused kind (`feature-index`).
- `tests/publications/build.rs:226-248` — `glossary_declaration_renders_a_collection_page` (page `glossary.md`, block `publication.collection.glossary`) stays.
- `tests/publications/export.rs:342-376` — `export_lists_collection_rows` pins `["emitted","name"]` → grows omitted rows with `reason`.
- `tests/cli_pipeline.rs` — pages list at 2044-2052, 2424-2438, 2490-2504 (and 2645+ variant); export collections rows 2149-2160; block lists 2244-2253 + 2329-2345; omission rows 2035-2040 (also 2511-2515, 2665-2669, 2906-2915). Dogfood gains `topic-map.md` + `knowledge-index.md` (render-now, rows exist) and two collection omission rows (example-gallery, api-catalogue).

## Risks

- Fact-once: `architecture-design` owns `orientation.research-map` in dogfood today (alphabetically first emitted among admittees) — topic-map cross-references it; the no-owner embed path is the test fixture's.
- Omission fatigue: two collection omission rows on every dogfood build until dogfooding tickets (kaya-34g, kaya-quh) land — accepted by design D5/A4.
- plan_removals must strip collection pages too when a previously emitted collection turns omitted (dogfood never regresses, but the removal loop takes `&[Omitted]` — appended collection rows must flow; tier string already parameterized in `Omitted`).
