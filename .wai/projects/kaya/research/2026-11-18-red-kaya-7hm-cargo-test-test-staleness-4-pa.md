---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-7hm-authored-staleness, pipeline-step:red]
---

RED kaya-7hm: `cargo test --test staleness` → 4 pass, 9 fail as expected: unknown citation marker not refused; acknowledge command absent (9 ack tests fail with kaya.acknowledge.* label expectations). New tests/staleness.rs (13 bound tests + 3 extras) and 13 TOMLs under .espectacular/changes/clarify-authored-staleness/kaya-projection; ah check --changes clarify-authored-staleness reports 0 structural. No src changes.

