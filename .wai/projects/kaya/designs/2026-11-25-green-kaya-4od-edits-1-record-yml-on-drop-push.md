---
tags: [pipeline-run:tdd-ro5u-2026-11-25-kaya-4od-scheduled-record-capture-races-in-flight-work, pipeline-step:green]
---

GREEN kaya-4od edits: (1) record.yml on: drop push block, keep schedule+workflow_dispatch, comment updated; (2) justfile record-capture: flock .kaya/capture.lock around body + dirty-tree skip; (3) scripts/record-conflicts.md union recipe.
