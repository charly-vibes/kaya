# Walking the record: `kaya ir --walk`

Reader question: *a generated block claims facts — which evidence backs them,
and has a human actually checked it?*

`kaya ir` prints the document IR: the records the compiler materialized and
the generated blocks it rendered. `kaya ir --walk` renders the same blocks,
but pairs each one with what a reviewer needs beside it, read-only:

```console
$ kaya ir --walk
```

Per document, per generated block:

- **`body`** — the exact bytes the block contributes to the document. This is
  what you check: every claim in it should trace to the provenance beneath it.
- **`provenance`** — each backing id resolved against the ledger: the event's
  type, subject, actor, and timestamp. An id the ledger cannot resolve is
  reported as `"unresolved": true` rather than dropped. When evidence was
  corrected, the walk names the *correcting* event, not the superseded origin.
- **`trail`** — the human verification recorded for the record that owns the
  block: `reviewed` and `waived` dispositions from `kaya disposition`, and
  acknowledgments from `kaya acknowledge`, each with the actor.

The one field to read carefully is `trail[].fresh`, the digest freshness:

| `fresh` | Meaning |
|---|---|
| `true` | the event binds a digest and the record's dependency digest still matches — the verification stands for the current content |
| `false` | the record's inputs changed after the event was recorded — re-run the disposition or acknowledge |
| `null` | the event carries no digest — a pre-digest legacy event or an acknowledgment — so freshness is *unknown*, never assumed stale |

Dispositions and waivers bind the record's dependency digest at recording
time (`kaya disposition <record> reviewed|waived`), which is what makes the
`fresh` comparison possible. The walk never writes: it refuses nothing and
touches no file, so it is safe at any point in a session.
