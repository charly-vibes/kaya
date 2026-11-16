//! kaya — a living record of exploration.
//!
//! The MVP-1 CLI: `init` bootstraps the ledger, `status` reports its shape and
//! pending proposals, `import` pulls observations from a source adapter,
//! `build` compiles the docs projections, and `promote`/`challenge` append
//! human promotion events. Every invocation prints exactly one genesis-style
//! JSON envelope to stdout; failures print an error envelope labeled with the
//! failure class — a CLI class (`usage`, `missing-ledger`, `ledger-io`,
//! `build-io`, `not-a-file`) or the label raised by the library that refused
//! the run.

use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Read};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::error::ErrorKind as ClapErrorKind;
use clap::{Parser, Subcommand};
use serde_json::json;
use sha2::{Digest, Sha256};

use kaya::adapters::{AdapterError, git, testrunner, wai};
use kaya::envelope::Envelope;
use kaya::event::{Actor, ActorType, Event, parse_line};
use kaya::ledger::{Anchor, AppendOutcome, Ledger, LedgerError};
use kaya::model::{self, EntityType, ModelError};
use kaya::projection::{ProjectionError, StaleBlock, Summary, index, report};
use kaya::promotion::{self, PromotionError, PromotionState};

/// Project-local directory holding kaya state.
const KAYA_DIR: &str = ".kaya";

/// The append-only research ledger, relative to the working directory.
const LEDGER_PATH: &str = ".kaya/ledger.jsonl";

/// Size of the single buffer streamed over the ledger by `kaya status`.
const STATUS_BUFFER_BYTES: usize = 64 * 1024;

/// A failure class label plus a human-readable explanation.
type Failure = (String, String);

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
    /// Report the ledger's event count, size, digest, and pending proposals
    Status,
    /// Capture observations from a source adapter into the ledger
    Import {
        #[command(subcommand)]
        source: ImportSource,
    },
    /// Compile the research index and experiment reports
    Build {
        /// Directory receiving the compiled markdown
        #[arg(long, default_value = "docs/research", value_name = "DIR")]
        out: PathBuf,
        /// Refuse to build unless the ledger's digest is exactly this hex value
        #[arg(long, value_name = "HEX")]
        expect_digest: Option<String>,
    },
    /// Approve a proposed assertion as a conclusion
    Promote {
        /// Id of the recorded assertion to establish
        assertion_id: String,
        /// Name of the approving human
        #[arg(long, value_name = "NAME")]
        actor: String,
    },
    /// Challenge an established conclusion with counter-events
    Challenge {
        /// Id of the established conclusion to challenge
        conclusion_id: String,
        /// Counter-event ids justifying the challenge
        #[arg(long = "events", required = true, num_args = 1.., value_name = "ID")]
        events: Vec<String>,
        /// Name of the challenging human
        #[arg(long, value_name = "NAME")]
        actor: String,
    },
}

#[derive(Debug, Subcommand)]
enum ImportSource {
    /// Import observed artifacts from the `.wai/` artifact tree
    Wai {
        /// Directory containing the `.wai/` tree
        #[arg(long, default_value = ".", value_name = "DIR")]
        root: PathBuf,
        /// Project namespace prefixed to subject ids
        #[arg(long, default_value = "kaya", value_name = "NAME")]
        project: String,
    },
    /// Import observed commits from repository history
    Git {
        /// Repository root to read history from
        #[arg(long, default_value = ".", value_name = "DIR")]
        root: PathBuf,
        /// Project namespace prefixed to subject ids
        #[arg(long, default_value = "kaya", value_name = "NAME")]
        project: String,
        /// Maximum number of recent commits to observe
        #[arg(long, default_value_t = 100, value_name = "N")]
        limit: usize,
    },
    /// Import one completed test run from a libtest JSONL report
    Test {
        /// Path to the machine-readable run report
        #[arg(long, value_name = "PATH")]
        report: PathBuf,
        /// Project namespace prefixed to subject ids
        #[arg(long, default_value = "kaya", value_name = "NAME")]
        project: String,
    },
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
        Command::Import { source } => cmd_import(source),
        Command::Build { out, expect_digest } => cmd_build(&out, expect_digest.as_deref()),
        Command::Promote {
            assertion_id,
            actor,
        } => cmd_promote(&assertion_id, &actor),
        Command::Challenge {
            conclusion_id,
            events,
            actor,
        } => cmd_challenge(&conclusion_id, &events, &actor),
    };
    emit(outcome.unwrap_or_else(|(label, message)| Envelope::error("error", &label, message)))
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

fn failure(label: impl Into<String>, message: impl Into<String>) -> Failure {
    (label.into(), message.into())
}

fn cmd_init() -> Result<Envelope, Failure> {
    fs::create_dir_all(KAYA_DIR)
        .map_err(|err| failure("ledger-io", format!("failed to create {KAYA_DIR}: {err}")))?;
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
            return Err(failure(
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
        Err(err) => Err(failure(
            "ledger-io",
            format!("failed to inspect {LEDGER_PATH}: {err}"),
        )),
    }
}

fn not_a_file() -> Failure {
    failure(
        "not-a-file",
        format!(
            "{LEDGER_PATH} exists but is not a regular ledger file; \
             move it aside and rerun `kaya init`"
        ),
    )
}

/// Map a library ledger refusal onto the CLI failure classes.
///
/// A missing ledger is the one class the CLI renames: `kaya.ledger.missing`
/// becomes `missing-ledger`, matching `status`, and the message already
/// points at `kaya init`.
fn ledger_failure(err: LedgerError) -> Failure {
    let label = if err.label == "kaya.ledger.missing" {
        "missing-ledger".to_string()
    } else {
        err.label
    };
    (label, err.message)
}

fn adapter_failure(err: AdapterError) -> Failure {
    (err.label, err.message)
}

fn model_failure(err: ModelError) -> Failure {
    (err.label, err.message)
}

fn projection_failure(err: ProjectionError) -> Failure {
    (err.label, err.message)
}

fn promotion_failure(err: PromotionError) -> Failure {
    (err.label, err.message)
}

/// Open the existing ledger, refusing an absent one with `missing-ledger`.
fn open_ledger() -> Result<Ledger, Failure> {
    Ledger::open(LEDGER_PATH).map_err(ledger_failure)
}

/// A human actor named on the command line.
///
/// The CLI has no way to name a tool or agent actor: promotion commands are
/// the one human door into the record.
fn human_actor(name: &str) -> Actor {
    Actor {
        r#type: ActorType::Human,
        name: name.to_owned(),
    }
}

fn cmd_status() -> Result<Envelope, Failure> {
    let mut file = File::open(LEDGER_PATH).map_err(|err| match err.kind() {
        ErrorKind::NotFound => failure(
            "missing-ledger",
            format!("missing ledger {LEDGER_PATH}: run `kaya init` to create it"),
        ),
        _ => failure("ledger-io", format!("failed to read {LEDGER_PATH}: {err}")),
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
            .map_err(|err| failure("ledger-io", format!("failed to read {LEDGER_PATH}: {err}")))?;
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
            "pending_proposals": pending_proposals()?,
        }),
    ))
}

/// Count assertions awaiting human approval.
///
/// Status stays a cheap, non-failing read: lines that do not parse as
/// `kaya.event/v0` records are skipped here and reported by the commands
/// that actually replay the ledger. Only when a well-formed promotion record
/// contradicts the promotion rules does status surface the refusal.
fn pending_proposals() -> Result<usize, Failure> {
    let bytes = match fs::read(LEDGER_PATH) {
        Ok(bytes) => bytes,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(0),
        Err(err) => {
            return Err(failure(
                "ledger-io",
                format!("failed to read {LEDGER_PATH}: {err}"),
            ));
        }
    };
    let mut events = Vec::new();
    for line in bytes.split(|byte| *byte == b'\n') {
        let Ok(text) = std::str::from_utf8(line) else {
            continue;
        };
        if let Ok(event) = parse_line(text) {
            events.push(event);
        }
    }
    PromotionState::from_events(&events)
        .map(|state| state.pending_count())
        .map_err(promotion_failure)
}

fn cmd_import(source: ImportSource) -> Result<Envelope, Failure> {
    let mut ledger = open_ledger()?;
    let (source_name, events) = match source {
        ImportSource::Wai { root, project } => (
            "wai",
            wai::capture(&root, &project).map_err(adapter_failure)?,
        ),
        ImportSource::Git {
            root,
            project,
            limit,
        } => (
            "git",
            git::capture(&root, &project, limit).map_err(adapter_failure)?,
        ),
        ImportSource::Test { report, project } => (
            "test-runner",
            vec![testrunner::capture(&report, &project).map_err(adapter_failure)?],
        ),
    };

    let mut appended = 0usize;
    let mut duplicates = 0usize;
    for event in &events {
        match ledger.append(event).map_err(ledger_failure)? {
            AppendOutcome::Appended => appended += 1,
            AppendOutcome::Duplicate => duplicates += 1,
        }
    }

    Ok(Envelope::ok(
        "import",
        json!({
            "source": source_name,
            "events": events.len(),
            "appended": appended,
            "duplicates": duplicates,
            "ledger": LEDGER_PATH,
        }),
    ))
}

fn cmd_build(out: &Path, expect_digest: Option<&str>) -> Result<Envelope, Failure> {
    let ledger = open_ledger()?;
    let (events, ledger_digest) = read_build_ledger(&ledger, expect_digest)?;

    let graph = model::build(&events).map_err(model_failure)?;
    let state = PromotionState::from_events(&events).map_err(promotion_failure)?;

    fs::create_dir_all(out).map_err(|err| {
        failure(
            "build-io",
            format!("failed to create {}: {err}", out.display()),
        )
    })?;
    let mut summary = index::build(&graph, &out.join("index.md")).map_err(projection_failure)?;

    let experiments: Vec<String> = graph
        .nodes()
        .filter(|node| node.entity == EntityType::Experiment)
        .map(|node| node.id.clone())
        .collect();
    if !experiments.is_empty() {
        let experiments_dir = out.join("experiments");
        fs::create_dir_all(&experiments_dir).map_err(|err| {
            failure(
                "build-io",
                format!("failed to create {}: {err}", experiments_dir.display()),
            )
        })?;
        for experiment_id in experiments {
            let key = experiment_id
                .rsplit('/')
                .next()
                .unwrap_or(experiment_id.as_str());
            let report_summary = report::build(
                &graph,
                &experiment_id,
                &experiments_dir.join(format!("{key}.md")),
                &state,
            )
            .map_err(projection_failure)?;
            merge_summary(&mut summary, report_summary);
        }
    }

    let mut recomputed = summary.recomputed;
    recomputed.sort();
    recomputed.dedup();
    let mut unchanged = summary.unchanged;
    unchanged.sort();
    unchanged.dedup();
    let mut stale: Vec<StaleBlock> = summary.stale;
    stale.sort_by(|left, right| {
        left.block
            .cmp(&right.block)
            .then_with(|| left.evidence.cmp(&right.evidence))
    });
    stale.dedup();
    let stale: Vec<serde_json::Value> = stale
        .into_iter()
        .map(|block| json!({ "block": block.block, "evidence": block.evidence }))
        .collect();

    Ok(Envelope::ok(
        "build",
        json!({
            "out": out.display().to_string(),
            "recomputed": recomputed,
            "unchanged": unchanged,
            "stale": stale,
            "pending_proposals": state.pending_count(),
            "ledger_digest": ledger_digest,
        }),
    ))
}

/// Read the ledger for a build, optionally pinned to an expected digest.
///
/// The bootstrap path repairs a crash-truncated tail; when the caller pins a
/// digest, the anchor is built from the recovered bytes so the verified read
/// refuses any ledger whose bytes differ from what the caller recorded.
fn read_build_ledger(
    ledger: &Ledger,
    expect_digest: Option<&str>,
) -> Result<(Vec<Event>, String), Failure> {
    let (events, _recovery) = ledger.read_recovered().map_err(ledger_failure)?;
    let bytes = fs::read(LEDGER_PATH)
        .map_err(|err| failure("ledger-io", format!("failed to read {LEDGER_PATH}: {err}")))?;
    let anchor = Anchor::of(&bytes);

    let Some(expected) = expect_digest else {
        return Ok((events, anchor.digest));
    };
    if anchor.digest != expected {
        return Err(failure(
            "kaya.ledger.digest_mismatch",
            format!(
                "ledger {LEDGER_PATH} does not match the expected digest {expected}: the recovered bytes digest {}",
                anchor.digest
            ),
        ));
    }
    let (verified, _recovery) = ledger.read_verified(&anchor).map_err(ledger_failure)?;
    Ok((verified, anchor.digest))
}

fn merge_summary(into: &mut Summary, from: Summary) {
    into.recomputed.extend(from.recomputed);
    into.unchanged.extend(from.unchanged);
    into.stale.extend(from.stale);
}

fn cmd_promote(assertion_id: &str, actor_name: &str) -> Result<Envelope, Failure> {
    let mut ledger = open_ledger()?;
    let (events, _recovery) = ledger.read_recovered().map_err(ledger_failure)?;
    let approval =
        promotion::approve(&human_actor(actor_name), assertion_id).map_err(promotion_failure)?;

    // Replaying the record with the candidate event is what refuses an
    // approval of an assertion that was never recorded: the projection, not
    // the CLI, owns the promotion rules.
    let mut replayed = events;
    replayed.push(approval.clone());
    PromotionState::from_events(&replayed).map_err(promotion_failure)?;

    ledger.append(&approval).map_err(ledger_failure)?;
    Ok(Envelope::ok("promote", json!({ "event": approval })))
}

fn cmd_challenge(
    conclusion_id: &str,
    counter_event_ids: &[String],
    actor_name: &str,
) -> Result<Envelope, Failure> {
    let mut ledger = open_ledger()?;
    let (events, _recovery) = ledger.read_recovered().map_err(ledger_failure)?;
    let challenge =
        promotion::challenge(&human_actor(actor_name), conclusion_id, counter_event_ids)
            .map_err(promotion_failure)?;

    // Replay proves the target is an established conclusion before the
    // challenge is written; the projection carries the refusal label.
    let mut replayed = events;
    replayed.push(challenge.clone());
    PromotionState::from_events(&replayed).map_err(promotion_failure)?;

    ledger.append(&challenge).map_err(ledger_failure)?;
    Ok(Envelope::ok("challenge", json!({ "event": challenge })))
}
