#!/usr/bin/env bash
# Purpose: Pipeline gate oracle that executes kaya's test suite.
# Responsibilities: run `cargo test --quiet` from the repository root and
#   propagate its exit status. Accepts artifact paths per the oracle contract
#   (scope = "all") but ignores them: the check is repository state, not
#   artifact prose.
# Rationale: wai's stock tests-pass oracle only greps the artifact for
#   evidence language. kaya tickets must be proven by executed tests, so the
#   gate runs the suite instead of trusting a written claim.

set -euo pipefail

if [[ ! -f Cargo.toml ]]; then
    echo "tests-pass: Cargo.toml not found in $(pwd) — scaffold the crate first (kaya-lb2)" >&2
    exit 1
fi

cargo test --quiet