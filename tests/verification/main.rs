//! Integration tests for `kaya check`, the read-only record verifier.
//!
//! Every test builds a disposable repository, imports the committed model
//! seed, commits the ledger, compiles the record with `kaya build` and
//! `kaya publish`, and commits the generated documents. One artifact is then
//! tampered with and `check` must report the drift from the worktree without
//! writing: a changed block body is `changed`, an absent generated document is
//! `missing`, a managed document the projection does not emit is `extra`, a
//! tampered publication page is one `changed` block finding, an absent
//! publication export is a `missing` document, a worktree ledger that no
//! longer extends `HEAD` is refused before any finding, and the exit code
//! carries the verdict (0 clean, 1 drift, 2 usage or I/O).
//!
//! The `--index` fixtures do the same over the git index: the ledger and every
//! compared document are read from `:<path>`, so staged drift is reported even
//! when the worktree is clean, worktree noise is ignored, a staged ledger in a
//! repository whose `HEAD` predates it is verifiable, and an unmerged index
//! entry is an I/O failure rather than a worktree fallback. The stale report
//! follows the same rule: a staged correction and its citing document produce
//! one finding naming the root-relative path, while a citation that lives only
//! in the worktree is invisible to `--index`.
//!
//! The opt-in flags are proven the same way: `--require-kind` and
//! `--require-source` turn an omitted dotted kind or an absent canonical
//! source into a named `data.required` violation (exit 1) and refuse
//! vocabulary they do not know (exit 2), while `--fail-on-stale-authored`
//! turns the projection's stale report into a violation naming the authored
//! block and the evidence ids a correction changed. Without that flag a stale
//! block is advisory: `check` reports it and exits 0, and `build` keeps
//! reporting it without failing.
//!
//! The fixtures follow `tests/acceptance.rs` and `tests/cli.rs`: the built
//! binary runs in a `tempfile::TempDir`, envelopes are parsed as JSON, and
//! commits use a fixed identity and dates.

mod common;
mod drift;
mod facts;
mod facts_fixture;
mod index_mode;
mod required;
mod snapshot;
mod sources;
mod stale;
mod tampering;
