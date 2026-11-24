---
tags: [pipeline-run:tdd-ro5u-2026-11-24-kaya-w9t-capture-commands-implementation, pipeline-step:green]
---

REFACTOR kaya-7r3-style tidy note: capture.rs owns redaction+bounds+id+decision in one module; main.rs arms are thin dispatch; no duplication across the four subcommands (shared handler). No further refactor needed.
