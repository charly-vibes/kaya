---
tags: [pipeline-run:tdd-ro5u-2026-11-24-review-remediation-capture-labels-race-retry-state-hardening, pipeline-step:red]
---

RED remediation batch: command=cargo test --test capture with src/ implementation stashed (tests kept). Observed: 4 failed, 27 passed — capture_project_off_alphabet_refused FAILED (no project validation existed), edited_capture_emits_a_superseding_correction FAILED (envelope lacked data.supersedes), plus the two label-assertion tests FAILED under old bound_exceeded label. Unit-side red implicit: parity test module and state tests live in src/ (stashed with implementation); their absence from the stashed tree confirms they are new expectations. Failure class matches plan: missing behavior, not setup breakage.
