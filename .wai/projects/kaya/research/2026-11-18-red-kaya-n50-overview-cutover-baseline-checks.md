---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-n50-overview-cutover, pipeline-step:red]
---

RED kaya-n50 — overview cutover baseline

Checks (editorial meter, run against the pre-cutover tree):
- `grep -c "kaya:authored" docs/public/overview.md` = 0: the compiled overview page has no authored interpretation, only the managed publication block.
- `grep -rl "grows out of the work" README.md docs/src docs/public` = README.md, docs/src/index.md: the same overview claim is maintained by hand in two places and absent from the compiled page.
- README "## The idea" section = 28 lines duplicating the event format and generated/authored block explanation that docs/src/index.md already carries under "Documentation compilation".
- The committed ledger has no Finding/Decision entities, so the concept product cannot be generated; the cutover is overview-only as approved.

Expected failure: the overview product does not answer its reader question from authored interpretation plus generated content, and the hand-written copies remain split across README and index.md.
