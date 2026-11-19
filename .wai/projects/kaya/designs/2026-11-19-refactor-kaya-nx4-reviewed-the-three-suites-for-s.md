---
tags: [pipeline-run:tdd-ro5u-2026-11-19-kaya-nx4-hegel-property-tests, pipeline-step:refactor]
---

REFACTOR kaya-nx4: reviewed the three suites for self-contained tidy: generator helpers are per-file (no shared module warranted — three files, three alphabets, no duplicated logic worth extracting across suites); oracle predicates kept local to each property; no dead code. Deliberate no-op beyond the fixes already applied in red (git init, exit oracle, listing scope, anchor search). Rationale: property suites stay reviewable next to their dossiers; extracting a shared harness module would couple three independent properties for zero behavior gain.
