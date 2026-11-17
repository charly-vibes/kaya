//! kaya — a living record of exploration.
//!
//! The MVP-1 CLI: `init` bootstraps the ledger, `status` reports its shape and
//! pending proposals, `import` pulls observations from a source adapter or
//! model events from a seed file, `build` compiles the docs projections, and
//! `promote`/`challenge` append human promotion events. Every invocation prints
//! exactly one genesis-style JSON envelope to stdout; failures print an error
//! envelope labeled with the failure class — a CLI class (`usage`,
//! `missing-ledger`, `ledger-io`, `build-io`, `not-a-file`) or the label raised
//! by the library that refused the run.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Read};
use std::path::{Path, PathBuf};
use std::process::{Command as ProcessCommand, ExitCode};

use clap::error::ErrorKind as ClapErrorKind;
use clap::{Parser, Subcommand};
use serde_json::json;
use sha2::{Digest, Sha256};

use kaya::adapters::{AdapterError, git, testrunner, wai};
use kaya::envelope::Envelope;
use kaya::event::{Actor, ActorType, Event, parse_line};
use kaya::ledger::{AppendOutcome, Ledger, LedgerError, changed_event_ids, sha256_hex};
use kaya::model::{self, EntityType, ModelError};
use kaya::projection::{ProjectionError, StaleBlock, Summary, index, report, reuse};
use kaya::promotion::{self, PromotionError, PromotionState};
use kaya::publication::{self, Decl, Tier};

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
        /// Accept an untracked ledger whose history git cannot verify
        #[arg(long)]
        allow_uncommitted: bool,
    },
    /// Compose the client-facing publication tree from the compiled research
    Publish {
        /// Directory receiving the published pages and export
        #[arg(long, default_value = "docs/public", value_name = "DIR")]
        out: PathBuf,
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
            _ => return emit(Envelope::error("error", "usage", err.to_string())),
        },
    };
    let outcome = match cli.command {
        Command::Init => cmd_init(),
        Command::Status => cmd_status(),
        Command::Import { source } => cmd_import(source),
        Command::Build {
            out,
            expect_digest,
            allow_uncommitted,
        } => cmd_build(&out, expect_digest.as_deref(), allow_uncommitted),
        Command::Publish { out } => cmd_publish(&out),
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

fn publication_failure(err: publication::PublicationError) -> Failure {
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
        ImportSource::Model { file } => ("model", read_model_events(&file)?),
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

/// Read every `model.*` record from a JSONL seed file.
///
/// The whole file is validated before any event is returned, so a refusal
/// appends nothing. Each line must parse as one `kaya.event/v0` object and
/// name a `model.` type; every refusal names the 1-based line.
fn read_model_events(path: &Path) -> Result<Vec<Event>, Failure> {
    let bytes = fs::read(path).map_err(|err| {
        failure(
            "kaya.adapters.failure_atomic",
            format!("model: record {}: read: {err}", path.display()),
        )
    })?;

    let mut events = Vec::new();
    for (index, raw) in bytes.split_inclusive(|byte| *byte == b'\n').enumerate() {
        let line = index + 1;
        let content = raw.strip_suffix(b"\n").unwrap_or(raw);
        let text = std::str::from_utf8(content).map_err(|err| {
            failure(
                "kaya.adapters.failure_atomic",
                format!(
                    "model: record {line} in {}: line is not UTF-8: {err}",
                    path.display()
                ),
            )
        })?;
        let event = parse_line(text).map_err(|err| {
            failure(
                "kaya.adapters.failure_atomic",
                format!(
                    "model: record {line} in {}: {}",
                    path.display(),
                    err.message
                ),
            )
        })?;
        if !event.r#type.starts_with("model.") {
            return Err(failure(
                "kaya.adapters.failure_atomic",
                format!(
                    "model: record {line} in {}: type \"{}\" is not a model event; only model.* records are importable",
                    path.display(),
                    event.r#type
                ),
            ));
        }
        events.push(event);
    }
    Ok(events)
}

fn cmd_build(
    out: &Path,
    expect_digest: Option<&str>,
    allow_uncommitted: bool,
) -> Result<Envelope, Failure> {
    let (events, ledger_digest) = read_build_ledger(expect_digest, allow_uncommitted)?;
    let changed_ids = changed_event_ids(&events);

    let graph = model::build(&events).map_err(model_failure)?;
    let state = PromotionState::from_events(&events).map_err(promotion_failure)?;

    fs::create_dir_all(out).map_err(|err| {
        failure(
            "build-io",
            format!("failed to create {}: {err}", out.display()),
        )
    })?;
    let index_path = out.join("index.md");
    let mut summary =
        index::build(&graph, &index_path, events.len()).map_err(projection_failure)?;
    flag_stale_authored(&mut summary, &index_path, &changed_ids)?;

    let experiments: Vec<String> = graph
        .nodes()
        .filter(|node| node.entity == EntityType::Experiment)
        .map(|node| node.id.clone())
        .collect();
    let mut omitted: Vec<report::Omitted> = Vec::new();
    if !experiments.is_empty() {
        let experiments_dir = out.join("experiments");
        fs::create_dir_all(&experiments_dir).map_err(|err| {
            failure(
                "build-io",
                format!("failed to create {}: {err}", experiments_dir.display()),
            )
        })?;
        for experiment_id in experiments {
            let file_name = format!("{}.md", experiment_id.replace('/', "-"));
            let report_path = experiments_dir.join(file_name);
            let built = report::build(&graph, &experiment_id, &report_path, &state)
                .map_err(projection_failure)?;
            omitted.extend(built.omitted);
            merge_summary(&mut summary, built.summary);
            flag_stale_authored(&mut summary, &report_path, &changed_ids)?;
        }
    }

    // The understanding projection is project-level: one document, both
    // kinds, whether or not the ledger declares an experiment. An empty
    // ledger leaves the document absent rather than writing a blank file.
    let understanding_path = out.join("understanding.md");
    let built =
        report::build_understanding(&graph, &understanding_path).map_err(projection_failure)?;
    omitted.extend(built.omitted);
    merge_summary(&mut summary, built.summary);
    flag_stale_authored(&mut summary, &understanding_path, &changed_ids)?;

    // The overview is the orientation slice of the same graph: Project,
    // Question, and current Findings, or no document at all.
    let overview_path = out.join("overview.md");
    let built = report::build_overview(&graph, &overview_path).map_err(projection_failure)?;
    omitted.extend(built.omitted);
    merge_summary(&mut summary, built.summary);
    flag_stale_authored(&mut summary, &overview_path, &changed_ids)?;

    // The reuse projection always runs, even when the ledger holds no reuse
    // references: an existing document's stale blocks and legacy names are
    // only visible by validating the documents, and an empty kind is reported
    // once so callers see the whole closed family.
    let reuse_inputs = reuse::inputs(&graph).map_err(projection_failure)?;
    let built = reuse::build(&reuse_inputs, out).map_err(projection_failure)?;
    omitted.extend(built.omitted);
    merge_summary(&mut summary, built.summary);
    for (_, path) in reuse::documents(out) {
        flag_stale_authored(&mut summary, &path, &changed_ids)?;
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
    let mut sources = summary.sources;
    for ids in sources.values_mut() {
        ids.sort();
        ids.dedup();
    }
    omitted.sort_by(|left, right| (left.family, left.kind).cmp(&(right.family, right.kind)));
    omitted.dedup();
    let omitted: Vec<serde_json::Value> = omitted
        .into_iter()
        .map(|item| {
            json!({
                "family": item.family,
                "kind": item.kind,
                "reason": item.reason,
            })
        })
        .collect();

    Ok(Envelope::ok(
        "build",
        json!({
            "out": out.display().to_string(),
            "recomputed": recomputed,
            "unchanged": unchanged,
            "omitted": omitted,
            "stale": stale,
            "sources": sources,
            "pending_proposals": state.pending_count(),
            "ledger_digest": ledger_digest,
        }),
    ))
}

/// Products `kaya publish` declares: the project-site pair, the six
/// developer-guide products, and the status, release, and architecture
/// products. Every include is the product's admission row, so a declaration can
/// never widen a product beyond its contract. `tutorial` stays undeclared until
/// its backing exploration documents have producers.
const PUBLISHED_PRODUCTS: [&str; 11] = [
    "overview",
    "reference",
    "getting-started",
    "how-to",
    "concept",
    "example",
    "troubleshooting",
    "faq",
    "status-compatibility",
    "release-migration",
    "architecture-design",
];

/// Packages `kaya publish` declares, with their members in declaration order.
const PUBLISHED_PACKAGES: [(&str, &[&str]); 2] = [
    ("project-site", &["overview", "reference"]),
    (
        "developer-guide",
        &[
            "getting-started",
            "how-to",
            "concept",
            "example",
            "troubleshooting",
            "faq",
        ],
    ),
];

/// Compose the eleven declared products and both declared packages over the
/// compiled research documents.
///
/// The products are `overview`, `reference`, `getting-started`, `how-to`,
/// `concept`, `example`, `troubleshooting`, `faq`, `status-compatibility`,
/// `release-migration`, and `architecture-design`. The packages are
/// `project-site` (over `overview` and `reference`) and `developer-guide`
/// (over the six `getting-started` through `faq` products).
///
/// The document source is read once from `docs/research`; the publication
/// layer refuses invalid declarations before writing, omits products without
/// a backing document, and exports the same attribution it renders. It also
/// owns the write: each page is a `publication.<tier>.<name>` managed block
/// inside its file, so published bytes stay compiler-owned without a second
/// pass here.
fn cmd_publish(out: &Path) -> Result<Envelope, Failure> {
    let documents =
        publication::read_documents(Path::new("docs/research")).map_err(publication_failure)?;

    let declarations: Vec<Decl> = PUBLISHED_PRODUCTS
        .iter()
        .map(|name| Decl {
            tier: Tier::Product,
            name: (*name).to_owned(),
            includes: publication::admission(name)
                .iter()
                .map(|kind| (*kind).to_owned())
                .collect(),
            members: Vec::new(),
        })
        .chain(PUBLISHED_PACKAGES.iter().map(|(name, members)| Decl {
            tier: Tier::Package,
            name: (*name).to_owned(),
            includes: Vec::new(),
            members: members.iter().map(|member| (*member).to_owned()).collect(),
        }))
        .collect();

    let built = publication::build(&declarations, &documents, out).map_err(publication_failure)?;

    let pages: Vec<serde_json::Value> = built
        .pages
        .iter()
        .map(|page| {
            json!({
                "name": page.name,
                "path": page.path.display().to_string(),
                "question": page.question,
                "documents": page.documents,
            })
        })
        .collect();
    let omitted: Vec<serde_json::Value> = built
        .omitted
        .iter()
        .map(|entry| {
            json!({
                "tier": entry.tier,
                "name": entry.name,
                "reason": entry.reason,
            })
        })
        .collect();
    // The envelope reports the documents the tree actually attributes, in kind
    // order: one entry per kind the export carries, never a compiled document
    // no emitted product composes.
    let mut attributed: BTreeSet<&str> = BTreeSet::new();
    for page in &built.pages {
        for kind in &page.documents {
            attributed.insert(kind.as_str());
        }
    }
    let documents: Vec<serde_json::Value> = attributed
        .into_iter()
        .filter_map(|kind| documents.0.get(kind))
        .map(|document| {
            json!({
                "kind": document.kind,
                "event_ids": document.event_ids,
            })
        })
        .collect();
    // The build's own summary is the publish report: it names every page block
    // and the export exactly as the writer saw them.
    let mut recomputed = built.summary.recomputed.clone();
    recomputed.sort();
    recomputed.dedup();
    let mut unchanged = built.summary.unchanged.clone();
    unchanged.sort();
    unchanged.dedup();

    Ok(Envelope::ok(
        "publish",
        json!({
            "out": out.display().to_string(),
            "recomputed": recomputed,
            "unchanged": unchanged,
            "pages": pages,
            "omitted": omitted,
            "documents": documents,
        }),
    ))
}

/// Flag authored blocks in one compiled document whose cited evidence moved.
///
/// The document is read back after compilation, never rewritten, so authored
/// bytes stay exactly as the human left them.
fn flag_stale_authored(
    summary: &mut Summary,
    path: &Path,
    changed_ids: &[String],
) -> Result<(), Failure> {
    if changed_ids.is_empty() {
        return Ok(());
    }
    let document = match fs::read_to_string(path) {
        Ok(document) => document,
        // A report can legitimately be absent: when both kinds have no
        // source records, the build writes no file. Absence means there is
        // no authored prose to check, not an I/O failure.
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(failure(
                "build-io",
                format!("failed to read {}: {error}", path.display()),
            ));
        }
    };
    summary.stale.extend(report::stale_authored(
        &document,
        changed_ids,
        &BTreeMap::new(),
    ));
    Ok(())
}

/// Read and verify the ledger for a build, without a second read.
///
/// The bytes are read exactly once. When git tracks the ledger, the committed
/// `HEAD` bytes must be a prefix and every added byte must complete a
/// newline-terminated line, so only ledger history git already recorded is
/// served. An untracked ledger has no such history: it is accepted only when
/// the caller pins its digest or explicitly allows uncommitted history.
fn read_build_ledger(
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

fn merge_summary(into: &mut Summary, from: Summary) {
    into.recomputed.extend(from.recomputed);
    into.unchanged.extend(from.unchanged);
    into.stale.extend(from.stale);
    for (block, ids) in from.sources {
        into.sources.entry(block).or_default().extend(ids);
    }
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
