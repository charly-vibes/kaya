---
tags: [pipeline-run:tdd-ro5u-2026-11-19-kaya-syx-turu-adapter, pipeline-step:red]
---

RED kaya-syx: command=cargo test --test turu. Expected failure confirmed: 13 tests, 0 passed, all failing on missing behavior (empty-vec stub, absent REGISTRY entry, absent CLI subcommand, model ignoring capture.entry.observed). Setup intact: cargo check --test turu clean. Deploy verified byte-identical: openspec/specs/kaya-adapters/{spec.md,layer.md} cmp-clean vs archive; 10 tomls in .espectacular/kaya-adapters/; turu scenario bound to bundle_captured_without_whisper_workspace_internals; all archived bindings resolve to existing test fns.
