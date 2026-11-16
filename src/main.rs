//! kaya — a living record of exploration.
//!
//! MVP-1 CLI skeleton: `kaya init` bootstraps the ledger and `kaya status`
//! reports its event count, size, and digest. Both print one genesis-style
//! JSON envelope to stdout; failures print an error envelope labeled with the
//! failure class (`usage`, `missing-ledger`, `ledger-io`, `not-a-file`).

use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Read};
use std::process::ExitCode;

use clap::error::ErrorKind as ClapErrorKind;
use clap::{Parser, Subcommand};
use kaya::envelope::Envelope;
use serde_json::json;
use sha2::{Digest, Sha256};

/// Project-local directory holding kaya state.
const KAYA_DIR: &str = ".kaya";

/// The append-only research ledger, relative to the working directory.
const LEDGER_PATH: &str = ".kaya/ledger.jsonl";

/// Size of the single buffer streamed over the ledger by `kaya status`.
const STATUS_BUFFER_BYTES: usize = 64 * 1024;

/// A failure class label plus a human-readable explanation.
type Failure = (&'static str, String);

#[derive(Debug, Parser)]
#[command(
    name = "kaya",
    version,
    about = "Compile a research record from the artifacts a project already produces"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Create the research ledger if it does not exist
    Init,
    /// Report the ledger's event count, size, and digest
    Status,
}

fn main() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => match err.kind() {
            ClapErrorKind::DisplayHelp | ClapErrorKind::DisplayVersion => err.exit(),
            _ => return emit(Envelope::error("error", "usage", err.to_string())),
        },
    };
    let outcome = match cli.command {
        Command::Init => cmd_init(),
        Command::Status => cmd_status(),
    };
    emit(outcome.unwrap_or_else(|(label, message)| Envelope::error("error", label, message)))
}

/// Print exactly one envelope and map it onto the process exit code.
fn emit(envelope: Envelope) -> ExitCode {
    let ok = envelope.ok;
    match envelope.print() {
        Ok(()) if ok => ExitCode::SUCCESS,
        Ok(()) => ExitCode::FAILURE,
        Err(message) => {
            eprintln!("kaya: {message}");
            ExitCode::FAILURE
        }
    }
}

fn cmd_init() -> Result<Envelope, Failure> {
    fs::create_dir_all(KAYA_DIR)
        .map_err(|err| ("ledger-io", format!("failed to create {KAYA_DIR}: {err}")))?;
    let created = match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(LEDGER_PATH)
    {
        Ok(_) => true,
        Err(err) if err.kind() == ErrorKind::AlreadyExists => {
            ensure_ledger_is_regular_file()?;
            false
        }
        Err(err) => {
            return Err((
                "ledger-io",
                format!("failed to create {LEDGER_PATH}: {err}"),
            ));
        }
    };
    Ok(Envelope::ok(
        "init",
        json!({ "ledger": LEDGER_PATH, "created": created }),
    ))
}

/// Reject a pre-existing path that shadows the ledger but is not a regular file.
fn ensure_ledger_is_regular_file() -> Result<(), Failure> {
    match fs::metadata(LEDGER_PATH) {
        Ok(metadata) if metadata.is_file() => Ok(()),
        Ok(_) => Err(not_a_file()),
        // A dangling symlink exists but resolves to no regular file.
        Err(err) if err.kind() == ErrorKind::NotFound => Err(not_a_file()),
        Err(err) => Err((
            "ledger-io",
            format!("failed to inspect {LEDGER_PATH}: {err}"),
        )),
    }
}

fn not_a_file() -> Failure {
    (
        "not-a-file",
        format!(
            "{LEDGER_PATH} exists but is not a regular ledger file; \
             move it aside and rerun `kaya init`"
        ),
    )
}

fn cmd_status() -> Result<Envelope, Failure> {
    let mut file = File::open(LEDGER_PATH).map_err(|err| match err.kind() {
        ErrorKind::NotFound => (
            "missing-ledger",
            format!("missing ledger {LEDGER_PATH}: run `kaya init` to create it"),
        ),
        _ => ("ledger-io", format!("failed to read {LEDGER_PATH}: {err}")),
    })?;

    // One streaming pass: digest, size, newline count, and tail byte together.
    let mut buffer = [0u8; STATUS_BUFFER_BYTES];
    let mut hasher = Sha256::new();
    let mut bytes: u64 = 0;
    let mut events: u64 = 0;
    let mut last_byte: Option<u8> = None;
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|err| ("ledger-io", format!("failed to read {LEDGER_PATH}: {err}")))?;
        if read == 0 {
            break;
        }
        let chunk = &buffer[..read];
        hasher.update(chunk);
        bytes += read as u64;
        events += chunk.iter().filter(|byte| **byte == b'\n').count() as u64;
        last_byte = Some(chunk[read - 1]);
    }

    let replay_integrity = match last_byte {
        None | Some(b'\n') => "ok",
        Some(_) => "recovered-truncated-tail",
    };
    Ok(Envelope::ok(
        "status",
        json!({
            "ledger": LEDGER_PATH,
            "events": events,
            "bytes": bytes,
            "digest": format!("{:x}", hasher.finalize()),
            "replay_integrity": replay_integrity,
        }),
    ))
}
