//! Command modules extracted from the single-file CLI entry.
//!
//! Responsibilities: hold the per-family command glue (capture, doctest,
//! impact, import, publication, status, verification) behind `pub(crate)`
//! entry functions `main.rs` dispatches to.
//!
//! Rationale: the clap argument tree stays in `main.rs`; each module carries
//! only the command bodies and their family-private helpers.

pub(crate) mod capture;
pub(crate) mod doctest;
pub(crate) mod feedback;
pub(crate) mod impact;
pub(crate) mod import;
pub(crate) mod publication;
pub(crate) mod status;
pub(crate) mod verification;
