---
tags: [pipeline-run:tdd-ro5u-2026-11-24-kaya-rp6-impact-gate, pipeline-step:quality-ledger]
---

QUALITY LEDGER kaya-rp6 impact gate:

Changed:
- src/impact.rs (new): the kaya.impact library — ChangeSet/Reason/ImpactMap/map_impacts (edge + rule + transitive containment mapping), Policy/Rule/Severity (explicit-severity line parser with quoted values and # comments), dependency_digest (SHA-256 over canonical path/artifact/cite edges), GateInput/evaluate/Report (stale > lapsed-waiver > reviewed > updated > validated > missing precedence, failed checks as failed_check obligations, stale obligations read from ledger state, deterministic BTreeMap-ordered report, no run timestamps).
- src/main.rs: Impact/Disposition CLI commands with usage-labeled parse errors, cmd_impact (verified ledger path via read_build_ledger(None,false), exit 0 pass/review vs 2 fail), cmd_disposition (human-only refusal before any append, verification.record.reviewed/waived events, digest binding for waivers, idempotent duplicate detection), impact_base_rev/git_changed_paths/git_ledger_event_ids/base_documents/git_output helpers, collect_document_ir extracted from cmd_ir so gate and kaya ir read one record.
- src/projection/ir.rs: named_blocks() exposing materialization naming for base-vs-current byte comparison.
- src/lib.rs: pub mod impact.
- tests/impact.rs: 15 tests — mapping (edge/rule/transitive), dispositions derived, advisory review, waiver binding + lapse, human-actor refusal, stale unmet (ledger-only change set), failed doctest, exit contract, complete pass, determinism, range E2E, policy parse refusals + comments, IR structure.

Verified:
- cargo test: 35 test binaries, 0 failures (includes tests/impact.rs 15/15 green).
- cargo clippy --all-targets: 0 warnings.
- ah check --changes add-document-ir-commit-gate: no issues (0 structural, 0 execution).
- ah check --changes add-document-ir-commit-gate --run-tests: OK 227 passed, 0 findings.
- openspec validate add-document-ir-commit-gate --strict: valid.
- Manual E2E on a scratch repo: disposition append + gate --staged full report inspected; refusal-before-append confirmed by ledger-count test.

Review:
- RO5U complete, verdict NEEDS_REVISION at review time, zero CRITICAL; both HIGH and the MEDIUM fixed this step (range base-rev derivation + E2E test, explicit severity refusal, inline-comment parsing, ls-tree-based base documents with labeled git refusals). TypeSafe verification ran; both HIGH findings verified below the 0.8 gate (0.64/0.79) and carry REVIEW_REQUIRED — re-verification after the fixes not yet run.

Risks:
- openspec archive would refuse the change's kaya-projection delta (ADDED requirement Document IR materialization already exists — pre-existing from commit 1bdef7b, kaya-aaz proposal; fix at archive time by converting ADDED to MODIFIED).
- The authored-1/authored naming seam between the staleness layer and the IR is bridged in one place (ir_block_name); unifying the naming convention is a behavior change left out of this ticket.

Next:
- wai pipeline next (ship-close: human approval, atomic commit, close the beads ticket, no push).
