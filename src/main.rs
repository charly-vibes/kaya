//! kaya — a living record of exploration.
//!
//! MVP-1 CLI skeleton: `kaya init` bootstraps the ledger and `kaya status`
//! reports its event count, size, and digest. Both print one genesis-style
//! JSON envelope to stdout.

use std::fs::{self, OpenOptions};
use std::io::ErrorKind;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use kaya::envelope::Envelope;
use serde_json::json;
use sha2::{Digest, Sha256};

/// Project-local directory holding kaya state.
const KAYA_DIR: &str = ".kaya";

/// The append-only research ledger, relative to the working directory.
const LEDGER_PATH: &str = ".kaya/ledger.jsonl";

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
    let cli = Cli::parse();
    let (error_label, outcome) = match cli.command {
        Command::Init => ("init", cmd_init()),
        Command::Status => ("status", cmd_status()),
    };
    let envelope = match outcome {
        Ok(envelope) => envelope,
        Err(message) => Envelope::error("error", error_label, message),
    };
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

fn cmd_init() -> Result<Envelope, String> {
    fs::create_dir_all(KAYA_DIR).map_err(|err| format!("failed to create {KAYA_DIR}: {err}"))?;
    let created = match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(LEDGER_PATH)
    {
        Ok(_) => true,
        Err(err) if err.kind() == ErrorKind::AlreadyExists => false,
        Err(err) => return Err(format!("failed to create {LEDGER_PATH}: {err}")),
    };
    Ok(Envelope::ok(
        "init",
        json!({ "ledger": LEDGER_PATH, "created": created }),
    ))
}

fn cmd_status() -> Result<Envelope, String> {
    let bytes = fs::read(LEDGER_PATH).map_err(|err| match err.kind() {
        ErrorKind::NotFound => {
            format!("missing ledger {LEDGER_PATH}: run `kaya init` to create it")
        }
        _ => format!("failed to read {LEDGER_PATH}: {err}"),
    })?;
    let events = bytes.iter().filter(|byte| **byte == b'\n').count();
    let replay_integrity = if bytes.is_empty() || bytes.ends_with(b"\n") {
        "ok"
    } else {
        "recovered-truncated-tail"
    };
    Ok(Envelope::ok(
        "status",
        json!({
            "ledger": LEDGER_PATH,
            "events": events,
            "bytes": bytes.len(),
            "digest": sha256_hex(&bytes),
            "replay_integrity": replay_integrity,
        }),
    ))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!("{digest:x}")
}
