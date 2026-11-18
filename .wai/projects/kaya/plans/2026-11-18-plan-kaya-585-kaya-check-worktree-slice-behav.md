---
tags: [pipeline-run:tdd-ro5u-2026-11-18-kaya-585-kaya-check, pipeline-step:plan]
---

PLAN kaya-585 — kaya check (worktree slice)

Behavior (spec ids): read_only; no_relaxed_verification; drift_report; exit_contract;
reported_paths. Scenarios: drift-reported-without-writing, unverifiable-ledger-refused,
drifted-block, clean-record, missing-generated-document, usage-error-exits-2,
paths-follow-root, no_side_effects, drift_exit.

Out of scope: --index (kaya-a4e), facts (kaya-8ot), assertions/stale enforcement (kaya-ntv),
acknowledge (kaya-7hm), hooks/CI.

Design: one expected-bytes core shared with the writers.
1. projection/index.rs: split `build` into pure `plan(graph, path, existing: Option<&str>,
   event_count) -> (String, Summary)` and the existing write wrapper.
2. projection/report.rs: pure `plan_experiment(graph, experiment_id, existing: Option<&str>,
   promotion) -> PlannedDocument`; pure `plan_understanding` / `plan_overview`. Extract the
   body of `commit_project_document` so it returns the final document (empty = delete) without
   touching disk. `build*` wrappers keep their current contracts and write only when changed.
3. projection/reuse.rs: `plan(inputs, out, existing: &BTreeMap<PathBuf, Option<String>>) ->
   Vec<PlannedDocument>`; `build` applies the plan.
4. publication/mod.rs: `plan(decls, docs, out, read_text, read_bytes) -> Plan` containing every
   page document, the export bytes, omissions and summary; `build` applies it. No second renderer.
5. src/verification/mod.rs: `Source` (worktree read/list), `expected()` builds the research docs
   from the ledger and then the publication tree from those expected docs, `compare()` walks
   managed blocks, `run()` returns envelope data plus exit verdict.
6. src/main.rs: `Check { root, out, publish_out, expect_digest }`; ledger read anchored at root,
   committed-history verification mandatory (no relax flag); envelope kind `check`, ok:true;
   drift entries {path, block?, kind}; exit 0/1/2. Usage and check I/O use kaya.verification.*.

Test cases first (tests/verification.rs, assert_cmd + tempfile):
- clean committed fixture: exit 0, empty drift, every documented path relative to root;
- tampered index block: exit 1, drift changed naming docs/research/index.md and the block;
- deleted index.md: exit 1, kind missing;
- stray generated block in docs/research/experiments/stray.md: exit 1, kind extra;
- no-side-effects: hash tree + ledger before/after a drifted check, identical, drift still reported;
- unverifiable ledger (append an uncommitted line): exit 2, kaya.ledger.uncommitted_history, drift [];
- bad flag value: exit 2, kaya.verification.* error envelope;
- --root from a subdirectory: paths relative to root, exit 0.
Fixture: git init, copy .kaya/seeds/add-core-research-record.jsonl, `kaya import model`, commit,
`kaya build`, then `git add -A && git commit` so the ledger and products are committed.

Commands: narrow `cargo test --test verification`; full `cargo test`, `cargo clippy --all-targets
-- -D warnings`, `ah check --changes add-record-verification --run-tests`, `just ci`.

Files: src/main.rs, src/verification/mod.rs, src/projection/{index,report,reuse}.rs,
src/publication/mod.rs, tests/verification.rs,
.espectacular/changes/add-record-verification/kaya-verification/*.toml.

Commit shape: refactor commit (pure planning split) then feature commit (check), or one
feature commit if the split is not independently green; separate tidy commit if needed.

