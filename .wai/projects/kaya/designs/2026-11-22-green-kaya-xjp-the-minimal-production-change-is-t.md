---
tags: [pipeline-run:tdd-ro5u-2026-11-22-kaya-xjp-verification-facts-census, pipeline-step:green]
---

GREEN kaya-xjp: the minimal production change is two edits in src/verification/facts.rs.

1. SOURCES grows from four to five entries: pub const SOURCES: [&str; 5] = ["wai", "git", "test-runner", "model", "turu"]. The canonical list is the single source of truth for three call sites: events_by_source seeds every key at 0 (so the census always reports every canonical source, including a zero turu), validate_request in assertions.rs accepts --require-source turu as vocabulary (and its error message joins the same list), and the facts report enumerates them.
2. source_of gains one arm: CAPTURE_ENTRY ("capture.entry.observed") => Some("turu"), with a named constant beside the sibling event-type constants.

Why the arm needs no provenance guard, unlike the wai arm: capture.artifact.observed is a shared capture family that another adapter could also emit, so wai's arm checks provenance.tool == "wai". capture.entry.observed is emitted only by the turu adapter (src/adapters/turu.rs:30), and the event type alone names its capture family - the same rule the git arm follows with capture.commit.observed. The actor name stays out of it: source_of never reads actor.

No other production change: the turu adapter already emits capture.entry.observed with provenance.tool=turu and a Source subject; the model already maps that event type to a Source node (src/model.rs:31, :439); the CLI import envelope already names the import source 'turu' (src/main.rs:570); validate_request and required() both read SOURCES, so the new vocabulary flows into --require-source with no edit.

Verification after the change: cargo test --test verification 35 passed 0 failed; cargo test 371 tests passed across 31 suites; cargo clippy --all-targets --all-features -- -D warnings clean; ah check --run-tests 180 passed 0 findings; openspec validate --specs --strict 9 passed 0 failed; openspec validate --archived --strict shows change/2026-11-22-add-turu-source-row valid (the 3 pre-existing incomplete-task failures in add-capture-attributes and add-decision-evidence-link predate this ticket); spk lint openspec/specs 0 issues.
