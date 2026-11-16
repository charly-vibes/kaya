---
tags: [pipeline-run:tdd-ro5u-2026-11-16-kaya-lb2-scaffold, pipeline-step:orient]
---

ORIENT kaya-lb2: crate scaffold with kaya init/status. Acceptance: init creates .kaya/ledger.jsonl idempotently; status exits 0 on empty ledger with digest sha256(empty); labeled error without init. Specs: kaya-event/ledger are later tickets; envelope independence seam. Files: Cargo.toml, src/main.rs, tests/cli.rs.
