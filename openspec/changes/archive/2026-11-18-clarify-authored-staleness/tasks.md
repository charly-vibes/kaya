# Tasks: clarify-authored-staleness

Semantics before enforcement: this change defines what "stale" means; the
`kaya check` flag that fails on it ships in `add-record-verification`.

## 1. Red — failing tests first

- [x] 1.0 Declare one contract per scenario:
      `.espectacular/changes/clarify-authored-staleness/kaya-projection/<scenario>.toml`
      bound to its future test name. (kaya-7hm)
- [x] 1.1 Marker fixtures: known id accepted and used for stale detection;
      unknown id refused naming document, marker, and id; marker in
      uncategorized prose preserved byte-identically and inert. (kaya-7hm)
- [x] 1.2 Trigger fixtures: append a non-superseding event for a cited block →
      not stale; append a supersede that changes its data → stale naming the
      id; multiple changed ids all named. (kaya-7hm)
- [x] 1.3 Acknowledgment fixtures: full coverage clears; one of two ids
      acknowledged stays stale naming the other; tool actor refused with no
      append; unknown document or block refused with no append; acknowledged
      ids that are no longer cited are inert; replay leaves generated blocks
      and graph unchanged; authored bytes identical before and after. (kaya-7hm)
- [x] 1.4 Idempotence fixture: the same acknowledgment appended twice changes
      no stale result and appends nothing on replay. (kaya-7hm)

## 2. Green — make the tests pass

- [x] 2.1 Validate `kaya:cites` markers inside authored blocks in the shared
      projection read path used by `build` and `check`; unknown ids fail with
      a labeled error. (kaya-7hm)
- [x] 2.2 Parse `verification.acknowledged` events and resolve them per
      (document, block, ids); replace the hard-coded empty map in
      `flag_stale_authored` (`src/main.rs:754`). (kaya-7hm)
- [x] 2.3 Add `kaya acknowledge <document> --block <name> --events <id>...
      --actor <name>`: human-only, appends exactly one event, emits the
      standard envelope, never rebuilds. (kaya-7hm)
- [x] 2.4 Keep `verification.*` out of the research graph and out of
      generated blocks. (kaya-7hm)

## 3. Refactor — separate commit

- [x] 3.1 Move acknowledgment resolution next to `stale_authored` so stale
      computation has one owner; no second marker parser. (kaya-7hm)
- [x] 3.2 Documentation: how to cite evidence in an authored block, and how
      to acknowledge a correction; add a `kaya:cites` marker to kaya's own
      dossier so the mechanism is exercised in the dogfood record. (kaya-7hm)

## 4. Archive checklist

- [x] All delta scenarios have a TOML contract and an executing test
      (`ah check --changes clarify-authored-staleness --run-tests` reports no
      `no-toml` or `no-tests-declared`).
- [x] `spk lint` clean on the layer.
- [x] Layer merge verified: deployed `kaya.projection` layer carries the six
      existing constraints plus `citation_markers`, `staleness_scope`, and
      `acknowledgment_effect`; no property is lost regardless of archive
      order.
- [x] Existing change `add-core-research-record` is not contradicted: its
      "Stale-authored flagging" requirement and this delta agree on the
      trigger and the clearing act.
- [x] kaya-7hm closed.