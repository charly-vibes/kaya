---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-8ot-kaya-check-facts, pipeline-step:plan]
---

PLAN kaya-8ot — facts for policy

Design:
- src/verification/facts.rs: Facts { events, ledger_digest, last_capture_commit, newest_event_at,
  events_by_source, emitted_kinds, omitted_kinds }; source classification from event.type and
  provenance.tool; capture frontier from capture.commit.observed data.sha ordered by parsed
  RFC3339 timestamps; dotted kind vocabulary.
- verification::check returns Outcome { drift, facts }; read_ledger returns (events, digest);
  plan_research returns (expected docs, omitted dotted kinds); emitted kinds come from the
  generated block names in expected research documents (namespace suffix stripped).
- src/main.rs serializes facts into the check envelope data alongside drift/stale.
Out of scope: thresholds, assertions, staleness, writes.

Tests first (tests/verification.rs):
- check_facts_reported: events count, digest = sha256 ledger, newest_event_at, key set, dotted kinds.
- check_capture_frontier_survives_rewrite: import git, record last_capture_commit, amend history,
  assert the recorded sha is still reported.
- check_sources_and_kinds_canonical: seed model + one wai artifact + git imports, zero test-runner,
  never the seed actor name, dotted emitted/omitted kinds.
TOMLs: facts-without-a-second-parser, unreachable-capture-commit, sources-and-kinds-are-canonical, facts-reported.

Commands: cargo test --test verification; cargo test; clippy; fmt; ah check --changes
add-record-verification --run-tests; just ci.
Commit: feat(check): report record facts and canonical vocabulary. Closes kaya-8ot.

