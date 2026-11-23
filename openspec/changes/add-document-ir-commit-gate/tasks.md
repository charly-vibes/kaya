# Tasks: add-document-ir-commit-gate

kaya-aaz delivers this change as a proposal; the tasks below are the
implementation plan for the tickets that follow. No production code ships
with this proposal.

## 1. Document IR (kaya-projection)

- [x] 1.1 Materialize the IR in the compiler: stable record ids, content
      hashes, event-id provenance, authored-block references (document,
      block, region hash, cited ids — no authored bytes copied), dependency
      edges; deterministic serialization for tests and a `kaya ir` debug
      view.
- [x] 1.2 Render from the IR: the emitted source map derives from record
      provenance (`ir.provenance_of` over bare and `<block>:<experiment>`
      keys); `kaya:cites` markers parse into IR dependencies through the
      report parser. Record-granular incremental invalidation lands with the
      impact gate (kaya-rp6), which owns the reverse index.
- [x] 1.3 Deploy the kaya-projection delta with its constraint and property
      rows in the spec tables; bind the scenarios in `.espectacular`. Bound:
      5 scenarios + 5 property mirrors. `structure-read-from-the-ir` stays
      unbound until its consumer exists (kaya-rp6).

## 2. Doctests (kaya-documents)

- [ ] 2.1 Doctest record type: runner, steps, expected output; extraction
      from declared contracts only; stable ids and content hashes.
- [ ] 2.2 Controlled runner: isolated workspace, timeout, expected-output
      comparison under a pinned normalization policy (deterministic
      expectations; volatile output normalized before comparison); results
      recorded as test-runner facts (kind `doctest`) so verification counts
      them under `test-runner`.
- [ ] 2.3 Supersede path for expected-output changes (human-recorded event;
      no run result ever rewrites expected output).
- [ ] 2.4 Deploy the kaya-documents delta with its property rows; bind the
      scenarios.

## 3. Impact gate (kaya-impact)

- [ ] 3.1 Impact mapping over IR dependency edges plus an explicit
      repository-maintained impact-rules policy file; rule attribution in
      the report.
- [ ] 3.2 Dispositions: closed vocabulary; `reviewed`/`waived` as
      verification-family events with human-actor refusal; stale-block
      integration with the scoped acknowledgment.
- [ ] 3.3 `kaya impact --staged|--range` report and exit contract;
      deterministic serialization without timestamps.
- [ ] 3.4 Deploy the kaya-impact spec and layer sidecar verbatim; bind the
      scenarios.

## 4. Wiring and docs

- [ ] 4.1 Lefthook wiring policy: staged local pass where cheap, full pass in
      `just ci`; via the global `~/.git-hooks` shim — never
      `lefthook install` locally.
- [ ] 4.2 `docs/src/document-taxonomy.md`: the Deferred section shrinks to
      the Org rejection with its revisit conditions, pointing at the
      deployed specs; book regenerated via `just docs`.

## 5. Verify (orchestrator)

- [ ] 5.1 `openspec validate --strict` and `ah check --run-tests` green;
      mutation check on kaya itself: change a documented command's output
      fixture and observe the gate fail, then add a disposition and observe
      it pass.
