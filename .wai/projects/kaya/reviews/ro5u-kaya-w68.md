# RO5U kaya-w68 — brief.md joins the read lists (diff of working tree, 2026-11-25)

Scope: src/publication/source.rs, src/verification/mod.rs, src/explain.rs,
src/publication/decls.rs, tests/explain.rs, tests/cli_pipeline.rs,
docs/public (regenerated), docs/research/brief.md.

Five-stage review by reviewer subagent; converged at stage 4. Stages:
Draft EXCELLENT · Correctness EXCELLENT · Clarity GOOD · Edge EXCELLENT.

## Critical
- none. CRITICAL_COUNT: 0 — gate passes.

## High
- none.

## Medium
- [FIXED] tests/explain.rs:175-177 — stale comment contradicted the flipped
  assertions (said tutorial "must read omitted"); rewritten to the post-w68
  reality (emitted from named view and census alike).
- [FIXED] src/publication/decls.rs:30-33 — module doc claimed tutorial
  publishes as a no-backing-document omission until exploration documents
  carry records; now states a populated exploration brief backs tutorial from
  a question-only ledger.

## Low
- [FIXED] src/publication/source.rs:68-70 — `read_documents` docstring's
  fixed-order enumeration omitted brief.md; enumeration now matches the array.

## Verified (evidence)
- Admission contract: openspec/specs/kaya-publications/spec.md:108-112 pins
  tutorial ← exploration.experiment-record + exploration.exploration-brief;
  src/publication/vocabulary.rs:158-165 implements exactly that. No widening.
- Mirror exactness: render_brief (src/projection/report.rs:863-880) returns
  None iff Questions/Ideas/Hypotheses all empty — exactly
  populated("exploration.exploration-brief") (src/explain.rs:409-414).
- Parity: verification/mod.rs:1146-1162 compares READER_FILES to
  RESEARCH_READ_ORDER; both grew 6→7 with brief.md at the same position;
  check's publication_inputs walks the same list.
- Consumer dispatch: read_documents loops generically; collect_documents
  filters by DOCUMENT_KINDS (∋ exploration.exploration-brief); kind comes
  from the block name, not the file (tests/publications/reader.rs:26).
- New e2e test sound: cmd_publish (src/main.rs:1382-1387) reads only
  docs/research (no ledger guard), so bare publish needs no
  --allow-uncommitted; tutorial page on a question-only ledger pins exactly
  the w68 behavior (tutorial admits only exploration kinds).
- Ordering inert: build writes one family per file; first-wins-across-files
  never competes; whitespace-only blocks already skip.
- All eight cli_pipeline expectation sites stay name-sorted
  (topic-map < tutorial) and omission-consistent.
- Dogfood: docs/research/brief.md cites kaya/Question/mvp-1-acceptance and
  kaya/Hypothesis/H-001; regenerated publication.json attributes exactly
  those ids; docs/public/tutorial.md composes the brief body.
- No leftover caveat text anywhere (grep over src/tests/docs/README clean).

Post-review-fix gates: cargo test 502 passed / 0 failed; cargo fmt clean;
cargo clippy --all-targets --all-features -D warnings clean.

Verdict: Production Ready WITH_NOTES (notes applied). Zero Critical.
