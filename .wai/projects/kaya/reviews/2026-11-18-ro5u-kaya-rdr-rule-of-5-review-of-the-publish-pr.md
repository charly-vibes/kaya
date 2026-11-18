---
reviews: 2026-11-18-green-kaya-rdr-publish-wiring-prepared-cargo-to.md
verdict: pass
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-rdr-publish-wiring-preparation, pipeline-step:ro5u-review]
---

RO5U kaya-rdr — Rule-of-5 review of the publish preparation

Artifact under review: Cargo.toml package metadata and exclusions, RELEASING.md, .github/workflows/release.yml header, README crates.io paragraph.
TYPESAFE_API_KEY is not set: findings are UNVERIFIED (self-reported).

STAGE 1 — DRAFT
The change prepares a distribution path without exercising it: the crate tarball is scoped, the one-time and per-release human steps are in one runbook, and the workflow points at it. No behavior of the compiler changes. Shape Quality: GOOD.

STAGE 2 — CORRECTNESS
CORR-001 LOW — RELEASING.md tag command
`grep '^version' Cargo.toml` + `sed -n 's/^version = "\(.*\)"/\1/p' | head -1` reads the first `version =` line. The manifest has exactly one today, so the command is correct, but a future `[workspace.package]` or target-specific version would silently take the wrong one.
Evidence: Cargo.toml has a single `version = "0.1.0"` at line 4.
Fix: none now; the workflow already refuses to publish when the tag and manifest version disagree (release.yml compares TAG against CRATE_VERSION), so a mistake fails loudly before upload.

CORR-002 LOW — excluded paths
`exclude` names directories rather than globs; cargo treats each entry as a path pattern, and the dry run confirms 44 files with no tooling trees. If a new tooling directory appears, it ships until added. Accepted: the list is explicit and reviewed at release time.
Correctness Quality: GOOD (dry run packages and compiles the crate; upload aborts only by design).

STAGE 3 — CLARITY
CLAR-001 LOW — RELEASING.md "Every release" step 2 prints the tag but step 3 leaves `<x.y.z>` for the reader. The two lines are adjacent and the tag is shown by step 2; acceptable.

Clarity Quality: GOOD.

STAGE 4 — EDGE CASES
EDGE-001 — token present but crates.io rejects the version (already published). The runbook says re-running is safe and a version that exists is rejected, never overwritten, and gives the yank path for a bad version. Covered.
EDGE-002 — token absent: workflow reports `disabled` without failing; the runbook states this and how to replace the secret. Covered.
EDGE-003 — release health aggregation: the crates channel is already included in the summary loop and the failure check; unchanged by this ticket.
Edge Case Coverage: GOOD.

STAGE 5 — EXCELLENCE
EXCL-001 LOW — the acceptance criterion "the crates.io channel stops reporting disabled" is only observable after the maintainer sets the secret and pushes an authorized tag; this ticket cannot verify it. Recorded as the remaining human step instead of pretending closure.

Convergence (after stage 4): new CRITICAL 0; no new issues in stage 5; false positives 0. CONVERGED.

Final: CRITICAL 0, HIGH 0, MEDIUM 0, LOW 4.
Verdict: READY — wiring prepared; the publish itself remains a human release action.

