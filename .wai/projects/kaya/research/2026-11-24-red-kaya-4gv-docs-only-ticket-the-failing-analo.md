---
tags: [pipeline-run:tdd-ro5u-2026-11-24-kaya-4gv-finding-to-source-relationship-delta, pipeline-step:red]
---

RED kaya-4gv: docs-only ticket — the failing analog of a red test is the absent change itself; evidence recorded: openspec validate add-finding-source-link --strict returns 'Change add-finding-source-link is valid' for the authored delta, and the implementation-level red is carried by the consuming AFK ticket (tests/model.rs grounds block does not exist yet — tasks 1.1-1.4 name those tests). cargo suite green pre-change: no regression surface.
