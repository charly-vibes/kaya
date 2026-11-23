---
tags: [pipeline-run:tdd-ro5u-2026-11-23-kaya-yvr-document-ir-materialization, pipeline-step:ship-close]
---

SHIP kaya-yvr: commits 25a198b (feat: document IR materialization + kaya ir + spec deploy + bindings) and ccc41d7 (fix: refuse grammar-broken sources) on main, not pushed. Beads kaya-yvr open->close on approve; export follows close. Gates at ship: cargo test green, clippy 0, just ci passed, openspec strict 9/9, spk lint 0, ah check exit 0, RO5U verdict pass with 1 critical fixed and 0 remaining. Residuals tracked: kaya-7es (doctests) next, kaya-rp6 (gate) after; record-granular invalidation + composed-record CLI population + structure-read-from-the-ir binding ride with kaya-rp6.
