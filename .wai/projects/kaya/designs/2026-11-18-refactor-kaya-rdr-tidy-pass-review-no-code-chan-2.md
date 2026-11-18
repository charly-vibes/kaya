---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-rdr-publish-wiring-preparation, pipeline-step:refactor]
---

REFACTOR kaya-rdr — tidy pass review

No code changed; no refactor commit. The metadata block is grouped with a comment explaining why each exclusion exists, and the runbook is ordered one-time setup first, then every release.

Candidates rejected: replacing the tag-version check with `cargo metadata --format-version 1` parsing — heavier than the single-version manifest needs and unreadable in a runbook; adding the packaging commands to CI — `cargo publish --dry-run` updates the registry index, which CI does not need on every run.

