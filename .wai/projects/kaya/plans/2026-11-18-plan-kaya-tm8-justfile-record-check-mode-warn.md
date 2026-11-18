---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-tm8-ci-parity, pipeline-step:plan]
---

PLAN kaya-tm8: justfile `record-check mode="warn"` = record-verify + release-claims + links + record-lag(mode); ci checkout fetch-depth 0; new step runs `just record-check <mode>` (warn on PR, fail on main); .gitattributes pins LF for docs/** and .kaya/**; .gitignore ignores .whisper/. Verify `just record-check warn` exits 0 and `just record-check fail` exits 1 on the current lag; injected staged drift already fails pre-commit.

