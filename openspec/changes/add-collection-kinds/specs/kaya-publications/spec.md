## MODIFIED Requirements

### Requirement: Justified emission

A product or collection SHALL be emitted only when at least one admissible
backing document exists — for a product, a document of an admitted kind; for
a collection, the rows its composition contract binds. A publication without
backing information SHALL be omitted and reported with reason
`no-backing-document`, and no publication tree SHALL contain a stub page for
it. A collection kind with no composition contract SHALL be refused with
`kaya.publications.tier_unimplemented` naming the kind and the implemented
collection kinds, before anything is written, leaving the tree
byte-identical. The refusal and the omission are both recorded decisions:
neither is ever a stub renderer or a page of empty sections.

#### Scenario: No stub page

- **WHEN** no document is admissible for product `tutorial`
- **THEN** no tutorial page is written and the build summary reports
  `tutorial` as omitted with reason `no-backing-document`

#### Scenario: Omission visible in the package

- **WHEN** a package lists an omitted product
- **THEN** the package's navigation names it as unavailable rather than
  linking to a page that does not exist

#### Scenario: Collection without backing rows omitted

- **WHEN** `example-gallery` is declared and no experiment dossier carries
  an `exploration.experiment-record` block
- **THEN** no gallery page is written and the build summary reports
  `example-gallery` as omitted with reason `no-backing-document`

#### Scenario: Collection without a composition contract refused

- **WHEN** a tree declares `feature-index`, which has no admissible source
  and no producer-backed path
- **THEN** the build refuses with `kaya.publications.tier_unimplemented`
  naming `feature-index` and the implemented collection kinds, and the tree
  is left byte-identical

### Requirement: Property coverage mirror

Every property row in the layer sidecar SHALL be verified by exactly one dedicated scenario.

#### Scenario: tier_resolved

- **WHEN** a publication declares a tier and a name
- **THEN** the pair resolves to exactly one tier vocabulary entry or is refused
- **VERIFIES** [[kaya.publications.tier_resolved]]

#### Scenario: fact_once

- **WHEN** a package and its products are built from one document graph
- **THEN** each fact body appears once and elsewhere only as a reference
- **VERIFIES** [[kaya.publications.fact_once]]

#### Scenario: stub_free

- **WHEN** a product has no admissible backing document
- **THEN** no page is written and the omission is reported
- **VERIFIES** [[kaya.publications.stub_free]]

#### Scenario: inadmissible_refused

- **WHEN** a product includes a document kind outside its admission row
- **THEN** the build refuses with a label naming product, kind, and row
- **VERIFIES** [[kaya.publications.inadmissible_refused]]

#### Scenario: format_attribution

- **WHEN** two delivery formats are projected from the same source
- **THEN** both trace every fact to the same event ids
- **VERIFIES** [[kaya.publications.format_attribution]]

#### Scenario: page_block_owned

- **WHEN** a publication page carries a managed block whose name is outside
  the publication block vocabulary
- **THEN** the build is refused naming the block and the page, with the page
  byte-identical
- **VERIFIES** [[kaya.publications.page_block_owned]]

#### Scenario: collection_source_bound

- **WHEN** a collection page is rendered
- **THEN** its body names only rows its pinned composition source carries
- **VERIFIES** [[kaya.publications.collection_source_bound]]

## ADDED Requirements

### Requirement: Collection composition contracts

Each implemented collection SHALL compose its page from exactly its pinned
source — for the document-sourced collections, exactly the pinned block
among the compiled documents the publication layer reads; for the
glossary, exactly its pinned compile-time vocabularies — never a ledger
query. A collection declaration SHALL carry no fields, its page SHALL
name no reader question, and its managed block SHALL be
`publication.collection.<name>`.

#### Scenario: every_declared_collection_is_implemented_or_refused

- **WHEN** the collection vocabulary is inspected
- **THEN** exactly glossary, topic-map, knowledge-index, example-gallery,
  and api-catalogue render pages, and declaring any other collection kind is
  a labeled refusal

#### Scenario: collection_declaring_fields_refused

- **WHEN** a tree declares a collection carrying `includes` or `members`
- **THEN** the build refuses naming the collection and the stray field, and
  no page is written

### Requirement: Topic-map composes the research map under fact-once

The topic-map SHALL compose its rows from the relationship statements of
`orientation.research-map` — one row per topic id with its entity type and
the relationship types it participates in — and its edge statements SHALL
obey fact-once: cross-referenced to the page that embeds the research-map
body, embedded by the topic-map only when no page in the tree does.

#### Scenario: topic_map_renders_from_the_research_map_block

- **WHEN** the research map carries three relationship statements and the
  tree is published
- **THEN** the topic-map page lists one row per distinct topic id with its
  entity type and relationship types, under the block
  `publication.collection.topic-map`

#### Scenario: topic_map_cross_references_embedded_edges

- **WHEN** the `reference` product embeds the research-map body in the same
  tree
- **THEN** the topic-map page references that page for the relationship
  statements and embeds none of them

#### Scenario: topic_map_embeds_edges_when_no_page_does

- **WHEN** no emitted page composes `orientation.research-map`
- **THEN** the topic-map page embeds the relationship statements and is
  their only rendering in the tree

### Requirement: Gallery and catalogue compose their producer blocks

`example-gallery` SHALL compose one row per recorded experiment from
`exploration.experiment-record` blocks, carrying the run, artifacts, and
observations the record names. `api-catalogue` SHALL compose one row per spec id from
`reuse-handoff.technical-reference`, carrying the id and its resolvable
reference; entry ids are the knowledge-brief block's rows and are not
catalogue rows.

#### Scenario: example_gallery_renders_from_experiment_records

- **WHEN** an experiment dossier carries an `exploration.experiment-record`
  block naming a run and artifacts
- **THEN** the gallery page lists one row for that experiment carrying the
  run and artifact references the record names

#### Scenario: api_catalogue_renders_from_the_technical_reference

- **WHEN** the technical reference carries spec rows
- **THEN** the catalogue page lists one row per spec id with its resolvable
  reference, composed from the block the reuse projection wrote

### Requirement: Knowledge-index composes the per-type census

`knowledge-index` SHALL compose a per-type census of the node ids the
compiled blocks carry, headed by the closed thirteen-entity set with a
count per type — never a ledger query.

#### Scenario: knowledge_index_renders_the_per_type_census

- **WHEN** the compiled blocks carry node ids of at least one entity type
- **THEN** the index page carries a section per closed entity type with a
  count, lists the node ids the tree names, and reports none for the types
  with no rows; node ids whose type segment is outside the closed entity
  set are not census rows
