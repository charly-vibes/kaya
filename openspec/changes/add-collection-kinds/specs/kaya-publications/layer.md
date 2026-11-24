---

id: kaya.publications.layer

kind: intent

statement: "WHEN a client-facing publication is built, THE kaya.publications capability SHALL classify it as exactly one product, collection or package from the closed vocabularies, compose products from document-family references without duplicating content, emit only publications with admissible backing documents, refuse any collection kind without a composition contract, bind each product to its reader question and admission row, compose each implemented collection from its pinned compiled source, and project every delivery format from one source and one source map."

---



# kaya.publications — dual-format layer (sidecar)



## Purpose



The client-facing half of kaya's scope: publications that answer a visitor's

questions rather than mirror the internal record. Twelve reader-facing

products sit over the eleven document kinds; collections group related

information — five implemented kinds compose from their pinned compiled

sources, the refused rest are labeled refusals; packages bundle products for

a purpose. Keeping the three tiers apart is what prevents the two failure

modes the origin investigation found: dozens of overlapping templates, and

stub pages that exist because a taxonomy said so rather than because there

is information to publish.



## Constraints



| id                        | kind      | expr                                                                                                                                                                                                                          | traces_to |

|---------------------------|-----------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------|

| tier_separated            | invariant | `every publication is exactly one of product, collection or package from the closed vocabularies; packages list products, collections list products or documents; an unknown tier or name is a labeled refusal`                | [[kaya.publications.layer]] |

| composition_no_duplication| invariant | `products compose document-family references and packages reference products; each fact body appears once in a published tree, elsewhere only as a cross-reference`                                                            | [[kaya.publications.layer]] |

| justified_emission        | invariant | `a product or collection is emitted only when at least one admissible backing document exists; otherwise it is omitted and reported with reason no-backing-document; a collection kind without a composition contract is a labeled refusal naming the kind and the implemented kinds; no stub page is written` | [[kaya.publications.layer]] |

| collection_contracts      | invariant | `each implemented collection composes its page from exactly its pinned source, never a ledger query: the document-sourced collections from exactly their pinned block among the compiled documents the publication layer reads (topic-map from the research-map relationship statements under fact-once, example-gallery from experiment-record blocks, api-catalogue from the technical-reference's spec-id rows, knowledge-index from the per-type census of the node ids the blocks carry) and the glossary from its pinned compile-time vocabularies; a collection declaration carries no fields and its page no reader question` | [[kaya.publications.layer]] |

| reader_question_bound     | invariant | `each product declares exactly one reader question and admits only the document kinds in its admission row; an inadmissible inclusion is a labeled refusal naming product, kind and row`                                       | [[kaya.publications.layer]] |

| single_source_multi_format| invariant | `every delivery format is projected from the same document source and source map; no format is maintained separately and every fact traces to the same event ids in every format`                                              | [[kaya.publications.layer]] |

| block_vocabulary          | invariant | `a publication page carries exactly one managed block, publication.<tier>.<name>, with the tier and the name from that tier's closed vocabulary; a managed block outside it is a labeled refusal naming the block and the page, before any page is written` | [[kaya.publications.layer]] |



## Model



### States

- `selected`

- `composed`

- `emitted`

- `omitted`



### Transitions



| id       | from      | to       | guard                                                                                                                         |

|----------|-----------|----------|-------------------------------------------------------------------------------------------------------------------------------|

| select   | selected  | composed | [[kaya.publications.layer.tier_separated]] ∧ [[kaya.publications.layer.reader_question_bound]] ∧ [[kaya.publications.layer.collection_contracts]] |

| compose  | composed  | emitted  | [[kaya.publications.layer.composition_no_duplication]] ∧ [[kaya.publications.layer.single_source_multi_format]]                  |

| omit     | composed  | omitted  | ¬`backing document exists` ∧ [[kaya.publications.layer.justified_emission]]                                                     |



## Properties



| id                         | kind | derives_from                                          | generator                                  | predicate                                                   |

|----------------------------|------|-------------------------------------------------------|--------------------------------------------|--------------------------------------------------------------|

| tier_resolved              | unit | [[kaya.publications.layer.tier_separated]]             | `publication_with_unknown_tier()`          | `one tier resolved ∨ labeled refusal`                        |

| fact_once                  | unit | [[kaya.publications.layer.composition_no_duplication]] | `package_with_shared_fact()`               | `body occurrences == 1 ∧ others are references`               |

| stub_free                  | unit | [[kaya.publications.layer.justified_emission]]         | `product_without_backing_document()`       | `no page written ∧ omission reported`                        |

| collection_source_bound    | unit | [[kaya.publications.layer.collection_contracts]]       | `collection_page_rendered()`               | `body names only rows its pinned source carries`              |

| inadmissible_refused       | unit | [[kaya.publications.layer.reader_question_bound]]      | `product_with_inadmissible_kind()`         | `labeled refusal naming product ∧ kind ∧ row`                 |

| format_attribution         | unit | [[kaya.publications.layer.single_source_multi_format]] | `two_formats_from_one_source()`            | `∀ fact: same event ids in both formats`                      |

| page_block_owned           | unit | [[kaya.publications.layer.block_vocabulary]]           | `publication_page_with_foreign_block()`    | `labeled refusal naming the block ∧ the page ∧ bytes intact`  |



## Requirements

### Requirement: Layer sidecar contract



This sidecar is the dual-format layer for the `kaya.publications` capability and SHALL be deployed verbatim by the archive companion; the plain delta in `spec.md` beside it is its openspec face.



#### Scenario: Archive deploys the layer



- **WHEN** the change `add-collection-kinds` is archived after its scenarios

  have executing contract tests

- **THEN** the capability deploys with this layer attached fail-closed
