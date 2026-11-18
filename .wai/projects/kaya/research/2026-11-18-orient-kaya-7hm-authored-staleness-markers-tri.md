---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-7hm-authored-staleness, pipeline-step:orient]
---

ORIENT kaya-7hm: authored staleness — markers, trigger, acknowledgment (P2, complexity l, lane afk). Files: src/main.rs, src/projection/report.rs, src/event.rs. Depends kaya-585/ntv (closed).

Spec: clarify-authored-staleness — 13 scenarios: citation markers (3), staleness trigger scope (2), acknowledgment events (3), acknowledgment command (2), property mirror (3).
Testable:
1. `kaya:cites <ids>` markers inside authored blocks resolve against ledger ids; unknown id refused by build and check with a labeled error naming document, marker and id. Markers outside authored blocks are inert and byte-preserved.
2. A cited id ages a block only when a later event supersedes it and changes effective data; appended non-superseding events never age a block; all changed ids are named.
3. `verification.acknowledged` (subject Document, data {block, events}) clears staleness only for that (document, block) and those ids; partial coverage leaves the block stale; it never edits authored bytes and never enters the research graph or generated blocks.
4. `kaya acknowledge <document> --block <name> --events <id>... --actor <name> [--actor-type human]` appends exactly one event (duplicate is a no-op), refuses tool actors/unknown documents/blocks with labeled errors appending nothing; ids no longer cited are accepted inert.
Labels: kaya.projection.citation_marker for unknown citations; kaya.acknowledge.* for command refusals. Envelope kind `acknowledge`.

Meter: cargo test && ah check --changes clarify-authored-staleness --run-tests → all scenarios execute, 0 no-toml. Anti-goals: ack never edits authored bytes; tool actors refused; verification events never enter the graph.

