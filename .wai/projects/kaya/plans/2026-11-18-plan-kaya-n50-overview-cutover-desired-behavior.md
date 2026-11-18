---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-n50-overview-cutover, pipeline-step:plan]
---

PLAN kaya-n50 — overview cutover

Desired behavior: docs/public/overview.md becomes the canonical reader-facing answer for the overview product — an authored block stating what kaya is, why it exists, when to use it and its status, followed by the managed publication block. README keeps badges, a one-line purpose, status, install, FAQ and links, and drops the duplicated "When to use" and "The idea" sections. docs/src/index.md keeps the long-form motivation and design rationale but opens by pointing at the compiled overview instead of restating it. docs/src/document-taxonomy.md gains a pointer to the deployed specs and the compiled overview; its taxonomy tables stay because they are the recovered-taxonomy record, not overview prose.

Out of scope: concept cutover (no Finding/Decision records), any projection or publication code change, new ledger records, document-taxonomy table removal.

Test/check shape: no permanent test — the meter is editorial. Throwaway checks: `grep -c "kaya:authored" docs/public/overview.md` is 0 before and 1 after; the phrase-duplication sweep (`living record`, `documentation drifts`, `grows out of the work`) shows one owner after; `kaya publish` twice: authored bytes identical, second run recomputed []; `mdbook build` green; `just ci` green.

Files: docs/public/overview.md, README.md, docs/src/index.md, docs/src/document-taxonomy.md (pointer only).

Commit shape: one feature commit for the cutover; evidence commit for the run artifacts.
