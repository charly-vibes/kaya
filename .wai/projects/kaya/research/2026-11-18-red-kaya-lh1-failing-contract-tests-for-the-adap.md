---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-lh1-reuse-families-technical-reference-knowledge-brief, pipeline-step:red]
---

RED kaya-lh1 — failing contract tests for the adapter-backed reuse kinds

Narrow command: `cargo test --test reuse_adapters`
Result: compile failure, exactly the missing behavior (no setup breakage):
- E0432: `TECHNICAL_REFERENCE_BLOCK` and `KNOWLEDGE_BRIEF_BLOCK` do not exist in `projection::reuse`.
- E0609: `Inputs` has no `specs` or `entries` field (available: revisions, commands, handoffs).
Eight tests written first in tests/reuse_adapters.rs: input selection (spec/entry/blank/non-string refusal), technical-reference rows, knowledge-brief rows naming every entry id, ledger byte-identity plus markdown-only writes, empty-kind omission once each with no directory, stale block removal, canonical document/block order, authored-byte preservation.

Spec contract declared for the scenario this ticket owns: `.espectacular/changes/add-document-taxonomy/kaya-documents/brief-adds-no-knowledge.toml`, id `brief-adds-no-knowledge`, cargo selectors `knowledge_brief_names_every_entry_id` and `ledger_gains_no_entity_and_only_markdown_is_written`.
`ah check --changes add-document-taxonomy` moved from 20 structural to 19 structural; no id-mismatch or no-tests-declared finding on the new contract.

Note: `ah scenario new` refuses an existing spec scenario ("scenario ... already exists under requirement ..."), so the contract file was authored directly in the deployed file format; the tool accepts it.
