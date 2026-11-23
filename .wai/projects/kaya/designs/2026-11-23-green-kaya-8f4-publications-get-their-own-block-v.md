---
tags: [pipeline-run:tdd-ro5u-2026-11-23-kaya-8f4-publication-block-vocabulary, pipeline-step:green]
---

GREEN kaya-8f4: publications get their own block vocabulary and validator, applied where publication pages are planned.

DECISION: the ticket offers 'publications get their own block vocabulary and validator' or 'a documented exemption'. The exemption would bless a state where a publication page carries a stale, misspelled, or foreign managed block and no gate notices: reproduced during orientation, 'kaya publish' exited 0 leaving docs/public/overview.md with BOTH a misspelled 'publication.produkt.reference' block AND a fresh 'publication.product.reference' block, and 'kaya check' reported drift [] for it. A vocabulary that is only described is not enforced. So: publications get their own block vocabulary and their own validator, and the documents capability's vocabulary gate keeps governing research documents - the two namespaces are different in shape (three segments tier.name versus two segments family.kind) and in ownership, and neither validator reads the other's pages.

IMPLEMENTATION:
- src/publication/mod.rs: is_publication_block(name) decides whether a managed block name is inside the publication vocabulary - publication.<tier>.<name>, with the tier one of the three tier names and the name a member of that tier's closed vocabulary (resolved through resolve_name, so the tier segment must agree with the tier the name resolves to). validate_page_blocks(file, document) scans a page's managed blocks and refuses the first name outside the vocabulary with PublicationError::block_vocabulary - LABEL_BLOCK_VOCABULARY 'kaya.publications.block_vocabulary' naming the block and the page.
- The validation runs where a page is planned (for the page's existing bytes, before replace_generated) and where an omitted product's page is removed (before the strip). The publication layer computes the whole plan before writing anything, so a refusal leaves the tree byte-untouched - the same fail-closed discipline the research side applies.
- No new vocabulary list: the publication vocabulary is derived from PRODUCTS, COLLECTIONS, PACKAGES which the layer already owns. The eleven document kinds and their vocabulary are untouched, and DOCUMENT_KINDS (the layer's private filter copy) is unchanged.

Verification after the change: cargo test --test publications 30 passed 0 failed (27 pre-existing plus the three new); cargo test 379 passed 0 failed across 32 suites; clippy clean; fmt clean.
