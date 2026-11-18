---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-tm8-ci-parity, pipeline-step:orient]
---

ORIENT kaya-tm8: CI parity and repo hygiene (P1, s, afk). Files: .github/workflows/ci.yml, .gitattributes, .gitignore, justfile.
Acceptance: CI executes `just record-check` (same as local); a drifted generated block fails the job; CRLF checkout does not; history-reading jobs use fetch-depth 0; LF pinned for docs/** and .kaya/**; .whisper ignored. CI never runs build/publish as a writer.

