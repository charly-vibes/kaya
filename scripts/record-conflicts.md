# Resolving record-capture conflicts (union by event id)

Two capture actors (local `just record-capture` and the Actions Record
workflow) both append to `.kaya/ledger.jsonl` and regenerate
`docs/research/` + `docs/public/`. When a push lands while an Actions
capture run is executing, the next local push rebases onto a remote
snapshot that contains different capture events for the same commits, and
the rebase conflicts on the ledger and the generated docs.

Observed live twice on 2026-11-25. Prevention: the Record workflow no
longer triggers on push (nightly + dispatch only, see
`.github/workflows/record.yml`), and the recipe skips dirty trees and
serializes on `.kaya/capture.lock`.

## The recipe

Generated files are pure projections of the ledger, and capture events are
content-derived and never reused, so conflicting snapshots are unions, not
divergences. Resolve by proving which side is the superset, then taking it:

1. Start from the rebase conflict (`git pull --rebase`; the fix commit
   usually reapplies cleanly, the capture commit conflicts).
2. Diff the event-id sets through the conflict stages. Under a rebase the
   sides are flipped from what merge intuition says: stage `:2:` holds the
   remote/base content, stage `:3:` holds the replayed local commit (this
   was verified experimentally; getting it backwards routes the common
   case into the rare branch).

   ```bash
   git show :2:.kaya/ledger.jsonl | jq -r '.id' | sort > /tmp/remote
   git show :3:.kaya/ledger.jsonl | jq -r '.id' | sort > /tmp/mine
   comm -13 /tmp/remote /tmp/mine | wc -l   # events only ours has
   ```

   `0` means the remote snapshot is a superset of ours (the common case —
   the Actions run observed our already-committed work).
3. If the remote is the superset: take its side wholesale. Under rebase
   semantics `--ours` is the remote/base side:

   ```bash
   git checkout --ours .kaya/ledger.jsonl docs
   git add .kaya docs
   git -c core.editor=true rebase --continue
   ```

4. If we hold unique events (rare — local capture of commits the Actions
   run never saw): append only those `comm` lines (from `/tmp/mine`) to
   the remote-side ledger (`git show :2:` content), regenerate the
   projections (`kaya build && kaya publish`), add, and continue.
   Appending to the local side would duplicate them.
5. Verify before pushing: `just record-verify`, `cargo test -q`, and
   `target/debug/kaya impact --staged` (expect `pass`, 0 unmet).

## Notes

- Do not hand-edit conflict hunks in generated files; every hunk is
  reproducible from the ledger.
- A rebase rewrites the commit shas the ledger's capture events name. The
  next capture appends fresh `capture.commit.observed` events for the
  rewritten HEAD; the stale-sha events stay as history.
- `record-lag.py` (`just record-check`) fails when the capture frontier
  falls too far behind; after conflict resolution it should pass.
