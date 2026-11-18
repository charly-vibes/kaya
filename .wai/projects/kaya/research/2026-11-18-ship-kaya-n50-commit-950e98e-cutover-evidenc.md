---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-n50-overview-cutover, pipeline-step:ship-close]
---

SHIP kaya-n50 — commit 950e98e (cutover) + evidence commit

Ticket kaya-n50 closed in bd; .beads/issues.jsonl exported in the feature commit. No push performed.

Verification at ship: kaya publish twice recomputed [] with byte-identical overview bytes; kaya build recomputed []; mdbook build green; just ci green; cargo test 24 targets; normalized duplication sweep 0; all relative links resolve.

Known advisory: pre-commit pretender gate reports the same 50 pre-existing style-threshold violations (0 new); scripts/sync-specs.py stays green.

Next: kaya-rdr — prepare the crates.io publish wiring and the exact authorization steps for the maintainer.

