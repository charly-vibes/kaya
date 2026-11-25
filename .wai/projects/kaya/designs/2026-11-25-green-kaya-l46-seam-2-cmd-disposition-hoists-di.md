---
tags: [pipeline-run:tdd-ro5u-2026-11-25-kaya-l46-human-verification-trail-attributed-and-content-bearing, pipeline-step:green]
---

GREEN kaya-l46: seam 2 - cmd_disposition hoists digest=dependency_digest(record), both reviewed+waived bind it, waiver shape unchanged, dedup untouched. seam 1 - kaya ir --walk: per block body + provenance resolved to {id,type,subject,actor,timestamp} (unresolved flagged, never failing) + per-record trail {actor, disposition, digest, fresh} (fresh null when legacy/acknowledged lacks digest). IrCollector pub(crate) for cross-module read. One approved test edit: escaped-LF needle (raw LF unsatisfiable in compact JSON). Orchestrator verified: acceptance 5/0, suite 513/0, fmt clean, clippy -D warnings clean.
