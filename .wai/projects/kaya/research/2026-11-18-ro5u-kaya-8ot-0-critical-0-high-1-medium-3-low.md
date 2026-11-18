---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-8ot-kaya-check-facts, pipeline-step:ro5u-review]
---

RO5U kaya-8ot: 0 Critical, 0 High, 1 Medium, 3 Low. Verdict incorrect pending one frontier fix.
MEDIUM 1: last_capture_commit selects the newest capture event then reads data.sha; an event without a string sha yields null despite older named commits, and raw timestamp string ordering misorders fractional seconds. Fix: filter events whose data.sha is a string, select by parsed RFC3339 timestamp then id.
LOW 2: test-runner classification arm untested; import a libtest report in one fixture and assert >=1.
LOW 3: frontier selection test imports one commit and mirrors the implementation key; commit twice with distinct author dates, import both, assert the later sha.
LOW 4: facts-without-a-second-parser is bound to a seed-only test; bind a multi-source test.

