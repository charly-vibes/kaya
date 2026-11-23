## ADDED Requirements

### Requirement: Publication block vocabulary

A publication page SHALL carry exactly one managed block, named
`publication.<tier>.<name>`, with `<tier>` one of the three closed tier names
and `<name>` a member of that tier's closed vocabulary; a managed block whose
name is outside that vocabulary SHALL be refused with a labeled error naming
the block and the page, before any page is written.

#### Scenario: Page block outside the vocabulary refused

- **WHEN** a publication page carries a managed block whose name is not
  `publication.<tier>.<name>` for that tier's vocabulary
- **THEN** the build is refused with a labeled error naming the block and the
  page, and the page is left byte-identical

#### Scenario: Document block in a publication page refused

- **WHEN** a publication page carries a document-vocabulary block name
- **THEN** the build is refused with a labeled error naming the block

#### Scenario: Tier and name agree

- **WHEN** a publication page carries `publication.<tier>.<name>` and the name
  belongs to another tier's vocabulary
- **THEN** the build is refused with a labeled error naming the block

## MODIFIED Requirements

### Requirement: Property coverage mirror

Every property row in the layer sidecar SHALL be verified by exactly one dedicated scenario.

#### Scenario: tier_resolved

- **WHEN** a publication declares a tier and a name
- **THEN** the pair resolves to exactly one tier vocabulary entry or is refused
- **VERIFIES** [[kaya.publications.layer.tier_resolved]]

#### Scenario: fact_once

- **WHEN** a package and its products are built from one document graph
- **THEN** each fact body appears once and elsewhere only as a reference
- **VERIFIES** [[kaya.publications.layer.fact_once]]

#### Scenario: stub_free

- **WHEN** a product has no admissible backing document
- **THEN** no page is written and the omission is reported
- **VERIFIES** [[kaya.publications.layer.stub_free]]

#### Scenario: inadmissible_refused

- **WHEN** a product includes a document kind outside its admission row
- **THEN** the build refuses with a label naming product, kind, and row
- **VERIFIES** [[kaya.publications.layer.inadmissible_refused]]

#### Scenario: format_attribution

- **WHEN** two delivery formats are projected from the same source
- **THEN** both trace every fact to the same event ids
- **VERIFIES** [[kaya.publications.layer.format_attribution]]

#### Scenario: block_vocabulary

- **WHEN** a publication page carries a managed block whose name is outside
  the publication block vocabulary
- **THEN** the build is refused naming the block and the page, with the page
  byte-identical
- **VERIFIES** [[kaya.publications.layer.block_vocabulary]]
