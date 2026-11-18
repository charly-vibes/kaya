# Tasks: add-record-verification

Red→green→refactor per requirement; every scenario in the delta is a
contract-test shape. One tdd-ro5u run per ticket.

## 1. Red — failing tests first

- [ ] 1.0 Declare one contract per scenario:
      `.espectacular/changes/add-record-verification/kaya-verification/<scenario>.toml`
      bound to its future test name, so `ah check --changes` reports
      `no-tests-declared` instead of `no-toml`. One TOML per scenario;
      slices are kaya-585 (worktree), kaya-a4e (index), kaya-8ot (facts),
      kaya-ntv (assertions and stale).
- [x] 1.1 Drift fixtures: corrupted managed block → `changed`; emitted
      document absent → `missing`; managed document the projection would not
      emit → `extra`; clean record → empty drift, exit 0. (kaya-585)
- [x] 1.2 No-side-effects fixture: hash the tree and the ledger before and
      after a check over a drifted record; assert identical and assert the
      drift was still reported. (kaya-585)
- [ ] 1.3 Index fixtures: document drifted in the index only → fails; drifted
      in the worktree only → passes; ledger staged with no `HEAD` copy →
      verifies instead of refusing; unmerged or unreadable index path → exit 2
      with `kaya.verification.check_io`, never a worktree fallback. (kaya-a4e)
- [ ] 1.4 Facts fixture: wai, git, and model events, known newest commit →
      `events_by_source` uses the `wai`/`git`/`test-runner`/`model` keys (the
      seed's human actor never appears), and emitted/omitted kinds are dotted
      `family.kind` names with no duplicate spelling. (kaya-8ot)
- [ ] 1.5 Assertion fixtures: `--require-kind` over an omitted dotted kind →
      violation with `no-records`; `--require-source test-runner` over a
      record with no run events → violation naming the source. (kaya-ntv)
- [ ] 1.6 Staleness fixtures: supersede under a cited authored block with and
      without `--fail-on-stale-authored`; bytes unchanged in both cases;
      `build` still exits 0. (kaya-ntv)
- [x] 1.7 Exit fixtures: clean 0, violations 1, bad flag/missing path 2;
      exactly one envelope on stdout in every case. (kaya-585)

## 2. Green — make the tests pass

- [x] 2.1 Extract the expected-bytes stage from `build`/`publish` so `check`
      renders the same bytes without writing; no second renderer. (kaya-585)
- [ ] 2.2 Add the index reader (`git show :<path>`) for the ledger and every
      projection document; treat an unstaged absent path as absent. (kaya-a4e)
- [x] 2.3 Add `kaya check` with `--root`, `--out`, `--publish-out`,
      `--expect-digest`; emit the `check` envelope with structured findings and
      the exit contract. `--index` lands in kaya-a4e. (kaya-585)
- [ ] 2.4 Add the facts block (kaya-8ot) and the opt-in assertions
      (kaya-ntv); reuse the build summary's kind vocabulary.
- [ ] 2.5 Wire `--fail-on-stale-authored` to the existing stale report; do not
      change `build` behavior. (kaya-ntv)

## 3. Refactor — separate commit

- [ ] 3.1 One comparison core shared by worktree and index modes; no mode
      branch inside the renderer. (kaya-585)
- [ ] 3.2 Documentation: command table in README; `docs/src/status.md`
      implemented list and its capability count (six → seven); the hook example
      that uses `check --index`, never `build`. (kaya-gh2)

## 4. Repository gates — outside this spec

These consume `kaya check`; they are repository policy, not capability
behavior. They may land after the command does.

- [ ] 4.1 `just record-check` (check + lag + links) and lefthook wiring:
      `kaya check --index --fail-on-stale-authored` in pre-commit,
      `just record-verify` in pre-push. (kaya-3kg)
- [ ] 4.2 CI runs the identical `just record-check`; any job reading history
      uses `fetch-depth: 0`; `.gitattributes` pins LF for docs and a
      `.whisper/` ignore is added. (kaya-tm8)
- [ ] 4.3 Lag gate reading `kaya check` facts: warn on PRs, fail on
      `main`/release above the configured bound. (kaya-8dg)
- [ ] 4.4 Scheduled capture on `main` (`import wai`, `import git`, `build`,
      `publish`) that commits the snapshot deliberately. (kaya-9go)
- [ ] 4.5 Link check over the built book (kaya-nwv) and the release-claim check for
      `docs/src/status.md` (kaya-gh2).
- [ ] 4.6 Publication tree reachable from the deployed docs (copy into the
      book or a second Pages job). (kaya-2b0)

## 5. Archive checklist

- [ ] Every `kaya.verification` scenario has a TOML contract and an executing
      test (`ah check --changes add-record-verification --run-tests` reports no
      `no-toml` or `no-tests-declared`).
- [ ] `spk lint` clean on the new layer; property rows match the mirror
      scenarios one-to-one.
- [ ] No duplicated renderer: a fixture builds and checks the same record and
      asserts equality of expected bytes.
- [ ] kaya-585, kaya-a4e, kaya-8ot, kaya-ntv closed; repository tickets
      remain open or closed on their own evidence.