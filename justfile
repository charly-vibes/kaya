# kaya — dev commands
#
# Same commands run locally and in CI for consistent diagnostics.
# Run `just` for the recipe list, `just ci` for the full pipeline.
# Tool versions are pinned in versions.ddl.toml.

set shell := ["bash", "-uc"]

# Default: list available recipes
default:
	@just --list

# === Build & Test ===

# Build the debug binary
build:
	cargo build

# Run the test suite
test:
	cargo test

# === Lint ===

# Check formatting (no changes)
fmt-check:
	cargo fmt --all --check

# Formatting gate, then clippy with warnings denied
lint: fmt-check
	cargo clippy --all-targets --all-features -- -D warnings

# === Docs ===

# Copy the deployed capability specs and their layers into the book
specs-docs:
	python3 scripts/sync-specs.py

# Build the mdBook and place the publication tree inside the artifact
docs: specs-docs
	mdbook build
	python3 scripts/build-publications.py

# Fail when the built book contains a relative link the artifact cannot resolve
links: docs
	python3 scripts/check-links.py

# Fail when status/README release claims contradict Cargo.toml
release-claims:
	python3 scripts/check-release-claims.py

# Full record gate: verify, claims, links, lag (mode warn|fail)
record-check mode="warn": record-verify release-claims links
	python3 scripts/record-lag.py --mode {{mode}} --bound 60

# Capture a scheduled snapshot: observe wai and git, then rebuild products.
# Serialized against concurrent captures via a lockfile; skipped when the
# tree is dirty (checked under the lock, so a queued capture re-checks),
# so a capture never builds over half-finished work (the Record workflow's
# checkout is always clean and is unaffected).
record-capture:
	#!/usr/bin/env bash
	set -euo pipefail
	mkdir -p .kaya
	exec 9>.kaya/capture.lock
	flock 9
	if [ -n "$(git status --porcelain)" ]; then
		echo "record-capture: SKIP - working tree is dirty; commit or stash first"
		exit 0
	fi
	cargo run --quiet -- import wai
	cargo run --quiet -- import git --exclude-message-prefix "chore(record):"
	cargo run --quiet -- build
	cargo run --quiet -- publish

# === Spec gates ===

# Verify the committed record against its projections without writing
record-verify:
	cargo run --quiet -- check

# Fail when the record's capture frontier is more than `bound` commits behind
# (nightly-only capture makes a day of lag normal; 60 covers the busiest
# measured day, 42 record-relevant commits on 2026-11-24)
record-lag mode="fail" bound="60":
	python3 scripts/record-lag.py --mode {{mode}} --bound {{bound}}

# Validate spec ↔ test correspondence (espectacular, bin: ah, run through
# scripts/ah-check so PATH resolves the repo-built kaya, not a stale install)
validate:
	scripts/ah-check check

# Lint the spec corpus (specodelic, bin: spk). Exit 2 means "no specs
# found" — an empty corpus is not a violation.
spec-lint:
	spk lint openspec/specs || test $? -eq 2

# Validate active OpenSpec changes strictly
spec-validate:
	openspec validate --changes --strict

# === CI ===

# Full CI pipeline — the same command ci.yml runs
ci: test lint docs validate spec-lint spec-validate impact-gate
	@echo "✅ CI pipeline passed"

# Full impact pass: gate the committed change against origin/main
impact-gate:
	cargo run --quiet -- impact --range origin/main

# === Session ===

# Show workspace status (wai)
status:
	wai status

# Check workspace health (wai)
doctor:
	wai doctor

# Orient at session start (wai)
prime:
	wai prime

# Show ready-to-work beads issues
ready:
	bd ready