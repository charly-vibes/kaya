RO5U kaya-18o review (reviewer, CONVERGED stage 4, verdict NEEDS_REVISION): C:0 H:0 M:2 L:3; mechanical Stage A pre-gate passed for all findings; below-VERIFY severities self-reported (banner).
[CORR-002][MEDIUM] Collection arm skipped stray-fields refusal (includes/members silently ignored on Decl, public API). FIXED: plan.rs:177-195 refuses stray includes then members via tier_fields BEFORE the unimplemented check, precedence documented.
[CORR-001][MEDIUM] tier_unimplemented message factually false after narrowing. FIXED: error.rs:117-118 "only the glossary collection is implemented..."; guard-test assertion message updated.
[CLAR-001][LOW] relationship bullets string-split RELATIONSHIP_SET. FIXED: iterate Relationship::ALL + as_str; RELATIONSHIP_SET re-privatized; only ENTITY_TYPE_SET stays pub(crate).
[CLAR-002][LOW] stale test headers. FIXED: build.rs:224/:252.
[EDGE-001][LOW] failure message omitted architecture-design. FIXED: cli_pipeline.rs:2884.
Verified-clean: refusal-before-write ordering; consumer routing end to end (render_pages -> plan_pages block vocabulary -> cmd_publish pages -> check parity); glossary body compile-time-only (no IO); build/publish separation intact (reverted pass fully gone); plan_removals product-only; export row shape sound; edge cases (duplicate decl, tier mismatch, refusal-before-write) all handled.
