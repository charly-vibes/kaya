---
tags: [pipeline-run:tdd-ro5u-2026-11-24-kaya-msh-wai-adapter-idempotent-re-import, pipeline-step:red]
---

RED kaya-msh: command=cargo test --test cli_pipeline -- reimport_of_unchanged edited_record_yields. Result: reimport_of_unchanged_tree_appends_nothing ok (meter green today via byte-exact dedupe); edited_record_yields_correction_never_id_reuse FAILED exactly as predicted — import exits 1 at the second import (kaya.ledger.id_reuse refusal on the edited heading), no correction path exists. Failure is the expected missing behavior, not setup breakage. Espectacular bindings appended: replay-dedupe.toml += reimport_of_unchanged_tree_appends_nothing; correction-round-trip.toml += edited_record_yields_correction_never_id_reuse.
