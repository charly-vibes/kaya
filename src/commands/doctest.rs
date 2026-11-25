//! Doctest commands: run declared contracts and supersede their outputs.
//!
//! Responsibilities: hold `cmd_doctest` (run every declared contract through
//! the controlled runner and record the run as one fact) and
//! `cmd_supersede_doctest` (record a human decision that a contract's
//! expected output changed), together with their private phrasing and
//! latest-supersede helpers.
//!
//! Rationale: the clap argument tree stays in `main.rs`; this module carries
//! only the command bodies and their family-private helpers, sharing the
//! ledger-opening and failure helpers kept in `main.rs`.

use std::fs;
use std::path::Path;

use kaya::doctest;
use kaya::envelope::Envelope;
use kaya::event::parse_line;
use kaya::ledger::AppendOutcome;
use serde_json::json;

use crate::{
    Failure, KAYA_DIR, LEDGER_PATH, adapter_failure, failure, human_actor, ledger_failure,
    open_ledger,
};

/// Run every declared doctest contract and record the run as one fact.
///
/// Documents are read in sorted order, every declared contract runs through
/// the controlled runner, and one `experiment.run.completed` event of kind
/// `doctest` records the run's counts. A document that refuses the doctest
/// grammar fails the whole run before any step executes; a failing step is
/// recorded as a failed count and the run still records its fact.
pub(crate) fn cmd_doctest(out: &Path, project: &str, timeout: u64) -> Result<Envelope, Failure> {
    let documents = doctest::collect_documents(out).map_err(|err| {
        failure(
            "kaya.doctest.unknown_dir",
            format!("cannot read document directory {}: {err}", out.display()),
        )
    })?;

    let mut declared = 0usize;
    let mut passed = 0usize;
    let mut failed = 0usize;
    let mut failures: Vec<String> = Vec::new();
    let mut labels: Vec<String> = Vec::new();
    let mut report_lines = String::new();
    for path in &documents {
        let text = fs::read_to_string(path).map_err(|err| {
            failure(
                "kaya.doctest.document_io",
                format!("cannot read {}: {err}", path.display()),
            )
        })?;
        let contracts = doctest::extract(&text).map_err(|error| {
            failure(
                "kaya.doctest.grammar",
                format!("{}: {error}", path.display()),
            )
        })?;
        for contract in &contracts {
            declared += 1;
            labels.push(format!("{}#{}", path.display(), contract.label));
            match doctest::run(contract, std::time::Duration::from_secs(timeout)) {
                doctest::Outcome::Passed => {
                    passed += 1;
                    report_lines.push_str("{\"type\":\"test\",\"event\":\"ok\"}\n");
                }
                doctest::Outcome::Failed { step, reason } => {
                    failed += 1;
                    failures.push(format!(
                        "{}#{} step {}: {}",
                        path.display(),
                        contract.label,
                        step,
                        describe_failure(&reason),
                    ));
                    report_lines.push_str("{\"type\":\"test\",\"event\":\"failed\"}\n");
                }
            }
        }
    }

    let mut appended = 0usize;
    let mut duplicates = 0usize;
    if declared > 0 {
        let report_path = Path::new(".kaya/doctest-run.jsonl");
        fs::create_dir_all(KAYA_DIR).map_err(|err| {
            failure(
                "kaya.doctest.report_io",
                format!("cannot create {KAYA_DIR}: {err}"),
            )
        })?;
        fs::write(report_path, &report_lines).map_err(|err| {
            failure(
                "kaya.doctest.report_io",
                format!("cannot write {}: {err}", report_path.display()),
            )
        })?;
        let event = doctest::capture_with_kind(report_path, project, Some("doctest"))
            .map_err(adapter_failure)?;
        let mut ledger = open_ledger()?;
        match ledger.append(&event).map_err(ledger_failure)? {
            AppendOutcome::Appended => appended += 1,
            AppendOutcome::Duplicate => duplicates += 1,
        }
    }

    Ok(Envelope::ok(
        "doctest",
        json!({
            "declared": declared,
            "passed": passed,
            "failed": failed,
            "failures": failures,
            "labels": labels,
            "appended": appended,
            "duplicates": duplicates,
            "ledger": LEDGER_PATH,
        }),
    ))
}

/// Human-readable phrasing of one failed step.
fn describe_failure(reason: &doctest::Failure) -> String {
    match reason {
        doctest::Failure::Exit(code) => format!("exited with {code}"),
        doctest::Failure::Mismatch => "output did not match".to_owned(),
        doctest::Failure::Timeout => "timed out".to_owned(),
    }
}

/// Record a human decision that a doctest's expected output changed.
///
/// The human edits the document first; this command records the event
/// naming the new expected-output hash, so the ledger shows who changed a
/// contract and when. A tool or agent actor is refused here exactly as the
/// library refuses it: expected output answers to humans only.
pub(crate) fn cmd_supersede_doctest(
    document: &Path,
    label: &str,
    actor_name: &str,
    actor_type: &str,
) -> Result<Envelope, Failure> {
    if actor_type != "human" {
        return Err(failure(
            "kaya.supersede.tool_actor",
            format!(
                "actor type \"{actor_type}\" may not supersede doctest expected output; \
                 only a human can declare what a human verified"
            ),
        ));
    }
    let text = fs::read_to_string(document).map_err(|err| {
        failure(
            "kaya.supersede.document_io",
            format!("cannot read {}: {err}", document.display()),
        )
    })?;
    let contracts = doctest::extract(&text).map_err(|error| {
        failure(
            "kaya.doctest.grammar",
            format!("{}: {error}", document.display()),
        )
    })?;
    let contract = contracts
        .iter()
        .find(|contract| contract.label == label)
        .ok_or_else(|| {
            failure(
                "kaya.supersede.unknown_doctest",
                format!(
                    "document {} declares no doctest labeled \"{label}\"",
                    document.display()
                ),
            )
        })?;
    let record_id = format!("{}#{label}", document.display());
    let to_hash = doctest::contract_hash(&contract.body);
    let previous = latest_supersede(&record_id)?;
    if let Some((_, previous_hash)) = &previous
        && *previous_hash == to_hash
    {
        return Err(failure(
            "kaya.supersede.unchanged",
            format!(
                "the expected output of \"{record_id}\" already carries hash {to_hash}; \
                 edit the document first, then record the supersede"
            ),
        ));
    }
    let event = doctest::supersede_event(
        &record_id,
        &to_hash,
        previous.as_ref().map(|(id, _)| id.as_str()),
        &human_actor(actor_name),
    )
    .map_err(|error| failure(error.label, error.message))?;
    let mut ledger = open_ledger()?;
    let appended = match ledger.append(&event).map_err(ledger_failure)? {
        AppendOutcome::Appended => true,
        AppendOutcome::Duplicate => false,
    };
    Ok(Envelope::ok(
        "supersede-doctest",
        json!({
            "record": record_id,
            "to_hash": to_hash,
            "supersedes": previous.as_ref().map(|(id, _)| id),
            "event": event.id,
            "appended": appended,
            "ledger": LEDGER_PATH,
        }),
    ))
}

/// The id and recorded hash of the newest supersede event naming
/// `record_id`, if one exists.
fn latest_supersede(record_id: &str) -> Result<Option<(String, String)>, Failure> {
    let text = fs::read_to_string(LEDGER_PATH).map_err(|err| {
        failure(
            "kaya.supersede.ledger_io",
            format!("cannot read the ledger {LEDGER_PATH}: {err}"),
        )
    })?;
    let mut latest: Option<(String, String)> = None;
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let Ok(event) = parse_line(line) else {
            continue;
        };
        if event.r#type == "verification.doctest.superseded"
            && event.data.get("record").and_then(|value| value.as_str()) == Some(record_id)
        {
            let to_hash = event
                .data
                .get("to_hash")
                .and_then(|value| value.as_str())
                .unwrap_or_default()
                .to_owned();
            latest = Some((event.id, to_hash));
        }
    }
    Ok(latest)
}
