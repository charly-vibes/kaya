---
tags: [pipeline-run:tdd-ro5u-2026-11-24-kaya-w9t-capture-commands-implementation, pipeline-step:red]
---

RED kaya-w9t: tests/capture.rs (new, 27 tests) authored per openspec/changes/add-capture-commands/specs/kaya-capture/spec.md — observed 26 FAILED / 1 passed (all failures the RED signature: subcommands absent -> usage envelope exit 1). The passing test (missing --actor refusal) asserts only refusal+no-append because the spec pins the actor_required label solely for --actor-type tool; documented. Contract decisions recorded in the test header: envelope kind 'capture', data.outcome appended|duplicate|corrected, data.event = stored/derived id; default key asserted against computed sha256. Spec-vs-summary deltas noted (missing-actor label unpinned; timestamp-injected determinism tested via observable duplicate semantics).
