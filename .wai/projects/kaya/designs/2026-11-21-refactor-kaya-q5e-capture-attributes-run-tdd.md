---
tags: [pipeline-run:tdd-ro5u-2026-11-21-kaya-q5e-capture-attributes, pipeline-step:refactor]
---

# REFACTOR kaya-q5e — capture attributes

Run: tdd-ro5u-2026-11-21-kaya-q5e-capture-attributes (step 5/9 refactor).

## Tidy applied (behavior-preserving)
`src/adapters/wai.rs` `capture_file`: removed the `identity_data.clone()` — the
event id is now computed directly from the freshly built identity map before the
optional `title`/`title_digest` keys are inserted into the same map. One `json!`
allocation + one id hash instead of build + clone + hash. Id semantics unchanged
(identity payload excludes content-derived attributes), verified byte-identical
behavior by the full suite and the id-stability pin test.

## Verification
- `cargo test`: 30/30 suites ok, 0 failures.
- `cargo clippy --all-targets -- -D warnings`: clean.

## No further refactor
Single-function scope, existing helpers reused (`sha256_hex`, `slug_for`); no
duplication introduced; naming matches module conventions (TITLE_MAX_LEN
documented, TITLE_DIGEST_LEN private like ID_DIGEST_LEN).

