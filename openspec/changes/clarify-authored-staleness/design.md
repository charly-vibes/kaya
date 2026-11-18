# Design: clarify-authored-staleness

## Context

The current behavior is coherent but undocumented and unusable:

| input | current result |
|---|---|
| new capture event | `changed_event_ids` empty → no stale |
| supersede of a cited id | id appears in `changed_ids` → block reported stale |
| acknowledgment | none exists; the map is hardcoded empty |
| unknown `kaya:cites` id | silently ignored |

The spec's sentence "after its last acknowledgment" therefore describes a
mechanism with no writer and no reader.

## Semantics

An authored block ages only when the **meaning of a fact it cites** changes.
Appending evidence changes what is true in the world but not what the author
asserted about the cited fact; a correction changes the cited fact itself.
That distinction is the whole design:

- `changed_event_ids` = ids whose effective data differs from their recorded
  data after supersede resolution. Unchanged.
- A block is stale when the intersection of its cited ids and that set is
  non-empty, minus ids covered by an acknowledgment for the same document and
  block.
- Partial acknowledgment is intentional: the author may have reviewed one
  changed id and not the other. Coverage is computed per id.

## Acknowledgment event

```json
{
  "type": "verification.acknowledged",
  "subject": { "type": "Document", "id": "<project>/Document/<document-path-slug>" },
  "data": { "document": "docs/research/experiments/….md", "block": "authored-1", "ids": ["evt-…"] },
  "actor": { "type": "human", "name": "<name>" },
  "provenance": { "tool": "kaya", "version": "…", "reference": "…" }
}
```

- `verification.*` is not a graph type; the model ignores it (`model::build`
  matches only its own event types and drops the rest). It affects stale
  resolution only. The subject is typed `Document`, outside the closed
  thirteen-entity set, which is safe precisely because the model never parses
  it; the spec states that explicitly.
- Only human actors may append it; the command refuses `actor.type == "tool"`.
- Acknowledgment does not touch authored bytes, so byte preservation is
  unaffected.
- Replay is idempotent: duplicate acks for the same (document, block, ids)
  are harmless; acks for ids no longer cited have no effect.

## Command

```
kaya acknowledge <document> --block <name> --events <id>... --actor <name>
```

It appends exactly one event and prints the standard envelope with
`envelope_kind: "acknowledge"`. It does not rebuild; the next `build`/`check`
reflects the acknowledgment.

## Alternatives rejected

- **Aging on any new event.** Requires deciding which arrivals concern which
  block; wrong guesses would cry wolf and train authors to ignore flags.
  Deferred.
- **Acknowledging by document only.** Clears unreviewed changes; violates the
  "human saw this evidence" intent.
- **Content-hash acknowledgment.** Renumbers on every prose edit and cannot be
  written before the edit. Block names are positional, but markers change only
  when citations change, and an ack names its ids explicitly.

## Risks

- **Positional block names.** `authored-1` renumbers if an earlier authored
  block is added. The command refuses a block name that is not an authored
  block in the named document, and an acknowledged id that is no longer cited
  is inert rather than wrong. A follow-up may key blocks by their first cited
  id.
- **Archive merge.** `kaya.projection` is not deployed yet; this change ships a
  cumulative layer (existing six properties + three new). Archive must merge,
  not overwrite (see tasks).