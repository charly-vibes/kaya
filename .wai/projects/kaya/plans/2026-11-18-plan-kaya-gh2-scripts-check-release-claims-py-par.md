---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-gh2-doc-truth, pipeline-step:plan]
---

PLAN kaya-gh2: scripts/check-release-claims.py parses Cargo.toml version, requires docs/src/status.md to name v<version> in its Release row, and forbids stale phrases ("no release tagged", "crates.io ... deferred", "kept current by"). Update status (seven capabilities, kaya check/acknowledge, v0.1.0/kaya-vibes published) and README wording. Verify current pass; inject a wrong version -> exit 1. justfile release-claims recipe; wired into record-check in kaya-tm8.

