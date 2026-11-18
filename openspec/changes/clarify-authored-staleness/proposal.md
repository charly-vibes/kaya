# Proposal: clarify-authored-staleness

## Why

The projection spec says a block is stale when cited evidence changes "after
its last acknowledgment", and that acknowledgment is a human act recorded as an
event. Neither half is implemented or defined precisely:

- `build` passes an empty acknowledgment map (`&BTreeMap::new()`,
  `src/main.rs:754`), so nothing can clear a flag and no command records one.
- The trigger is narrower than readers assume. `changed_event_ids`
  (`src/ledger.rs:404-419`) returns only events whose effective data differs
  after supersede resolution, so a block goes stale on a **correction**, never
  on newly arrived evidence. Observed: appending a run event leaves `stale`
  empty; appending a supersede for a cited id reports
  `{"block":"authored-1","evidence":["evt-…"]}`.
- `kaya:cites` markers are parsed by the projection but nothing requires or
  validates them. Test fixtures use them; kaya's own documents contain none, so
  staleness is inert in practice.

Without a precise trigger and a way to acknowledge, a stale flag is a dead
end: the only remedy is rewriting the authored prose, which defeats the
purpose of recording that the author has seen the change.

## What Changes

- Validate `kaya:cites` markers: ids must resolve to admitted events;
  unknown ids are refused with a labeled error naming marker and id.
- Define the staleness trigger explicitly: a cited id is changed only by a
  supersede that changes its effective data. New arrivals never age a block;
  that extension is a non-goal here.
- Add acknowledgment as a human-recorded event
  (`verification.acknowledged`) and a human-only command; acknowledgment
  clears exactly the named block and ids, never the whole document.
- No changes to generated content, block grammar, or byte preservation.

## Decisions

1. **Corrections-only aging in this milestone.** Aging on new evidence needs
   a relevance rule (which arrivals concern which block); adding it now would
   guess. Deferred with a ticket, stated as a non-goal.
2. **Acknowledgment is scoped, never global.** It names document, block, and
   the exact cited ids; partial coverage leaves the block stale.
3. **Tool actors cannot acknowledge.** Same principle as promotion: clearing
   a human flag is a human act.
4. **`build` keeps reporting; verification enforces.** `--fail-on-stale-authored`
   lives in `kaya check` (`add-record-verification`); this change defines what
   "stale" means and how it is cleared.

## Impact

- Specs: +3 requirements on `kaya.projection`; no existing requirement text
  changed, but its unimplemented acknowledgment clause gains a concrete shape.
- Code: marker validation, acknowledgment event parsing, and a new CLI
  command; no change to generated blocks.
- Tickets: kaya-7hm. Verification enforcement is `add-record-verification`
  (kaya-585), which reads the stale report defined here.
- Archive order matters: `kaya.projection` is still an undeployed delta of
  `add-core-research-record`. The deployment must merge this change's layer
  properties with the capability's existing six; the archive checklist requires
  the merged layer regardless of order (see tasks).