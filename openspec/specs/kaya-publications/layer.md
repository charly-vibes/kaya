---
id: kaya.publications.layer
kind: intent
statement: "WHEN a client-facing publication is built, THE kaya.publications capability SHALL classify it as exactly one product, collection or package from the closed vocabularies, compose products from document-family references without duplicating content, emit only products with admissible backing documents, bind each product to its reader question and admission row, and project every delivery format from one source and one source map."
---

# kaya.publications — dual-format layer (sidecar)

## Purpose

The client-facing half of kaya's scope: publications that answer a visitor's
questions rather than mirror the internal record. Twelve reader-facing
products sit over the eleven document kinds; collections group related
information; packages bundle products for a purpose. Keeping the three tiers
apart is what prevents the two failure modes the origin investigation found:
dozens of overlapping templates, and stub pages that exist because a taxonomy
said so rather than because there is information to publish.

## Constraints

| id                        | kind      | expr                                                                                                                                                                                                                          | traces_to |
|---------------------------|-----------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| tier_separated            | invariant | `every publication is exactly one of product, collection or package from the closed vocabularies; packages list products, collections list products or documents; an unknown tier or name is a labeled refusal`                | [[kaya.publications.layer]] |
| composition_no_duplication| invariant | `products compose document-family references and packages reference products; each fact body appears once in a published tree, elsewhere only as a cross-reference`                                                            | [[kaya.publications.layer]] |
| justified_emission        | invariant | `a product is emitted only when at least one admissible backing document exists; otherwise it is omitted and reported, and no stub page is written`                                                                            | [[kaya.publications.layer]] |
| reader_question_bound     | invariant | `each product declares exactly one reader question and admits only the document kinds in its admission row; an inadmissible inclusion is a labeled refusal naming product, kind and row`                                       | [[kaya.publications.layer]] |
| single_source_multi_format| invariant | `every delivery format is projected from the same document source and source map; no format is maintained separately and every fact traces to the same event ids in every format`                                              | [[kaya.publications.layer]] |

## Model

### States
- `selected`
- `composed`
- `emitted`
- `omitted`

### Transitions

| id       | from      | to       | guard                                                                                                                         |
|----------|-----------|----------|-------------------------------------------------------------------------------------------------------------------------------|
| select   | selected  | composed | [[kaya.publications.layer.tier_separated]] ∧ [[kaya.publications.layer.reader_question_bound]]                                  |
| compose  | composed  | emitted  | [[kaya.publications.layer.composition_no_duplication]] ∧ [[kaya.publications.layer.single_source_multi_format]]                  |
| omit     | composed  | omitted  | ¬`backing document exists` ∧ [[kaya.publications.layer.justified_emission]]                                                     |

## Properties

| id                         | kind | derives_from                                          | generator                                  | predicate                                                   |
|----------------------------|------|-------------------------------------------------------|--------------------------------------------|--------------------------------------------------------------|
| tier_resolved              | unit | [[kaya.publications.layer.tier_separated]]             | `publication_with_unknown_tier()`          | `one tier resolved ∨ labeled refusal`                        |
| fact_once                  | unit | [[kaya.publications.layer.composition_no_duplication]] | `package_with_shared_fact()`               | `body occurrences == 1 ∧ others are references`               |
| stub_free                  | unit | [[kaya.publications.layer.justified_emission]]         | `product_without_backing_document()`       | `no page written ∧ omission reported`                        |
| inadmissible_refused       | unit | [[kaya.publications.layer.reader_question_bound]]      | `product_with_inadmissible_kind()`         | `labeled refusal naming product ∧ kind ∧ row`                 |
| format_attribution         | unit | [[kaya.publications.layer.single_source_multi_format]] | `two_formats_from_one_source()`            | `∀ fact: same event ids in both formats`                      |

## Requirements
### Requirement: Layer sidecar contract

This sidecar is the dual-format layer for the `kaya.publications` capability and SHALL be deployed verbatim by the archive companion; the plain delta in `spec.md` beside it is its openspec face.

#### Scenario: Archive deploys the layer

- **WHEN** the change `add-document-taxonomy` is archived after its scenarios
  have executing contract tests
- **THEN** the capability deploys with this layer attached fail-closed