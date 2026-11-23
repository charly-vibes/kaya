---
tags: [pipeline-run:tdd-ro5u-2026-11-23-kaya-8f4-publication-block-vocabulary, pipeline-step:red]
---

RED kaya-8f4: command=cargo test --test publications -> 27 passed, 3 failed.

Expected failure, missing behavior and not setup breakage: all three new tests fail because the publication build accepts a page whose managed block name is outside the publication vocabulary. The failures show the build SUCCEEDING where it must refuse:

1. publication_page_with_a_misspelled_block_is_refused - a page carrying 'publication.produkt.reference' builds: Built { pages: [Page { name: 'reference', ... }], omitted: [], summary: Summary { recomputed: ['publication.product.reference', 'publication.json'], ... } }. The stale misspelled block is left in the page beside the fresh one.
2. publication_page_with_a_document_block_is_refused - a page carrying the research block 'orientation.research-map' builds the same way.
3. publication_page_with_a_name_outside_its_tier_is_refused - a page carrying 'publication.product.glossary' (a collection name under the product tier) builds the same way.

No test was broken by the addition: the 27 pre-existing publication tests still pass, and the failure messages come from the new assertions themselves.

The three cases are the observable forms of the defect found during orientation: the publication layer replaces only the block it names, so any other generated block in a publication page survives a build silently - and 'kaya check' reports no drift for it either (verified by a live fixture during orientation).
