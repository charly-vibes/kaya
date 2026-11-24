---
tags: [pipeline-run:tdd-ro5u-2026-11-24-kaya-4gv-finding-to-source-relationship-delta, pipeline-step:orient]
---

ORIENT kaya-4gv: MVP-2 hook verified (agent-session-capture proposal.md:137-161 verbatim; arms now at src/model.rs:195-212): no Finding->Source edge expressible; cites is Decision->Source; requires_exact_pair covers 5 structural edges; acyclic subgraph is influences|derives only. Change authored by design subagent mirroring the 2026-11-21-add-decision-evidence-link template: relationship grounds (Finding -> Source, exact pair, NOT acyclic — a grounds cycle is unconstructible since Source admits no outgoing grammar edge), name argued against cites/observes/derives. Deltas: kaya-model layer.md (statement, relationship_set_closed + direction_fixed exprs, 4 coverage rows with generator fns), spec.md (Typed relationship closure + Direction discipline + Property coverage mirror MODIFIED), tasks 1.1-1.4 per test, 2.1 code change. openspec validate --strict: [RESULT].
