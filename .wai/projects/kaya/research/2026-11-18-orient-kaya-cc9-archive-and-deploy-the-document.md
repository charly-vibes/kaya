---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-cc9-archive-and-deploy-document-taxonomy, pipeline-step:orient]
---

ORIENT kaya-cc9 — archive and deploy the document taxonomy

Ticket: kaya-cc9, P3, complexity:s, lane:afk, base_commit df816560, files ["docs/src/SUMMARY.md"]. Dependency kaya-lh1 is now closed, so the ticket is READY and claimed.

Acceptance restated as testable:
1. Every scenario in the two deltas (kaya.documents 17, kaya.publications 16) has a contract file whose cargo selector executes a real test: `ah check --changes add-document-taxonomy --run-tests` reports 0 structural.
2. `openspec archive add-document-taxonomy` deploys both layers into openspec/specs and leaves the change archived.
3. `spk lint openspec/specs` is clean on the deployed layers.
4. The book Design specs section names the deployed layers instead of pointing at change deltas.
5. Archive happens only now, after every taxonomy scenario has an executing test.

Current evidence: `ah check --changes add-document-taxonomy` → 0 passed, 19 structural, all no-toml (was 20 before lh1 bound brief-adds-no-knowledge). The 19 bind to existing passing tests:
kaya-documents/
- block-name-outside-the-vocabulary-refused → tests/documents.rs legacy_block_name_is_refused_before_writing
- isolated-entity-still-listed → tests/research_map.rs isolated_entity_is_listed_once_and_no_edge_is_invented
- missing-runs-leave-no-section → tests/report.rs record_kind_without_runs_is_omitted_and_reported
- omission-reported-once → tests/cli_pipeline.rs omitted_kinds_reported_once_in_build_envelope
- record-not-asserted-in-the-specification → tests/documents.rs specification_without_runs_renders_and_record_is_omitted
- reproduction-package-points-at-a-revision → tests/reuse.rs inputs_selects_references_from_the_graph
- specification-not-inferred-from-a-run → tests/documents.rs specification_kind_is_omitted_for_a_record_only_experiment
- typed-edge-rendered → tests/research_map.rs every_declared_edge_renders_once_with_both_endpoints_and_type
kaya-publications/
- every-product-declares-a-question → tests/publications.rs admission_table_is_pinned_for_every_product
- inadmissible-inclusion-refused → tests/publications.rs inadmissible_inclusion_is_refused
- markdown-and-export-agree → tests/publications.rs markdown_and_export_share_one_source_and_rebuild_identically
- no-separately-maintained-format → tests/publications_export.rs repeat_publish_is_byte_identical
- no-stub-page → tests/publications.rs product_without_backing_document_is_omitted_without_a_page
- omission-visible-in-the-package → tests/cli_pipeline.rs build_writes_overview_and_publish_uses_it
- package-cannot-list-a-family → tests/publications.rs package_cannot_list_a_family
- package-embeds-no-product-body → tests/publications.rs project_site_over_overview_and_reference
- product-and-package-composed → tests/publications.rs project_site_over_overview_and_reference
- shared-fact-rendered-once → tests/publications.rs shared_fact_is_rendered_once_with_a_cross_reference
- unknown-product-refused → tests/publications.rs tier_resolved_for_every_name_and_unknown_refused

Contract file format (deployed convention): tool_version 0.12.3, id = scenario slug, archetype PF, status active, superseded_by "", authored_with 0.12.3, [tests] cargo = [{flags = "<test fn>"}]. `ah scenario new` refuses existing spec scenarios, so files are authored directly as lh1 did.

Risks found before planning:
- `openspec validate --changes --strict` currently fails add-document-taxonomy on two long-requirement WARNINGs (kaya-publications "Publication tiers" and "Reader-question binding" exceed 500 chars). Archive validation will likely surface them; fixing means moving examples into scenarios or splitting requirements — a spec edit inside this change, so cc9 owns it.
- Change tasks 3.10 (n50), 3.11 (rdr), 3.12 (aaz) stay unchecked because they are separate open tickets; archive must not pretend they are done. Their checklist items live outside the archive prerequisites.
- `openspec archive` moves delta specs into openspec/specs and updates the change directory; deployment is destructive, so the commit must include both sides.
