---
tags: [pipeline-run:tdd-ro5u-2026-11-19-kaya-g11-status-events-count, pipeline-step:green]
---

GREEN g11: status counts by structural recognition (UTF-8 JSON object naming kaya.event/v0), not newlines; blank_lines field added; unterminated complete final line counts; non-empty unparseable lines skipped; digest/bytes/replay_integrity unchanged. Corrective round: cli.rs contract tests restored (minimal schema-only seeds now count), clippy collapsed-if fixed. Review-driven: counting runs on the SAME single streaming read as digest/bytes (removes second read + race), bytes.is_empty gate (single blank line reports blank_lines 1). Gates: full suite 30 suites, cli 9, ah run-tests 150/0, clippy clean.
