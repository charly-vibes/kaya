---
tags: [pipeline-run:tdd-ro5u-2026-11-24-kaya-g41-pretender-role-thresholds, pipeline-step:quality-ledger]
---

QUALITY LEDGER kaya-g41 — Changed: pretender.toml (new, measured thresholds), pretender-baseline.json (126 -> 3 documented helper-blind-spot entries), beads export. Verified: gate stable across consecutive runs (3 known / 0 new, no adopt); hook check --diff-only exit 0; suite 451/0 (config-only). Review: RO5U CONVERGED; 2 P2 (silently-ignored role keys: script cyc/abc/file_lines, test file_lines) + 2 P3 (comment misattributions, inert params_max) all fixed and probe-verified. Known limitation documented in-file: pretender 0.10 honors only cognitive/function_lines/nesting (+min_assertions) per role; cyc/abc/file_lines come from the global section. Follow-ups: kaya-g9u (split main.rs, then tighten file_lines) filed. Next: none.
