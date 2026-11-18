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

# === Spec gates ===

# Verify the committed record against its projections without writing
record-verify:
	cargo run --quiet -- check

# Fail when the record's capture frontier is more than `bound` commits behind
record-lag mode="fail" bound="25":
	python3 scripts/record-lag.py --mode {{mode}} --bound {{bound}}

# Validate spec ↔ test correspondence (espectacular, bin: ah)
validate:
	ah check

# Lint the spec corpus (specodelic, bin: spk). Exit 2 means "no specs
# found" — an empty corpus is not a violation.
spec-lint:
	spk lint openspec/specs || test $? -eq 2

# Validate active OpenSpec changes strictly
spec-validate:
	openspec validate --changes --strict

# === CI ===

# Full CI pipeline — the same command ci.yml runs
ci: test lint docs validate spec-lint spec-validate
	@echo "✅ CI pipeline passed"

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