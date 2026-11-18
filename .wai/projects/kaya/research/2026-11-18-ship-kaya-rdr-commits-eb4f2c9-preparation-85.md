---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-rdr-publish-wiring-preparation, pipeline-step:ship-close]
---

SHIP kaya-rdr — commits eb4f2c9 (preparation) + 85a7957 (evidence)

kaya-rdr stays OPEN by design: its acceptance is the maintainer setting CARGO_REGISTRY_TOKEN and an authorized tag run publishing the crate, which this session must not perform. The ticket carries a comment with the exact remaining steps (scoped token, gh secret set, tag == Cargo.toml version, authorized tag push, channel report crates|updated, cargo search).

Verification at ship: cargo package --list 44 files (was 482); cargo publish --dry-run packaged, compiled and aborted at upload only; cargo test 24 targets; just ci green. No secret set, no tag, no push, no publish.

Known advisory: the same 50 pre-existing pretender style-threshold findings, 0 new.

