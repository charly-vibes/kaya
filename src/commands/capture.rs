//! Capture commands: record one authored text as a single model event.
//!
//! Responsibilities: hold `cmd_capture`, the CLI glue shared by the note,
//! question, decision, and finding subcommands, opening the ledger and
//! delegating to the library capture runner.
//!
//! Rationale: the clap argument tree stays in `main.rs`; this module carries
//! only the command body so the CLI entry stays readable as the command
//! surface grows.

use crate::capture::Kind;
use crate::{CaptureArgs, Failure, open_ledger};
use kaya::envelope::Envelope;

/// Capture one authored text as a `model.entity.created` event.
pub(crate) fn cmd_capture(kind: Kind, text: &str, args: &CaptureArgs) -> Result<Envelope, Failure> {
    let mut ledger = open_ledger()?;
    crate::capture::run(
        &mut ledger,
        kind,
        text,
        &args.actor,
        &args.actor_type,
        args.key.as_deref(),
        &args.project,
    )
}
