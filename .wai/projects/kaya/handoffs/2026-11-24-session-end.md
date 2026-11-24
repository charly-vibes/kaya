---
date: 2026-11-24
project: kaya
phase: research
---

# Session Handoff

## What Was Done

<!-- Summary of completed work -->

## Key Decisions

<!-- Decisions made and rationale -->

## Gotchas & Surprises

<!-- What behaved unexpectedly? Non-obvious requirements? Hidden dependencies? -->

## What Took Longer Than Expected

<!-- Steps that needed multiple attempts. Commands that failed before the right one. -->

## Open Questions

<!-- Unresolved questions -->

## Next Steps

<!-- Prioritized list of what to do next -->

## Context

### open_issues

```
○ kaya-34g P3 Dogfood the experiment-record producer: record kaya's next experiment run with observations so the dossier gains an exploration.experiment-record block - un-omits example-gallery and the tutorial product
○ kaya-5cb P3 [bug] ah check invokes global kaya 0.1.0 (stale ~/.cargo/bin/kaya) - rewrote docs/public/publication.json in pre-0.2 export shape during 9dv run (restored from HEAD); pin ah/espectacular to repo binary or make ah check read-only
○ kaya-g9u P3 Split src/main.rs (2383 lines) into command modules: cmd_* dispatch fns accumulate complexity debt; pretender.toml file_lines 3500 ceiling is deliberate until this lands — tighten then
○ kaya-io6 P3 [bug] Scheduled snapshot job pushes straight to main (~5min cadence); collides with concurrent pushes - forced two rebases during v0.3.0 release (6f86e06, 287470f). Make the snapshot job push with rebase+retry (pull --rebase then push in a loop) so it yields to interactive pushes instead of racing them
○ kaya-quh P3 Dogfood reuse rows: import specodelic/turu sources so the projection compiles technical-reference.md - un-omits api-catalogue and feeds how-to/reference
○ kaya-w68 P3 [bug] Read-list gap: brief.md carries exploration.exploration-brief but is in neither read list, so tutorial cannot compose it - align both lists (source.rs + RESEARCH_READ_ORDER)
○ kaya-ae4 P4 release.yml actions target Node 20 and are force-moved to Node 24 (checkout/upload-artifact/download-artifact deprecation warnings on v0.3.0 run 45494228945); bump action versions before GitHub hard-fails

--------------------------------------------------------------------------------
Total: 7 issues (7 open, 0 in progress)

Status: ○ open  ◐ in_progress  ● blocked  ✓ closed  ❄ deferred
Priority: P0–P4 (label only; not a status icon)
```

