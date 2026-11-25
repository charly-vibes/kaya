//! Impact commands: the documentation impact gate and human dispositions.
//!
//! Responsibilities: hold `cmd_impact` (map a change set over the document
//! IR to impacted records and report the `pass` / `review` / `fail`
//! verdict) and `cmd_disposition` (record one human review or waiver),
//! together with their private git change-set and base-document helpers.
//!
//! Rationale: the clap argument tree stays in `main.rs`; this module carries
//! only the command bodies and their family-private helpers, sharing the
//! verified-ledger reader and IR collector kept in `main.rs` with the
//! publication family.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::Command as ProcessCommand;

use kaya::envelope::Envelope;
use kaya::event::{Event, Provenance, SCHEMA_V0, Subject, new_id, parse_line};
use kaya::impact::{self, ChangeSet};
use kaya::ledger::AppendOutcome;
use kaya::projection::report;
use serde_json::json;

use crate::{
    Failure, LEDGER_PATH, collect_document_ir, failure, human_actor, ledger_failure, open_ledger,
    read_build_ledger,
};

/// Gate a change set: map it over the document IR, derive the disposition
/// of every impacted record, run the mandatory mechanical checks, and
/// report `pass`, `review`, or `fail`.
///
/// The gate reads the same verified ledger path build reads: appended
/// corrections and observations are visible, rewritten history is refused.
/// Exit codes carry the verdict — 0 for pass and review, 2 for fail — while
/// the envelope always carries the full report.
pub(crate) fn cmd_impact(
    staged: bool,
    range: Option<&str>,
    out: &Path,
    timeout_secs: u64,
) -> Result<(Envelope, u8), Failure> {
    if staged && range.is_some() {
        return Err(failure(
            "kaya.impact.usage",
            "--staged and --range are exclusive; name exactly one change source",
        ));
    }
    if !staged && range.is_none() {
        return Err(failure(
            "kaya.impact.usage",
            "name exactly one change source: --staged or --range <revset>",
        ));
    }
    let policy = impact::Policy::load(Path::new(impact::IMPACT_RULES))
        .map_err(|error| failure(error.label, error.message))?;

    let (events, _) = read_build_ledger(None, false)?;
    let (document_ir, collector) = collect_document_ir(out, &events)?;
    let sources: Vec<(String, String)> = collector
        .sources
        .iter()
        .map(|source| (source.path.clone(), source.bytes.clone()))
        .collect();

    let base_ref = if staged {
        "HEAD".to_owned()
    } else {
        impact_base_rev(range.unwrap_or_default())?
    };
    let change_set = ChangeSet {
        paths: git_changed_paths(staged, &base_ref)?,
        event_ids: git_ledger_event_ids(staged, &base_ref)?,
    };
    let base_docs = base_documents(&sources, &base_ref)?;

    let input = impact::GateInput {
        ir: &document_ir,
        sources: &sources,
        base_docs: &base_docs,
        events: &events,
        changes: &change_set,
        policy: &policy,
        timeout: std::time::Duration::from_secs(timeout_secs),
    };
    let report = impact::evaluate(&input);
    let code = if report.result == "fail" { 2 } else { 0 };
    let payload = serde_json::to_value(&report).map_err(|error| {
        failure(
            "kaya.impact.serialize",
            format!("failed to serialize the impact report: {error}"),
        )
    })?;
    Ok((Envelope::ok("impact", payload), code))
}

/// Record one human disposition — a review or a waiver — for one impacted
/// record.
///
/// The record must exist in the current IR; the event names it with the
/// subject typed Document, carries the actor and, for a waiver, the
/// rationale; both dispositions bind the dependency digest of the record. Refusals happen
/// before any append, so a refused decision appends nothing.
#[allow(clippy::too_many_arguments)]
pub(crate) fn cmd_disposition(
    record_id: &str,
    kind: &str,
    rationale: Option<&str>,
    actor_name: &str,
    actor_type: &str,
    out: &Path,
) -> Result<Envelope, Failure> {
    if actor_type != "human" {
        return Err(failure(
            impact::LABEL_HUMAN_ACTOR,
            format!(
                "actor type \"{actor_type}\" may not record dispositions; \
                 only a human can review or waive an impact"
            ),
        ));
    }
    let (event_type, waived) = match kind {
        "reviewed" => (impact::REVIEWED_EVENT, false),
        "waived" => (impact::WAIVED_EVENT, true),
        other => {
            return Err(failure(
                "kaya.impact.unknown_disposition",
                format!("unknown disposition \"{other}\": name `reviewed` or `waived`"),
            ));
        }
    };
    if waived && rationale.is_none() {
        return Err(failure(
            "kaya.impact.rationale_required",
            "a waived disposition carries a rationale",
        ));
    }

    let mut ledger = open_ledger()?;
    let (events, _recovery) = ledger.read_recovered().map_err(ledger_failure)?;
    let (document_ir, _collector) = collect_document_ir(out, &events)?;
    let record = document_ir.records.get(record_id).ok_or_else(|| {
        failure(
            "kaya.impact.unknown_record",
            format!(
                "unknown record \"{record_id}\": a disposition names a record the \
                 document IR materializes"
            ),
        )
    })?;
    // Both dispositions bind the digest of the record as materialized now,
    // so a later walk can tell whether the human decision still covers the
    // evidence it was recorded against.
    let digest = impact::dependency_digest(record);

    let mut data = json!({ "record": record_id, "disposition": kind });
    if let Some(rationale) = rationale {
        data["rationale"] = json!(rationale);
    }
    data["digest"] = json!(digest);

    let existing = events.iter().find(|event| {
        if event.r#type != event_type
            || event.subject.r#type != report::DOCUMENT_SUBJECT
            || event.subject.id != record_id
        {
            return false;
        }
        if event.data == data {
            return true;
        }
        // A legacy reviewed event predates the digest binding: its data is
        // `{record, disposition}` only. Dedup on the candidate's digest-less
        // shape instead of appending a second event for the same decision.
        // Waivers always carried a digest, so they compare whole.
        !waived
            && event.data.get("digest").is_none()
            && event.data == json!({ "record": record_id, "disposition": kind })
    });
    if let Some(existing) = existing {
        return Ok(Envelope::ok(
            "disposition",
            json!({ "event": existing, "appended": false }),
        ));
    }

    let event = Event {
        id: new_id(),
        schema: SCHEMA_V0.to_owned(),
        r#type: event_type.to_owned(),
        timestamp: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        actor: human_actor(actor_name),
        subject: Subject {
            r#type: report::DOCUMENT_SUBJECT.to_owned(),
            id: record_id.to_owned(),
        },
        data,
        provenance: Provenance {
            tool: "kaya".to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            reference: record_id.to_owned(),
        },
        artifacts: Vec::new(),
    };
    let appended = matches!(
        ledger.append(&event).map_err(ledger_failure)?,
        AppendOutcome::Appended
    );
    Ok(Envelope::ok(
        "disposition",
        json!({ "event": event, "appended": appended }),
    ))
}

/// The base rev named by a `--range` value: `main..` names `main`; a bare
/// rev names itself. A full range with two revs is refused — the gate
/// always diffs the base against HEAD, never an arbitrary pair.
fn impact_base_rev(range: &str) -> Result<String, Failure> {
    let base = range.strip_suffix("..").unwrap_or(range);
    if base.is_empty() || base.contains("..") {
        return Err(failure(
            "kaya.impact.usage",
            "--range names the base of the comparison, e.g. `main` or `main..`; \
             the gate diffs it against HEAD",
        ));
    }
    Ok(base.to_owned())
}

/// The change set's repository paths from git: the staged diff against
/// HEAD, or the diff between the base rev and HEAD.
fn git_changed_paths(staged: bool, base: &str) -> Result<BTreeSet<String>, Failure> {
    let text = if staged {
        git_output(["diff", "--cached", "--name-only", "-z"])?
    } else {
        git_output(["diff", "--name-only", "-z", &format!("{base}..HEAD")])?
    };
    Ok(text
        .split('\0')
        .filter(|path| !path.is_empty())
        .map(str::to_owned)
        .collect())
}

/// The change set's introduced ledger event ids, read from the patch git
/// reports for the ledger itself: every added line is one event.
fn git_ledger_event_ids(staged: bool, base: &str) -> Result<BTreeSet<String>, Failure> {
    let text = if staged {
        git_output(["diff", "--cached", "--", LEDGER_PATH])?
    } else {
        git_output(["diff", &format!("{base}..HEAD"), "--", LEDGER_PATH])?
    };
    let mut ids = BTreeSet::new();
    for line in text.lines() {
        let Some(added) = line.strip_prefix('+') else {
            continue;
        };
        if added.starts_with("++") || added.trim().is_empty() {
            continue;
        }
        let Ok(event) = parse_line(added) else {
            return Err(failure(
                "kaya.impact.ledger_diff",
                format!("the ledger diff carries a line that is not an event: {added}"),
            ));
        };
        ids.insert(event.id);
    }
    Ok(ids)
}

/// The document sources at the change-set base, for regenerated-different
/// comparisons.
///
/// The base's tracked paths are read once from `git ls-tree`, so a source
/// absent at the base is absent from the map while a broken base rev — or
/// any other git failure — is a labeled refusal, never silence.
fn base_documents(
    sources: &[(String, String)],
    base: &str,
) -> Result<BTreeMap<String, String>, Failure> {
    let listing = git_output(["ls-tree", "-r", "--name-only", base])?;
    let tracked: BTreeSet<&str> = listing.lines().collect();
    let mut base_docs = BTreeMap::new();
    for (path, _) in sources {
        if !tracked.contains(path.as_str()) {
            continue;
        }
        let bytes = git_output(["show", &format!("{base}:{path}")])?;
        base_docs.insert(path.clone(), bytes);
    }
    Ok(base_docs)
}

/// Run git, returning stdout; a failed git invocation is a labeled refusal.
fn git_output<const N: usize>(args: [&str; N]) -> Result<String, Failure> {
    let output = ProcessCommand::new("git")
        .args(args)
        .output()
        .map_err(|error| {
            failure(
                "kaya.impact.git_io",
                format!("git {} could not run: {error}", args.join(" ")),
            )
        })?;
    if !output.status.success() {
        return Err(failure(
            "kaya.impact.git_io",
            format!(
                "git {} failed: {}",
                args.join(" "),
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}
