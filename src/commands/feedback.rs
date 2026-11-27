//! Feedback command: report issues to the kaya repository tracker.
//!
//! Responsibilities: hold `cmd_feedback`, the additive `kaya feedback`
//! subcommand that delegates to `genesis::feedback::handle_feedback` — kind
//! validation with typo suggestions, stdin/scratch body assembly, home-path
//! and target-repo redaction, dry-run previews, and gh filing.
//!
//! Rationale: all issue-assembly machinery lives in genesis; this module
//! only maps the genesis outcome onto kaya's envelope dialect and failure
//! labels.

use genesis::feedback::gh::GhResult;
use genesis::feedback::{FeedbackArgs, handle_feedback};
use serde_json::{Value, json};

use kaya::envelope::Envelope;

use crate::Failure;

pub(crate) fn cmd_feedback(
    kind: String,
    title: Option<String>,
    dry_run: bool,
    from_last_error: bool,
) -> Result<Envelope, Failure> {
    let mut args = FeedbackArgs::new(kind, dry_run, from_last_error);
    if let Some(title) = title {
        args = args.with_title(title);
    }
    match handle_feedback(
        &args,
        "kaya",
        env!("CARGO_PKG_VERSION"),
        "charly-vibes/kaya",
        &std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
    ) {
        Ok(result) => Ok(Envelope::ok("feedback", outcome(result))),
        Err(message) => Err(crate::failure("kaya.feedback", message)),
    }
}

/// Map the genesis filing outcome onto the envelope's `data` payload.
fn outcome(result: GhResult) -> Value {
    match result {
        GhResult::Created { url, number } => {
            json!({ "outcome": "created", "url": url, "number": number })
        }
        GhResult::FallbackUrl(url) => json!({ "outcome": "fallback", "url": url }),
        GhResult::LocalFile(path) => json!({ "outcome": "local-file", "path": path }),
    }
}
