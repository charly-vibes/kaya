---
tags: [pipeline-run:tdd-ro5u-2026-11-25-kaya-ynj-research-surfaces-render-information-not-references, pipeline-step:green]
---

RED-VERIFIED kaya-ynj: three tests appended to tests/research_map.rs by subagent (entity_created_event helper + model::build events path). Orchestrator rerun confirms FAILED 9 passed 3 failed, zero compile errors. isolated_decision_renders_its_text_without_a_title: row '- p/Decision/d1 — Decision — d1' proves flat data.text dropped at entity_from_event admission. title_beats_text_when_both_present: honest red - flat title also dropped today, test becomes the precedence pin once admission lands. relationship_rows_carry_endpoint_labels: '- p/Decision/d1 -[supports]-> p/Decision/d2' proves bare-id endpoints. Root causes pinned: admission (model.rs:488-507 attributes-only), label fallback (mod.rs:353 title-only), endpoint labels (index.rs:176-185 no node lookup).
