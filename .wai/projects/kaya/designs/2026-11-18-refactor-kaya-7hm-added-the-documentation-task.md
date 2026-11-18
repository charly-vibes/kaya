---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-7hm-authored-staleness, pipeline-step:refactor]
---

REFACTOR kaya-7hm: added the documentation task — README documents `kaya check`, `check --index`, `kaya acknowledge`, citation markers and ack semantics; docs/research/index.md's authored block now cites a ledger event with `<!-- kaya:cites evt-b64818… -->`, exercising the mechanism in the dogfood record. `kaya build` leaves authored bytes untouched; `kaya check` stays clean. spk lint on the deployed projection layer, openspec validate --strict green.

