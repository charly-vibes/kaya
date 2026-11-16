---
tags: [pipeline-run:tdd-ro5u-2026-11-16-kaya-htk-release-workflow, pipeline-step:green]
---

GREEN kaya-htk: added .github/workflows/release.yml (tag-only trigger, 5 targets, test gate, SHA-pinned actions, read-default/write-only-publish permissions, version-tag check, packaging+checksums+provenance, gh release, token-gated crates.io, retried+reported tap/bucket propagation), scripts/update-homebrew.py, scripts/update-scoop.py, tests/release_scripts.rs, plus README badge, status page row, CHANGELOG entry.

Decisions:
- Checksum lookup compares basenames so sha256sum's ./* prefix works; exact filename match keeps version/tag correspondence enforced (partial release aborts before writing).
- Python path fixed the inherited fleet template bug: wai/ddl formulas render system "\#{bin}/wai" (escaped interpolation, brew test would exec a literal '#{bin}/wai'); kaya renders "#{bin}/kaya" and ruby -c passes.
- crates.io step is token-gated inside the step (secrets context is not allowed in step if:) and writes a channel status file; a missing token reports 'disabled', not failure.
- Health summary includes crates/homebrew/scoop and fails only on 'failed'.
- Release step order matches ddl/wai: release assets first, then crates.io, then channels.
