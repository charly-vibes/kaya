//! Publication commands: compile, inspect, publish, and verify the record.
//!
//! Responsibilities: hold `cmd_build` (incremental projection compilation
//! with skip/restore and staleness flagging), `cmd_ir` (read-only document
//! IR), `cmd_publish` (the client-facing tree), and `cmd_check` (the drift
//! verdict), together with their private single-document, summary-merge,
//! staleness, and failure-mapping helpers.
//!
//! Rationale: the four commands share one gate over the verified ledger and
//! the same projection plans, so their helpers stay together while the
//! verified-ledger and IR collectors remain shared in `main.rs` with the
//! impact family.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use kaya::envelope::Envelope;
use kaya::event::Event;
use kaya::ledger::changed_event_ids;
use kaya::model::{self, EntityType};
use kaya::projection::{
    ProjectionError, StaleBlock, Summary, index, render_invocations, report, reuse, state,
};
use kaya::promotion::PromotionState;
use kaya::publication;
use kaya::verification;
use serde_json::json;

use crate::{
    Failure, LEDGER_PATH, collect_document_ir, emit_code, failure, model_failure,
    projection_failure, promotion_failure, read_build_ledger,
};

pub(crate) fn cmd_build(
    out: &Path,
    expect_digest: Option<&str>,
    allow_uncommitted: bool,
    instrument: bool,
) -> Result<Envelope, Failure> {
    let (events, ledger_digest) = read_build_ledger(expect_digest, allow_uncommitted)?;
    let changed_ids = changed_event_ids(&events);
    let known_ids: BTreeSet<String> = events.iter().map(|event| event.id.clone()).collect();

    let graph = model::build(&events).map_err(model_failure)?;
    let state = PromotionState::from_events(&events).map_err(promotion_failure)?;

    fs::create_dir_all(out).map_err(|err| {
        failure(
            "build-io",
            format!("failed to create {}: {err}", out.display()),
        )
    })?;

    // Incremental regeneration: a document whose inputs — ledger digest,
    // generator identity, and on-disk bytes — all match the last successful
    // build is not rebuilt; its recorded contribution is restored into the
    // summary instead, so the envelope stays complete and truthful. Citation
    // validation and staleness flagging below always run, even when skipped.
    let state_path = Path::new(LEDGER_PATH).with_file_name("build-state.json");
    let saved = state::BuildState::current(state::BuildState::load(&state_path), &ledger_digest);
    // Skipped documents keep their recorded entries: a run that skips every
    // document must persist the same gate it consumed, or the next build
    // would alternate back to a full render. The digest gate above guarantees
    // carried entries still belong to this build's document set.
    let mut documents = saved
        .as_ref()
        .map(|saved| saved.documents.clone())
        .unwrap_or_default();
    let mut omitted: Vec<state::OmittedEntry> = Vec::new();
    let mut summary = Summary::default();

    let index_path = out.join("index.md");
    let index_paths = vec![("index.md".to_owned(), index_path.clone())];
    match saved
        .as_ref()
        .and_then(|s| s.skippable("index.md", &index_paths))
    {
        Some(entry) => restore_skipped(&mut summary, entry),
        None => {
            let built = index::build(&graph, &index_path, report::recorded_event_count(&events))
                .map_err(projection_failure)?;
            state::record(
                &mut documents,
                "index.md",
                &index_paths,
                &built.summary,
                &[],
            );
            merge_summary(&mut summary, built.summary);
        }
    }
    flag_stale_authored(&mut summary, &index_path, &changed_ids, &known_ids, &events)?;

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
            let file_name = format!("{}.md", experiment_id.replace('/', "-"));
            let report_path = experiments_dir.join(&file_name);
            let key = format!("experiments/{file_name}");
            let paths = vec![(key.clone(), report_path.clone())];
            match saved.as_ref().and_then(|s| s.skippable(&key, &paths)) {
                Some(entry) => {
                    restore_skipped(&mut summary, entry);
                    omitted.extend(entry.omitted.iter().cloned());
                }
                None => {
                    let built = report::build(&graph, &experiment_id, &report_path, &state)
                        .map_err(projection_failure)?;
                    let entry_omitted: Vec<state::OmittedEntry> = built
                        .omitted
                        .iter()
                        .map(state::OmittedEntry::from)
                        .collect();
                    omitted.extend(entry_omitted.iter().cloned());
                    state::record(&mut documents, &key, &paths, &built.summary, &entry_omitted);
                    merge_summary(&mut summary, built.summary);
                }
            }
            flag_stale_authored(
                &mut summary,
                &report_path,
                &changed_ids,
                &known_ids,
                &events,
            )?;
        }
    }

    // The understanding projection is project-level: one document, both
    // kinds, whether or not the ledger declares an experiment. An empty
    // ledger leaves the document absent rather than writing a blank file.
    build_single(
        &mut summary,
        &mut omitted,
        &mut documents,
        saved.as_ref(),
        "understanding.md",
        &out.join("understanding.md"),
        |path| report::build_understanding(&graph, path),
    )?;
    flag_stale_authored(
        &mut summary,
        &out.join("understanding.md"),
        &changed_ids,
        &known_ids,
        &events,
    )?;

    // The overview is the orientation slice of the same graph: Project,
    // Question, and current Findings, or no document at all.
    build_single(
        &mut summary,
        &mut omitted,
        &mut documents,
        saved.as_ref(),
        "overview.md",
        &out.join("overview.md"),
        |path| report::build_overview(&graph, path),
    )?;
    flag_stale_authored(
        &mut summary,
        &out.join("overview.md"),
        &changed_ids,
        &known_ids,
        &events,
    )?;

    build_single(
        &mut summary,
        &mut omitted,
        &mut documents,
        saved.as_ref(),
        "brief.md",
        &out.join("brief.md"),
        |path| report::build_brief(&graph, path),
    )?;
    flag_stale_authored(
        &mut summary,
        &out.join("brief.md"),
        &changed_ids,
        &known_ids,
        &events,
    )?;

    // The reuse projection always runs, even when the ledger holds no reuse
    // references: an existing document's stale blocks and legacy names are
    // only visible by validating the documents, and an empty kind is reported
    // once so callers see the whole closed family.
    let reuse_paths: Vec<(String, PathBuf)> = reuse::documents(out)
        .into_iter()
        .map(|(block, path)| (block.to_owned(), path))
        .collect();
    match saved
        .as_ref()
        .and_then(|s| s.skippable("reuse", &reuse_paths))
    {
        Some(entry) => {
            restore_skipped(&mut summary, entry);
            omitted.extend(entry.omitted.iter().cloned());
        }
        None => {
            let reuse_inputs = reuse::inputs(&graph).map_err(projection_failure)?;
            let built = reuse::build(&reuse_inputs, out).map_err(projection_failure)?;
            let entry_omitted: Vec<state::OmittedEntry> = built
                .omitted
                .iter()
                .map(state::OmittedEntry::from)
                .collect();
            omitted.extend(entry_omitted.iter().cloned());
            state::record(
                &mut documents,
                "reuse",
                &reuse_paths,
                &built.summary,
                &entry_omitted,
            );
            merge_summary(&mut summary, built.summary);
        }
    }
    for (_, path) in reuse::documents(out) {
        flag_stale_authored(&mut summary, &path, &changed_ids, &known_ids, &events)?;
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
    omitted.sort();
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

    // Persist the state only after a successful build, so a failed build
    // leaves the previous state authoritative for the next run.
    let final_state = state::BuildState::empty(&ledger_digest).with_documents(documents);
    final_state.store(&state_path).map_err(|error| {
        failure(
            "build-io",
            format!("failed to write {}: {error}", state_path.display()),
        )
    })?;

    let mut data = json!({
        "out": out.display().to_string(),
        "recomputed": recomputed,
        "unchanged": unchanged,
        "omitted": omitted,
        "stale": stale,
        "sources": sources,
        "events": events.len(),
        "pending_proposals": state.pending_count(),
        "ledger_digest": ledger_digest,
    });
    if instrument {
        data["render_invocations"] = json!(render_invocations());
    }
    Ok(Envelope::ok("build", data))
}

/// Build one single-document projection with the skip predicate, or record
/// and merge its contribution after a real build.
fn build_single<B>(
    summary: &mut Summary,
    omitted: &mut Vec<state::OmittedEntry>,
    documents: &mut BTreeMap<String, state::DocumentState>,
    saved: Option<&state::BuildState>,
    key: &str,
    path: &Path,
    build: B,
) -> Result<(), Failure>
where
    B: FnOnce(&Path) -> Result<report::Built, ProjectionError>,
{
    let paths = vec![(key.to_owned(), path.to_path_buf())];
    match saved.and_then(|s| s.skippable(key, &paths)) {
        Some(entry) => {
            restore_skipped(summary, entry);
            omitted.extend(entry.omitted.iter().cloned());
        }
        None => {
            let built = build(path).map_err(projection_failure)?;
            let entry_omitted: Vec<state::OmittedEntry> = built
                .omitted
                .iter()
                .map(state::OmittedEntry::from)
                .collect();
            omitted.extend(entry_omitted.iter().cloned());
            state::record(documents, key, &paths, &built.summary, &entry_omitted);
            merge_summary(summary, built.summary);
        }
    }
    Ok(())
}

/// Restore a skipped document's recorded summary contribution: its generated
/// blocks report `unchanged` exactly as a real unchanged build would, and its
/// source map is restored verbatim.
fn restore_skipped(summary: &mut Summary, entry: &state::DocumentState) {
    summary.unchanged.extend(entry.blocks.iter().cloned());
    for (block, ids) in &entry.sources {
        summary
            .sources
            .entry(block.clone())
            .or_default()
            .extend(ids.iter().cloned());
    }
}

/// Print the document IR of the compiled record without writing anything.
///
/// The plans run read-only against the existing document sources and the
/// ledger: nothing outside the document sources and the ledger feeds the IR,
/// so build caches and stale sidecars cannot change it. The IR covers the
/// research index, every experiment report, the understanding and overview
/// documents, and the reuse family.
pub(crate) fn cmd_ir(
    out: &Path,
    expect_digest: Option<&str>,
    allow_uncommitted: bool,
) -> Result<Envelope, Failure> {
    let (events, _) = read_build_ledger(expect_digest, allow_uncommitted)?;
    let (document_ir, _) = collect_document_ir(out, &events)?;
    let payload = serde_json::to_value(&document_ir).map_err(|error| {
        failure(
            "ir-serialize",
            format!("failed to serialize the document IR: {error}"),
        )
    })?;
    Ok(Envelope::ok("ir", payload))
}

/// Compose the twelve declared products and both declared packages over the
/// compiled research documents.
///
/// The declarations are [`publication::default_declarations`], so `kaya
/// publish` and `kaya check` compile exactly the same tree. The document
/// source is read once from `docs/research`; the publication layer refuses
/// invalid declarations before writing, omits products without a backing
/// document, and exports the same attribution it renders. It also owns the
/// write: each page is a `publication.<tier>.<name>` managed block inside its
/// file, so published bytes stay compiler-owned without a second pass here.
pub(crate) fn cmd_publish(out: &Path) -> Result<Envelope, Failure> {
    let documents =
        publication::read_documents(Path::new("docs/research")).map_err(publication_failure)?;

    let declarations = publication::default_declarations();

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

/// Verify the record without writing: the expected bytes come from the same
/// planners `build` and `publish` use, compared against the worktree or, with
/// `options.index`, against the staged blobs a commit would record.
///
/// `check` reports drift, unmet requirements, and staleness in its own success
/// envelope and lets the exit code carry the verdict: 0 clean, 1 drift or an
/// enabled violation, 2 usage or I/O failure. Only the flags decide which
/// findings are violations, so the same envelope proves both a plain advisory
/// run and an enforced gate. It creates no directory, appends no event, and
/// evaluates no threshold.
pub(crate) fn cmd_check(
    root: &Path,
    out: &Path,
    publish_out: &Path,
    expect_digest: Option<&str>,
    options: &verification::CheckOptions,
) -> ExitCode {
    match verification::check(root, out, publish_out, expect_digest, options) {
        Ok(outcome) => {
            let code = outcome.exit_code();
            let facts = outcome.facts;
            let envelope = Envelope::ok(
                "check",
                json!({
                    "drift": outcome.drift,
                    "required": outcome.required,
                    "stale": outcome.stale,
                    "events": facts.events,
                    "ledger_digest": facts.ledger_digest,
                    "last_capture_commit": facts.last_capture_commit,
                    "newest_event_at": facts.newest_event_at,
                    "events_by_source": facts.events_by_source,
                    "emitted_kinds": facts.emitted_kinds,
                    "omitted_kinds": facts.omitted_kinds,
                }),
            );
            emit_code(envelope, code)
        }
        Err(error) => emit_code(Envelope::error("error", &error.label, error.message), 2),
    }
}

/// Validate and report the authored blocks of one compiled document.
///
/// The document is read back after compilation, never rewritten, so authored
/// bytes stay exactly as the human left them. Its citation markers are checked
/// first — always, even when nothing changed — because staleness over a
/// citation of an id the ledger does not record would report on evidence that
/// does not exist. Only then are changed citations reported, minus the ids a
/// human acknowledged for this document and block.
fn flag_stale_authored(
    summary: &mut Summary,
    path: &Path,
    changed_ids: &[String],
    known_ids: &BTreeSet<String>,
    events: &[Event],
) -> Result<(), Failure> {
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
    let label = report::document_label_for(path);
    report::validate_citations(&document, &label, known_ids).map_err(projection_failure)?;
    if changed_ids.is_empty() {
        return Ok(());
    }
    let acknowledged = report::acknowledged(events, &label);
    let stale = report::stale_authored(&document, changed_ids, &acknowledged);
    summary
        .stale
        .extend(report::unacknowledged(stale, &acknowledged));
    Ok(())
}

fn merge_summary(into: &mut Summary, from: Summary) {
    into.recomputed.extend(from.recomputed);
    into.unchanged.extend(from.unchanged);
    into.stale.extend(from.stale);
    for (block, ids) in from.sources {
        into.sources.entry(block).or_default().extend(ids);
    }
}

fn publication_failure(err: publication::PublicationError) -> Failure {
    (err.label, err.message)
}
