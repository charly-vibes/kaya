---
tags: [pipeline-run:tdd-ro5u-2026-11-24-review-remediation-capture-labels-race-retry-state-hardening, pipeline-step:refactor]
---

REFACTOR remediation batch: tidy landed inside the batch and re-verified green — CaptureArgs flatten removes the 4x5 duplicated clap field blocks (EXCL-001), check_slug(field, value) replaces check_key and serves both key and project, effective_text_event dedups the newest-text-event find expression that the retry loop would otherwise duplicate, pid-unique temp builder is 3 lines in store(). Nothing further moved: the R05U review already shaped the structures, and any deeper restructuring would be behavior-risk without a finding. Tests green post-tidy: capture 31/31, full suite 35 suites ok, clippy -D warnings clean, fmt clean.
