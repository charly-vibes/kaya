---
reviews: 2026-11-24-ro5u-review-kaya-g41-findings.md
verdict: pass
tags: [pipeline-run:tdd-ro5u-2026-11-24-kaya-g41-pretender-role-thresholds, pipeline-step:ro5u-review]
---

RO5U kaya-g41 review (reviewer, CONVERGED stage 4, verdict NEEDS_REVISION, all findings empirically reproduced on pretender 0.10.0): P0/P1: 0; P2: 2 (both fixed); P3: 2 (both fixed).
[P2-1][FIXED] [thresholds.script] cyclomatic_max/abc_max/file_lines_max silently ignored by 0.10 (probe: cyc 2 / abc 20 / lines 10 all passed) — role sections govern only some metrics; scripts actually governed by global ceilings (cyc 25, abc 150, file_lines 3500). Dead keys removed, limitation documented in the section.
[P2-2][FIXED] [thresholds.test] file_lines_max silently ignored (probe 100 passed with 3222-line file) — a placebo that would break silently when the app ceiling tightens. Removed with note.
[P3-1][FIXED] Comment misattributions: main.rs is 2383 lines (not 3222 — that is tests/cli_pipeline.rs); app cyclomatic max 23 belongs to evaluate() in src/impact.rs (not cmd_import, cyc 2). Corrected.
[P3-2][FIXED] params_max keys inert under 0.10 (metric measures 0 for all 1373 units) — removed from all roles.
Also: AGENTS.md managed-block regeneration attempted by pretender init was reverted (unrequested churn); main.rs split debt ticketed as kaya-g9u so the header's tighten-then pointer is real.
Verified-clean: every retained threshold exceeds its measured per-role maximum (1373 units measured); 3 baselined min_assertions entries are genuinely asserting tests via helper macros (bodies verified); gate stable — two consecutive runs 3 known / 0 new, hook exit 0; no adopt needed.

