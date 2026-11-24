---
tags: [pipeline-run:tdd-ro5u-2026-11-24-kaya-rp6-impact-gate, pipeline-step:refactor]
---

REFACTOR kaya-rp6: tidy pass, behavior unchanged. Waiver precedence branch flattened to one if-let; rule-target insertion via entry().or_insert_with; clippy clean (from_ref for borrowed single-element slices); cargo fmt applied. Named the staleness/IR naming seam in ir_block_name's doc instead of unifying record ids — unification would change the deployed IR record-id contract (document_ir tests + kaya-projection delta pin <path>#authored), which is behavior, not tidy. Full suite green after refactor: 34 test binaries, 0 failures, 0 clippy warnings.
