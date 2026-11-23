# RO5U kaya-yvr — document IR materialization (diff of 25a198b + ccc41d7)

Scope: src/projection/ir.rs, projections wiring (index/report/reuse), cmd_ir, tests/document_ir.rs, deployed spec + bindings.

## Critical
- [FIXED in ccc41d7] src/main.rs IrCollector::collect — created a labeled Failure for managed-block grammar violations and then discarded it via unwrap_or_default, silently emitting an empty rendered list: a corrupted docs/research source produced a wrong IR instead of a refusal. Fix: collect -> Result<(), Failure>, all five call sites propagate; doc pins "the IR never silently drops a document's records".

## High
- none.

## Medium
- ir::materialize panics on precondition violation (grammar-invalid sources, out-of-vocabulary block names). Accepted: documented invariant; every caller runs the plan gates (validate_generated_blocks / classify_block) before materialize; a panic here is a programmer-error signal, not input handling.

## Low
- ir::provenance_of unions bare and `<block>:<experiment>` namespaced keys; the closed block vocabulary guarantees no name prefixes another, so the `"{block}:"` match is unambiguous.
- materialize silently overwrites records on duplicate input paths; all callers are single-writer per document.

## Stage checks
- Draft: shape good — IR isolated in one module, wiring minimal, tests map 1:1 to deployed scenarios.
- Correctness: 1 critical found and fixed (above); 0 remaining.
- Clarity: module docs state the derivation contract; for_document's in-place summary canonicalization is documented.
- Edge cases: reuse None documents scan the empty string and produce no records (verified); cache independence is exercised through the CLI.
- Excellence: 7/7 tests map to deployed scenarios; allocation bounded (one clone per record body); IO: each source read once.

Verdict: PASS — 0 criticals remaining. Post-fix gates: clippy -D warnings 0, cargo test all green, just ci passed, ah check clean.
