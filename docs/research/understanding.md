<!-- kaya:begin generated:understanding.findings-and-analysis -->
### Findings

- kaya/Finding/2026-11-25-121fc9ca — The omission machinery was honest about absence but silent about the unblock path; the admission table and feeder rules were source-code-only knowledge until kaya explain made them first-class CLI answers.
- kaya/Finding/2026-11-25-5be3a163 — Doc audit 2026-11-24: the kaya-managed human surface was navigational, not explanatory - getting-started and reference were citation stubs, five products and two collections had no backing documents, and newcomer comprehension flowed around kaya through README and openspec specs.
<!-- kaya:end generated:understanding.findings-and-analysis -->

<!-- kaya:begin generated:understanding.decision-record -->
### Decisions

- kaya/Decision/2026-11-25-05000531 — Ledger event ids are content-derived and never reused; corrections append superseding events instead of rewriting, and the wai adapter skips unchanged artifacts on re-import so replays stay idempotent.
- kaya/Decision/2026-11-25-0c7dd16e — Content-derived artifact attributes (title, title_digest) never feed the event id, so re-observing an edited artifact keeps its identity while the title moves.
- kaya/Decision/2026-11-25-9806082e — Conclusions stay proposals until a human promotes them; promotion is the only path from assertion to established fact, so the record cannot self-establish.
- kaya/Decision/2026-11-25-a6d6cf17 — Release claims are checked mechanically: the status document must agree with the Cargo.toml version, so shipped docs cannot drift from the manifest.
- kaya/Decision/2026-11-25-e1533efd — A publication product without a populated backing block is omitted and reported with reason no-backing-document; the tree never carries stub pages, so absence is always truthful.

### Evidence

- kaya/Source/docs-src-index.md@43b606f — docs/src/index.md@43b606f
- kaya/Source/openspec-changes-add-core-research-record-proposal.md@43b606f — openspec/changes/add-core-research-record/proposal.md@43b606f
- kaya/Source/openspec-changes-add-core-research-record-specs-kaya-adapters-spec.md@43b606f — openspec/changes/add-core-research-record/specs/kaya-adapters/spec.md@43b606f
- kaya/Source/openspec-changes-add-core-research-record-specs-kaya-event-layer.md@43b606f — openspec/changes/add-core-research-record/specs/kaya-event/layer.md@43b606f
- kaya/Source/openspec-changes-add-core-research-record-specs-kaya-promotion-spec.md@43b606f — openspec/changes/add-core-research-record/specs/kaya-promotion/spec.md@43b606f
- kaya/Source/wai-projects-kaya-designs-2026-11-16-post-bootstrap-alignment-consumers-mvp-1-accep.md@43b606f — .wai/projects/kaya/designs/2026-11-16-post-bootstrap-alignment-consumers-mvp-1-accep.md@43b606f
- attribution-stale — decision kaya/Decision/2026-11-25-05000531: no decision-source links in this ledger; source evidence is ledger-global
- attribution-stale — decision kaya/Decision/2026-11-25-0c7dd16e: no decision-source links in this ledger; source evidence is ledger-global
- attribution-stale — decision kaya/Decision/2026-11-25-9806082e: no decision-source links in this ledger; source evidence is ledger-global
- attribution-stale — decision kaya/Decision/2026-11-25-a6d6cf17: no decision-source links in this ledger; source evidence is ledger-global
- attribution-stale — decision kaya/Decision/2026-11-25-e1533efd: no decision-source links in this ledger; source evidence is ledger-global
<!-- kaya:end generated:understanding.decision-record -->
