---
tags: [pipeline-run:tdd-ro5u-2026-11-17-kaya-scq-status-release-architecture, pipeline-step:green]
---

GREEN kaya-scq: cmd_publish declares eleven products (all but tutorial) with includes from publication::admission; no publication-module change needed. Regenerated docs/public: architecture-design (owns the research-map body; reference cross-references it), release-migration and status-compatibility (cross-reference the shared overview body), publication.json with 11 product rows + 2 packages; omitted [concept, example, faq, troubleshooting]. Tests: publications_export.rs (row/emission pinning, per-section attribution equal to export event_ids, repeat byte identity) + CLI remaining-products publish; contract format-attribution. Docs status lines updated. Verified: 23 test targets green, fmt/clippy clean, publish idempotent, ah check --run-tests 13 passed.
