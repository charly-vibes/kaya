# Tasks: add-finding-source-link

Red→green per the kaya-4gv MVP-2 hook; each scenario below is a
contract-test shape bound in `.espectacular`.

## 1. Red — failing tests first (kaya-4gv)

- [ ] 1.1 `grounds_relationship_joins_the_closed_set` — a
      `model.relationship.asserted` "grounds" Finding→Source event builds
      an edge.
- [ ] 1.2 `unknown_relationship_refusal_names_grounds_in_the_closed_set` —
      the relationship-set refusal names `grounds` alongside the existing
      ten.
- [ ] 1.3 `grounds_direction_is_finding_to_source_exact_pair` — reversed
      and wrong-entity pairs refused, label `kaya.model.direction_fixed`,
      rule text `Finding -> Source`.
- [ ] 1.4 `grounds_to_an_unknown_node_is_refused_as_dangling` — a
      `grounds` edge to an absent node refuses with label
      `kaya.model.references_resolve` naming the endpoint.

## 2. Green — make the tests pass

- [ ] 2.1 `src/model.rs`: `Relationship::Grounds` in the enum, `ALL`
      10→11, `as_str` "grounds", `fixed_direction` Finding→Source,
      `requires_exact_pair`, `RELATIONSHIP_SET` gains `, grounds`; module
      doc header amended (attribution via `cites` **and** grounding via
      `grounds`); enum doc comment amended (attribution via `cites` and
      grounding via `grounds`); `requires_exact_pair` doc comment "five
      structural relationships" → six.
- [ ] 2.2 Bind the 7 new scenarios in the change overlay and deploy
      bindings under `.espectacular/kaya-model/` (7 new tomls); the 4
      mirror scenarios bind to the §1 test fns
      `grounds_relationship_joins_the_closed_set`,
      `unknown_relationship_refusal_names_grounds_in_the_closed_set`,
      `grounds_direction_is_finding_to_source_exact_pair`,
      `grounds_to_an_unknown_node_is_refused_as_dangling`.

## 3. Verify (orchestrator)

- [ ] 3.1 `openspec validate --changes --strict` — pass.
- [ ] 3.2 `cargo test --test model` green; full `cargo test` ok.
- [ ] 3.3 `ah check --changes add-finding-source-link --run-tests` — 0
      findings.
