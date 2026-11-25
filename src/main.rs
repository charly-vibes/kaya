//! kaya — a living record of exploration.
//!
//! The MVP-1 CLI: `init` bootstraps the ledger, `status` reports its shape and
//! pending proposals, `import` pulls observations from a source adapter or
//! model events from a seed file, `build` compiles the docs projections, and
//! `promote`/`challenge` append human promotion events, while `acknowledge`
//! records that a human reviewed an authored block's changed evidence. Every
//! invocation prints exactly one genesis-style JSON envelope to stdout;
//! failures print an error envelope labeled with the failure class — a CLI
//! class (`usage`, `missing-ledger`, `ledger-io`, `build-io`, `not-a-file`) or
//! the label raised by the library that refused the run.

use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::{Command as ProcessCommand, ExitCode};

use clap::error::ErrorKind as ClapErrorKind;
use clap::{Parser, Subcommand};
use kaya::ledger::{Ledger, LedgerError, sha256_hex};

use kaya::adapters::AdapterError;

use kaya::envelope::Envelope;
use kaya::event::{Actor, ActorType, Event, parse_line};
use kaya::model::{self, EntityType, ModelError};
use kaya::projection::{BlockKind, ProjectionError, Summary, index, ir, report, reuse, scan};
use kaya::promotion::{PromotionError, PromotionState};

mod capture;
mod commands;
mod explain;
use kaya::verification;

/// Project-local directory holding kaya state.
pub(crate) const KAYA_DIR: &str = ".kaya";

/// The append-only research ledger, relative to the working directory.
pub(crate) const LEDGER_PATH: &str = ".kaya/ledger.jsonl";

/// A failure class label plus a human-readable explanation.
pub(crate) type Failure = (String, String);

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

/// The argument set shared by every capture subcommand; the text is the
/// subcommand's positional and `capture::Kind` its only real difference.
#[derive(Debug, clap::Args)]
pub(crate) struct CaptureArgs {
    /// Name of the author
    #[arg(long, value_name = "NAME")]
    pub(crate) actor: String,
    /// Actor kind: `human` (the default) or `agent`
    #[arg(long = "actor-type", default_value = "human", value_name = "TYPE")]
    pub(crate) actor_type: String,
    /// Node key; defaults to `<UTC date>-<sha256(text)[:8]>`
    #[arg(long, value_name = "SLUG")]
    pub(crate) key: Option<String>,
    /// Project namespace recorded in the subject id
    #[arg(long, default_value = "kaya", value_name = "NAME")]
    pub(crate) project: String,
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
        /// Accept an untracked ledger whose history git cannot verify
        #[arg(long)]
        allow_uncommitted: bool,
        /// Count render constructor invocations into data.render_invocations
        #[arg(long)]
        instrument: bool,
    },
    /// Print the document IR of the record without writing anything
    Ir {
        /// Research projection directory read for its document sources
        #[arg(long, default_value = "docs/research", value_name = "DIR")]
        out: PathBuf,
        /// Refuse to materialize unless the ledger's digest is exactly this hex value
        #[arg(long, value_name = "HEX")]
        expect_digest: Option<String>,
        /// Accept an untracked ledger whose history git cannot verify
        #[arg(long)]
        allow_uncommitted: bool,
        /// Walk each generated block beside its resolved provenance and human trail
        #[arg(long)]
        walk: bool,
    },
    /// Compose the client-facing publication tree from the compiled research
    Publish {
        /// Directory receiving the published pages and export
        #[arg(long, default_value = "docs/public", value_name = "DIR")]
        out: PathBuf,
    },
    /// Verify the committed record against the projections without writing
    Check {
        /// Repository root holding the ledger and both projection trees
        #[arg(long, default_value = ".", value_name = "DIR")]
        root: PathBuf,
        /// Research projection directory, relative to the root
        #[arg(long, default_value = "docs/research", value_name = "DIR")]
        out: PathBuf,
        /// Publication projection directory, relative to the root
        #[arg(long, default_value = "docs/public", value_name = "DIR")]
        publish_out: PathBuf,
        /// Refuse to check unless the ledger's digest is exactly this hex value
        #[arg(long, value_name = "HEX")]
        expect_digest: Option<String>,
        /// Read the ledger and projections from the git index, not the worktree
        #[arg(long)]
        index: bool,
        /// Require the record to emit this dotted `family.kind`, repeatable
        #[arg(long = "require-kind", value_name = "FAMILY.KIND")]
        require_kind: Vec<String>,
        /// Require events from this canonical capture source, repeatable
        #[arg(long = "require-source", value_name = "SOURCE")]
        require_source: Vec<String>,
        /// Fail the check when an authored block cites changed evidence
        #[arg(long)]
        fail_on_stale_authored: bool,
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
    /// Acknowledge an authored block's changed evidence as reviewed
    Acknowledge {
        /// Document carrying the authored block
        document: PathBuf,
        /// Authored block name, e.g. `authored-1`
        #[arg(long, value_name = "NAME")]
        block: String,
        /// Event ids the acknowledgment covers
        #[arg(long = "events", required = true, num_args = 1.., value_name = "ID")]
        events: Vec<String>,
        /// Name of the acknowledging human
        #[arg(long, value_name = "NAME")]
        actor: String,
        /// Actor kind; only `human` may acknowledge authored evidence
        #[arg(long = "actor-type", default_value = "human", value_name = "TYPE")]
        actor_type: String,
    },
    /// Run every declared doctest contract and record the run as one fact
    Doctest {
        /// Research projection directory holding the document sources
        #[arg(long, default_value = "docs/research", value_name = "DIR")]
        out: PathBuf,
        /// Project namespace recorded in the event identity
        #[arg(long, default_value = "kaya", value_name = "NAME")]
        project: String,
        /// Wall-clock budget for the whole run, in seconds
        #[arg(long, default_value_t = 10, value_name = "SECS")]
        timeout: u64,
    },
    /// Record a human decision that a doctest's expected output changed
    SupersedeDoctest {
        /// Document carrying the doctest
        document: PathBuf,
        /// Doctest label, e.g. `doctest-1` or a declared label
        #[arg(long, value_name = "NAME")]
        label: String,
        /// Name of the superseding human
        #[arg(long, value_name = "NAME")]
        actor: String,
        /// Actor kind; only `human` may supersede expected output
        #[arg(long = "actor-type", default_value = "human", value_name = "TYPE")]
        actor_type: String,
    },
    /// Gate a change set: the records it impacts and their obligations
    Impact {
        /// Gate the staged change set against HEAD
        #[arg(long)]
        staged: bool,
        /// Gate the change set between this base rev and HEAD, e.g. `main` or `main..`
        #[arg(long, value_name = "REV")]
        range: Option<String>,
        /// Research projection directory holding the document sources
        #[arg(long, default_value = "docs/research", value_name = "DIR")]
        out: PathBuf,
        /// Wall-clock budget for one doctest run, in seconds
        #[arg(long, default_value_t = 10, value_name = "SECS")]
        timeout: u64,
    },
    /// Record a human disposition for one impacted record
    Disposition {
        /// Impacted record id, e.g. `docs/research/index.md#doctest-1`
        record: String,
        /// Disposition kind: `reviewed` or `waived`
        kind: String,
        /// Why the record is waived; required for `waived`
        #[arg(long, value_name = "TEXT")]
        rationale: Option<String>,
        /// Name of the deciding human
        #[arg(long, value_name = "NAME")]
        actor: String,
        /// Actor kind; only `human` may record dispositions
        #[arg(long = "actor-type", default_value = "human", value_name = "TYPE")]
        actor_type: String,
        /// Research projection directory holding the document sources
        #[arg(long, default_value = "docs/research", value_name = "DIR")]
        out: PathBuf,
    },
    /// Capture an authored observation as one model event
    Note {
        /// The observation text, stored after secret redaction
        text: String,
        #[command(flatten)]
        args: CaptureArgs,
    },
    /// Capture an authored question as one model event
    Question {
        /// The question text, stored after secret redaction
        text: String,
        #[command(flatten)]
        args: CaptureArgs,
    },
    /// Capture an authored decision as one model event
    Decision {
        /// The decision text, stored after secret redaction
        text: String,
        #[command(flatten)]
        args: CaptureArgs,
    },
    /// Capture an authored finding as one model event
    Finding {
        /// The finding text, stored after secret redaction
        text: String,
        #[command(flatten)]
        args: CaptureArgs,
    },
    /// Explain publication omissions and their unblock paths without writing
    Explain {
        /// Optional product, collection, or document kind name
        name: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum ImportSource {
    /// Import observed artifacts from the `.wai/` artifact tree
    Wai {
        /// Directory containing the `.wai/` tree
        #[arg(long, default_value = ".", value_name = "DIR")]
        root: PathBuf,
        /// Project namespace recorded in the event identity
        #[arg(long, default_value = "kaya", value_name = "NAME")]
        project: String,
    },
    /// Import observed commits from repository history
    Git {
        /// Repository root to read history from
        #[arg(long, default_value = ".", value_name = "DIR")]
        root: PathBuf,
        /// Project namespace recorded in the event identity
        #[arg(long, default_value = "kaya", value_name = "NAME")]
        project: String,
        /// Maximum number of recent commits to observe
        #[arg(long, default_value_t = 100, value_name = "N")]
        limit: usize,
        /// Skip commits whose subject starts with this prefix (repeatable)
        #[arg(long = "exclude-message-prefix", value_name = "PREFIX")]
        exclude_message_prefix: Vec<String>,
    },
    /// Import observed whisper entries from a turu bundle file
    Turu {
        /// Path to the turu bundle file
        #[arg(long, value_name = "PATH")]
        bundle: PathBuf,
        /// Project namespace recorded in the event identity
        #[arg(long, default_value = "kaya", value_name = "NAME")]
        project: String,
    },
    /// Import one completed test run from a libtest JSONL report
    Test {
        /// Path to the machine-readable run report
        #[arg(long, value_name = "PATH")]
        report: PathBuf,
        /// Project namespace recorded in the event identity
        #[arg(long, default_value = "kaya", value_name = "NAME")]
        project: String,
    },
    /// Import model events from a JSONL seed file
    Model {
        /// Path to the JSONL file of `model.*` records
        #[arg(long, value_name = "PATH")]
        file: PathBuf,
    },
}

fn main() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => match err.kind() {
            ClapErrorKind::DisplayHelp | ClapErrorKind::DisplayVersion => err.exit(),
            _ if invoked("check") => {
                return emit_code(
                    Envelope::error("error", "kaya.verification.usage", err.to_string()),
                    2,
                );
            }
            _ if invoked("acknowledge") => {
                return emit(Envelope::error(
                    "error",
                    "kaya.acknowledge.usage",
                    err.to_string(),
                ));
            }
            _ if invoked("impact") || invoked("disposition") => {
                return emit(Envelope::error(
                    "error",
                    "kaya.impact.usage",
                    err.to_string(),
                ));
            }
            _ => return emit(Envelope::error("error", "usage", err.to_string())),
        },
    };
    let outcome = match cli.command {
        Command::Init => commands::status::cmd_init(),
        Command::Status => commands::status::cmd_status(),
        Command::Explain { name } => explain::run(name.as_deref()),
        Command::Import { source } => commands::import::cmd_import(source),
        Command::Build {
            out,
            expect_digest,
            allow_uncommitted,
            instrument,
        } => commands::publication::cmd_build(
            &out,
            expect_digest.as_deref(),
            allow_uncommitted,
            instrument,
        ),
        Command::Ir {
            out,
            expect_digest,
            allow_uncommitted,
            walk,
        } => commands::publication::cmd_ir(&out, expect_digest.as_deref(), allow_uncommitted, walk),
        Command::Publish { out } => commands::publication::cmd_publish(&out),
        Command::Check {
            root,
            out,
            publish_out,
            expect_digest,
            index,
            require_kind,
            require_source,
            fail_on_stale_authored,
        } => {
            let options = verification::CheckOptions {
                index,
                require_kinds: require_kind,
                require_sources: require_source,
                fail_on_stale_authored,
            };
            return commands::publication::cmd_check(
                &root,
                &out,
                &publish_out,
                expect_digest.as_deref(),
                &options,
            );
        }
        Command::Promote {
            assertion_id,
            actor,
        } => commands::verification::cmd_promote(&assertion_id, &actor),
        Command::Challenge {
            conclusion_id,
            events,
            actor,
        } => commands::verification::cmd_challenge(&conclusion_id, &events, &actor),
        Command::Acknowledge {
            document,
            block,
            events,
            actor,
            actor_type,
        } => {
            commands::verification::cmd_acknowledge(&document, &block, &events, &actor, &actor_type)
        }
        Command::Impact {
            staged,
            range,
            out,
            timeout,
        } => {
            return match commands::impact::cmd_impact(staged, range.as_deref(), &out, timeout) {
                Ok((envelope, code)) => emit_code(envelope, code),
                Err((label, message)) => emit(Envelope::error("error", &label, message)),
            };
        }
        Command::Disposition {
            record,
            kind,
            rationale,
            actor,
            actor_type,
            out,
        } => commands::impact::cmd_disposition(
            &record,
            &kind,
            rationale.as_deref(),
            &actor,
            &actor_type,
            &out,
        ),
        Command::Doctest {
            out,
            project,
            timeout,
        } => commands::doctest::cmd_doctest(&out, &project, timeout),
        Command::SupersedeDoctest {
            document,
            label,
            actor,
            actor_type,
        } => commands::doctest::cmd_supersede_doctest(&document, &label, &actor, &actor_type),
        Command::Note { text, args } => {
            commands::capture::cmd_capture(capture::Kind::Note, &text, &args)
        }
        Command::Question { text, args } => {
            commands::capture::cmd_capture(capture::Kind::Question, &text, &args)
        }
        Command::Decision { text, args } => {
            commands::capture::cmd_capture(capture::Kind::Decision, &text, &args)
        }
        Command::Finding { text, args } => {
            commands::capture::cmd_capture(capture::Kind::Finding, &text, &args)
        }
    };
    emit(outcome.unwrap_or_else(|(label, message)| Envelope::error("error", &label, message)))
}

/// Whether the invocation names `name` as its subcommand.
///
/// A parse failure never yields the [`Command`] enum, so the raw argument is
/// the only way to keep a subcommand's usage failures in its own label
/// namespace.
fn invoked(name: &str) -> bool {
    std::env::args_os().nth(1).is_some_and(|arg| arg == name)
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

/// Print exactly one envelope and exit with the verdict it carries.
///
/// `check` is the one command whose success envelope can still mean drift, so
/// the exit code is decided by the caller rather than by `ok`: usage and I/O
/// failures print an error envelope and exit 2, an executed check exits with
/// the drift verdict.
pub(crate) fn emit_code(envelope: Envelope, code: u8) -> ExitCode {
    match envelope.print() {
        Ok(()) => ExitCode::from(code),
        Err(message) => {
            eprintln!("kaya: {message}");
            ExitCode::from(2)
        }
    }
}

pub(crate) fn failure(label: impl Into<String>, message: impl Into<String>) -> Failure {
    (label.into(), message.into())
}

/// Map a library ledger refusal onto the CLI failure classes.
///
/// A missing ledger is the one class the CLI renames: `kaya.ledger.missing`
/// becomes `missing-ledger`, matching `status`, and the message already
/// points at `kaya init`.
pub(crate) fn ledger_failure(err: LedgerError) -> Failure {
    let label = if err.label == "kaya.ledger.missing" {
        "missing-ledger".to_string()
    } else {
        err.label
    };
    (label, err.message)
}

pub(crate) fn adapter_failure(err: AdapterError) -> Failure {
    (err.label, err.message)
}

pub(crate) fn model_failure(err: ModelError) -> Failure {
    (err.label, err.message)
}

pub(crate) fn projection_failure(err: ProjectionError) -> Failure {
    (err.label, err.message)
}

fn promotion_failure(err: PromotionError) -> Failure {
    (err.label, err.message)
}

/// Open the existing ledger, refusing an absent one with `missing-ledger`.
pub(crate) fn open_ledger() -> Result<Ledger, Failure> {
    Ledger::open(LEDGER_PATH).map_err(ledger_failure)
}

/// A human actor named on the command line.
///
/// The CLI has no way to name a tool or agent actor: promotion and
/// acknowledgment are the human doors into the record.
pub(crate) fn human_actor(name: &str) -> Actor {
    Actor {
        r#type: ActorType::Human,
        name: name.to_owned(),
    }
}

/// Materialize the document IR from the research sources, using the same
/// projection plans build uses.
///
/// The gate reads the record build ships, never a parallel one, so the gate
/// and the compiled record cannot disagree about record ids or edges.
pub(crate) fn collect_document_ir(
    out: &Path,
    events: &[Event],
) -> Result<(ir::DocumentIr, IrCollector), Failure> {
    let graph = model::build(events).map_err(model_failure)?;
    let state = PromotionState::from_events(events).map_err(promotion_failure)?;

    let mut collector = IrCollector::default();

    let index_path = out.join("index.md");
    let existing = read_document(&index_path)?;
    let (document, summary) = index::plan(
        &graph,
        &index_path,
        existing.as_deref(),
        report::recorded_event_count(events),
    )
    .map_err(projection_failure)?;
    collector.collect(&index_path, &document, summary)?;

    let experiments: Vec<String> = graph
        .nodes()
        .filter(|node| node.entity == EntityType::Experiment)
        .map(|node| node.id.clone())
        .collect();
    let experiments_dir = out.join("experiments");
    for experiment_id in experiments {
        let report_path = experiments_dir.join(format!("{}.md", experiment_id.replace('/', "-")));
        let existing = read_document(&report_path)?;
        let planned = report::plan(&graph, &experiment_id, existing.as_deref(), &state)
            .map_err(projection_failure)?;
        collector.collect(&report_path, &planned.document, planned.summary)?;
    }

    let understanding_path = out.join("understanding.md");
    let existing = read_document(&understanding_path)?;
    let planned =
        report::plan_understanding(&graph, existing.as_deref()).map_err(projection_failure)?;
    collector.collect(&understanding_path, &planned.document, planned.summary)?;

    let overview_path = out.join("overview.md");
    let existing = read_document(&overview_path)?;
    let planned = report::plan_overview(&graph, existing.as_deref()).map_err(projection_failure)?;
    collector.collect(&overview_path, &planned.document, planned.summary)?;

    let brief_path = out.join("brief.md");
    let existing = read_document(&brief_path)?;
    let planned = report::plan_brief(&graph, existing.as_deref()).map_err(projection_failure)?;
    collector.collect(&brief_path, &planned.document, planned.summary)?;

    let reuse_inputs = reuse::inputs(&graph).map_err(projection_failure)?;
    let mut existing_map = BTreeMap::new();
    for (_, path) in reuse::documents(out) {
        let document = read_document(&path)?;
        existing_map.insert(path, document);
    }
    for planned in reuse::plan(&reuse_inputs, out, &existing_map).map_err(projection_failure)? {
        let reuse::PlannedDocument {
            path,
            document,
            summary,
            ..
        } = planned;
        if let Some(document) = document {
            collector.collect(&path, &document, summary)?;
        }
    }

    Ok((
        ir::materialize(&collector.sources, &collector.rendered, &BTreeMap::new()),
        collector,
    ))
}

/// Read a document source, treating absence as no document.
///
/// Mirrors the projection reader's semantics without crossing into the
/// library's crate-private surface.
fn read_document(path: &Path) -> Result<Option<String>, Failure> {
    match fs::read_to_string(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(failure(
            "ir-io",
            format!("failed to read {}: {error}", path.display()),
        )),
    }
}

/// Accumulates the document sources and rendered blocks the IR materializes
/// from.
#[derive(Default)]
pub(crate) struct IrCollector {
    pub(crate) sources: Vec<ir::SourceDocument>,
    pub(crate) rendered: Vec<ir::RenderedDocument>,
}

impl IrCollector {
    /// Record one planned document: its bytes as a source, its generated
    /// blocks as rendered records.
    ///
    /// A document that breaks the managed-block grammar is a labeled refusal:
    /// the IR never silently drops a document's records.
    fn collect(&mut self, path: &Path, document: &str, summary: Summary) -> Result<(), Failure> {
        let path_string = path.to_string_lossy().to_string();
        self.sources.push(ir::SourceDocument {
            path: path_string.clone(),
            bytes: document.to_string(),
        });
        let blocks: Vec<ir::RenderedBlock> = scan(document)
            .map_err(|error| {
                failure(
                    "ir-grammar",
                    format!(
                        "{} broke the managed-block grammar: {error}",
                        path.display()
                    ),
                )
            })?
            .into_iter()
            .filter_map(|block| match block.kind {
                BlockKind::Generated(name) => {
                    let provenance = ir::provenance_of(&summary, &name);
                    Some(ir::RenderedBlock {
                        name,
                        body: block.body,
                        provenance,
                    })
                }
                BlockKind::Authored => None,
            })
            .collect();
        self.rendered.push(ir::RenderedDocument {
            path: path_string,
            blocks,
        });
        Ok(())
    }
}

/// Read and verify the ledger for a build, without a second read.
///
/// The bytes are read exactly once. When git tracks the ledger, the committed
/// `HEAD` bytes must be a prefix and every added byte must complete a
/// newline-terminated line, so only ledger history git already recorded is
/// served. An untracked ledger has no such history: it is accepted only when
/// the caller pins its digest or explicitly allows uncommitted history.
pub(crate) fn read_build_ledger(
    expect_digest: Option<&str>,
    allow_uncommitted: bool,
) -> Result<(Vec<Event>, String), Failure> {
    let bytes = fs::read(LEDGER_PATH).map_err(|err| match err.kind() {
        ErrorKind::NotFound => failure(
            "missing-ledger",
            format!("missing ledger {LEDGER_PATH}: run `kaya init` to create it"),
        ),
        _ => failure("ledger-io", format!("failed to read {LEDGER_PATH}: {err}")),
    })?;

    match committed_ledger()? {
        Some(committed) => verify_committed_ledger(&committed, &bytes)?,
        None => {
            if expect_digest.is_none() && !allow_uncommitted {
                return Err(failure(
                    "kaya.ledger.uncommitted_history",
                    format!(
                        "ledger {LEDGER_PATH} is untracked, so git cannot verify its history; \
                         rerun with --expect-digest <hex> to pin its exact bytes or \
                         --allow-uncommitted to accept them as recorded"
                    ),
                ));
            }
        }
    }

    let events = parse_ledger_bytes(&bytes)?;
    let digest = sha256_hex(&bytes);
    if let Some(expected) = expect_digest
        && digest != expected
    {
        return Err(failure(
            "kaya.ledger.digest_mismatch",
            format!(
                "ledger {LEDGER_PATH} does not match the expected digest {expected}: the verified bytes digest {digest}"
            ),
        ));
    }
    Ok((events, digest))
}

/// The committed `HEAD` bytes of the ledger, when git tracks it.
///
/// `None` means the file is untracked or the working tree is not a git
/// repository; a tracked file absent from `HEAD` is a refusal, because there
/// is no committed history to verify against.
fn committed_ledger() -> Result<Option<Vec<u8>>, Failure> {
    let tracked = ProcessCommand::new("git")
        .args(["ls-files", "--error-unmatch", LEDGER_PATH])
        .output();
    match tracked {
        Ok(output) if output.status.success() => {}
        Ok(_) | Err(_) => return Ok(None),
    }

    let spec = format!("HEAD:{LEDGER_PATH}");
    let committed = ProcessCommand::new("git")
        .args(["show", &spec])
        .output()
        .map_err(|err| {
            failure(
                "kaya.ledger.uncommitted_history",
                format!(
                    "ledger {LEDGER_PATH} is tracked but `git show {spec}` could not run: {err}"
                ),
            )
        })?;
    if !committed.status.success() {
        return Err(failure(
            "kaya.ledger.uncommitted_history",
            format!(
                "ledger {LEDGER_PATH} is tracked but HEAD carries no committed copy; \
                 commit the ledger before building"
            ),
        ));
    }
    Ok(Some(committed.stdout))
}

/// Require committed bytes to prefix the current ledger and complete its lines.
fn verify_committed_ledger(committed: &[u8], current: &[u8]) -> Result<(), Failure> {
    let prefix_ok =
        current.starts_with(committed) && (committed.is_empty() || committed.ends_with(b"\n"));
    let growth_ok = current.len() == committed.len()
        || current[committed.len().min(current.len())..].ends_with(b"\n");
    if prefix_ok && growth_ok {
        return Ok(());
    }
    let line = first_unverified_line(committed, current);
    Err(failure(
        "kaya.ledger.uncommitted_history",
        format!(
            "ledger {LEDGER_PATH} does not extend its committed HEAD history: line {line} is unverified; \
             commit the ledger, pin the exact bytes with --expect-digest <hex>, or accept them with --allow-uncommitted"
        ),
    ))
}

/// 1-based line number of the first byte git did not commit.
fn first_unverified_line(committed: &[u8], current: &[u8]) -> usize {
    let shared = committed.len().min(current.len());
    let offset = committed[..shared]
        .iter()
        .zip(&current[..shared])
        .position(|(left, right)| left != right)
        .unwrap_or(shared);
    current[..offset]
        .iter()
        .filter(|byte| **byte == b'\n')
        .count()
        + 1
}

/// Parse the exact verified buffer, one event per newline-terminated line.
fn parse_ledger_bytes(bytes: &[u8]) -> Result<Vec<Event>, Failure> {
    let mut events = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for (index, raw) in bytes.split_inclusive(|byte| *byte == b'\n').enumerate() {
        let line = index + 1;
        let content = raw.strip_suffix(b"\n").unwrap_or(raw);
        let text = std::str::from_utf8(content).map_err(|err| {
            failure(
                "kaya.ledger.record_invalid",
                format!("{LEDGER_PATH} line {line} is not UTF-8: {err}"),
            )
        })?;
        let event = parse_line(text).map_err(|err| {
            failure(
                err.label,
                format!("{LEDGER_PATH} line {line}: {}", err.message),
            )
        })?;
        if !seen.insert(event.id.clone()) {
            return Err(failure(
                "kaya.ledger.id_reuse",
                format!(
                    "{LEDGER_PATH} line {line} reuses event id \"{}\"; ids are never reused — record a correction naming the id in data.supersedes",
                    event.id
                ),
            ));
        }
        events.push(event);
    }
    Ok(events)
}
