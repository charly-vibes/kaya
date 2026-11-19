---
reviews: 2026-11-19-green-g11-status-counts-by-structural-recognition.md
verdict: pass
tags: [pipeline-run:tdd-ro5u-2026-11-19-kaya-g11-status-events-count, pipeline-step:ro5u-review]
---

RO5U g11 verdict CRITICAL 0 gate passes (READY). Recognition semantics = documented lenient contract, bounded one-directional divergence, digest authoritative; envelope additive; parity sweep untouched. Findings fixed: F1 bytes.is_empty gate (single blank line reports blank_lines 1); F2 counting moved onto the same streaming read as digest/bytes (removes second read + desync race); cli.rs contract tests restored after the intermediate full-parse counting broke them (schema-only seeds) — recognition is the fix, not weakened tests.
