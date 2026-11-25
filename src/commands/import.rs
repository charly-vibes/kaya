//! Import commands: pull observations and model events into the ledger.
//!
//! Responsibilities: hold `cmd_import`, the CLI entry for every `import`
//! source subcommand, including the re-observation correction replay that
//! re-derives corrections for previously captured lines, and
//! `read_model_events`, the JSONL seed-file reader.
//!
//! Rationale: the clap `ImportSource` enum stays in `main.rs`; this module
//! carries the command body and its import-only file reader so append
//! semantics live beside the adapters they fan into.

use std::fs;
use std::path::Path;

use kaya::envelope::Envelope;
use kaya::event::{Event, parse_line};
use kaya::ledger::{AppendOutcome, LABEL_ID_REUSE};
use serde_json::json;

use kaya::adapters::{
    correction_generation, git, reobservation_correction, stored_correction_for, testrunner, turu,
    wai,
};

use crate::{
    Failure, ImportSource, LEDGER_PATH, adapter_failure, failure, ledger_failure, open_ledger,
};

pub(crate) fn cmd_import(source: ImportSource) -> Result<Envelope, Failure> {
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
            exclude_message_prefix,
        } => (
            "git",
            git::capture_excluding(&root, &project, limit, &exclude_message_prefix)
                .map_err(adapter_failure)?,
        ),
        ImportSource::Test { report, project } => (
            "test-runner",
            vec![testrunner::capture(&report, &project).map_err(adapter_failure)?],
        ),
        ImportSource::Turu { bundle, project } => (
            "turu",
            turu::capture(&bundle, &project).map_err(adapter_failure)?,
        ),
        ImportSource::Model { file } => ("model", read_model_events(&file)?),
    };

    let mut appended = 0usize;
    let mut duplicates = 0usize;
    let mut stored = ledger.read_recovered().map_err(ledger_failure)?.0;
    for event in &events {
        let outcome = match ledger.append(event) {
            Ok(outcome @ AppendOutcome::Appended) => outcome,
            Ok(_outcome @ AppendOutcome::Duplicate)
                if event.r#type.starts_with("capture.")
                    && stored_correction_for(&stored, event) =>
            {
                // A replay of a previously captured line whose effective
                // view a later correction overrides (e.g. a revert to an
                // earlier heading): re-correct so the newest state wins.
                let generation = correction_generation(&stored, event);
                let correction = reobservation_correction(event, generation);
                let outcome = ledger.append(&correction).map_err(ledger_failure)?;
                stored.push(correction);
                outcome
            }
            Ok(outcome) => outcome,
            Err(err) if err.label == LABEL_ID_REUSE && event.r#type.starts_with("capture.") => {
                // A corrected re-observation: re-derive the correction event
                // (deterministic per artifact identity, revisit generation,
                // and data state) and append that instead. Any refusal from
                // the correction itself still propagates.
                let generation = correction_generation(&stored, event);
                let correction = reobservation_correction(event, generation);
                let outcome = ledger.append(&correction).map_err(ledger_failure)?;
                stored.push(correction);
                outcome
            }
            Err(err) => return Err(ledger_failure(err)),
        };
        match outcome {
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
