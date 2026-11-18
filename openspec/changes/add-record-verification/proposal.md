# Proposal: add-record-verification

## Why

The record cannot be checked without writing. `kaya build` verifies the ledger
against `HEAD:.kaya/ledger.jsonl` and rewrites projection blocks, so it cannot
run on staged content: a newly staged ledger fails
`kaya.ledger.uncommitted_history` (`src/main.rs:763-866`), and when a block has
drifted, `build` repairs it, exits 0, and leaves the worktree dirty (observed:
`recomputed ["orientation.research-map"]`, then `M docs/research/index.md`).
No command answers "does the committed record match its projections?" without
side effects, so no pre-commit hook can enforce it; the repository's own
`docs/src/status.md` drifted past every gate and still claims no release
exists.

Two adjacent gaps make the drift worse. Staleness is reported but
unenforceable — `build` exits 0 with a non-empty `stale` list, and
acknowledgment is hard-coded empty (`&BTreeMap::new()`, `src/main.rs:754`)
even though the projection spec requires it. And policy checks have no facts:
`kaya status` reports bytes, digest, events, ledger, pending proposals, and
replay integrity only, so any currency or coverage gate would need a second
JSONL parser.

## What Changes

- Add the `kaya.verification` capability, exposed as the `kaya check` command:
  a read-only verifier with worktree and staged (`--index`) modes, a drift
  report, an explicit exit contract, record facts for repository policy,
  opt-in kind/source assertions, and staleness enforcement.
- `build` and `publish` keep their contracts. `check` shares their projection
  computation but writes nothing; no new event types are introduced.
- Staleness *semantics* (markers, trigger scope, acknowledgment) are specified
  by `clarify-authored-staleness`. This change only enforces what the
  projection reports.
- Non-goals: capture, network access, thresholds, allowlists, and any writer
  behavior inside `check`.

## Decisions

1. **A separate command, not `build --check`.** `build`'s contract is "verify
   HEAD, then write". Overloading it invites exactly the pre-commit misuse
   that fails today.
2. **`--index` reads `:<path>` blobs.** Hooks run on partially staged trees;
   comparing the worktree produces false pass/fail.
3. **Facts, not policy.** kaya reports the last captured commit, per-source
   counts, and emitted/omitted kinds. Bounds, required lists, and allowlists
   stay in the repository.
4. **Exit codes are the contract.** 0 clean, 1 violation, 2 usage or I/O.
   Violations are structured data (`drift`, `missing`, `stale`), never prose.
5. **Enforcement is opt-in per flag.** `--fail-on-stale-authored`,
   `--require-kind`, `--require-source` change the exit code; plain `check`
   never fails on advisory content.
6. **One canonical vocabulary.** Sources are `wai`, `git`, `test-runner`, and
   `model`, derived from the event's capture family and provenance, not from
   the actor name — model seed events carry a human actor. Kinds are dotted
   `family.kind` names, so emitted and omitted kinds share one spelling.

## Impact

- Specs: +1 capability (`kaya.verification`); no existing requirement changed.
- Docs: the published status page says "six capabilities"; it must name seven
  and list the new command (tracked in tasks and kaya-gh2).
- Code: new CLI surface; the projection builders gain a non-writing entry
  point; the ledger reader gains an index source; no ledger writes.
- Tickets: kaya-585 (implementation). Repository wiring is kaya-3kg;
  staleness semantics are kaya-7hm.