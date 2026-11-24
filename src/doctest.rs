//! Doctest execution contracts: declared examples as first-class records.
//!
//! Purpose: extract declared doctest contracts from document sources, run
//! them through a controlled runner in isolation, record results as
//! test-runner facts, and gate expected-output changes behind a
//! human-recorded supersede event.
//! Responsibilities: parse the fenced `doctest` grammar with labeled
//! refusals; execute steps in an isolated workspace under a pinned
//! normalization policy and a timeout; build deterministic `experiment.run.completed`
//! events of kind `doctest`; refuse non-human supersede actors.
//! Rationale: an example that declares a contract is an executable claim
//! about the documentation; undeclared code stays prose, and no run result
//! ever rewrites what a human declared as expected.
//!
//! Grammar limitation, pinned: an expected-output line beginning with `$ `
//! would parse as the next step, so expected output cannot contain such a
//! line; write it with a `printf` step instead.

use std::fs;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use chrono::{SecondsFormat, Utc};
use sha2::{Digest, Sha256};

use crate::event::{Actor, ActorType, Event, Provenance, SCHEMA_V0, Subject, new_id};
use crate::projection::ProjectionError;

pub use crate::adapters::testrunner::capture_with_kind;

/// A declared contract broke the doctest fence grammar.
pub const LABEL_DOCTEST_GRAMMAR: &str = "kaya.documents.doctest_grammar";
/// A declared contract named a runner outside the closed runner vocabulary.
pub const LABEL_DOCTEST_RUNNER: &str = "kaya.documents.doctest_runner";
/// A non-human actor tried to supersede doctest expected output.
pub const LABEL_DOCTEST_TOOL_SUPERSEDE: &str = "kaya.documents.doctest_tool_supersede";

/// Which controlled runner executes a declared contract.
///
/// The vocabulary is closed: a contract naming anything else is refused, so a
/// runner can never become an arbitrary program-execution surface.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Runner {
    /// Each step runs as a `bash -c` command in the isolated workspace.
    Shell,
}

impl Runner {
    /// The runner named by a declaration, or `None` outside the vocabulary.
    pub fn parse(name: &str) -> Option<Runner> {
        match name {
            "shell" => Some(Runner::Shell),
            _ => None,
        }
    }

    /// The wire spelling of this runner.
    pub fn as_str(self) -> &'static str {
        match self {
            Runner::Shell => "shell",
        }
    }
}

/// One declared step: a shell command and the stdout a passing run produces.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Step {
    /// The command line, without the `$ ` prefix.
    pub command: String,
    /// The expected stdout, compared under the pinned normalization policy.
    pub expected: String,
}

/// A declared doctest execution contract extracted from a document source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Contract {
    /// The stable label naming the contract within its document:
    /// the declared `#<label>` or the positional `doctest-<n>`.
    pub label: String,
    /// The controlled runner executing the steps.
    pub runner: Runner,
    /// The declared steps, in document order.
    pub steps: Vec<Step>,
    /// The exact fenced body bytes; hashed into the record's content hash.
    pub body: String,
}

/// Why one step failed its run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Failure {
    /// The step exited nonzero with this status code.
    Exit(i32),
    /// The step exited zero but its stdout did not match the expectation.
    Mismatch,
    /// The step exceeded the run's wall-clock budget and was killed.
    Timeout,
}

/// The terminal outcome of one controlled run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Every step matched its expectation.
    Passed,
    /// The named step failed for the recorded reason; later steps never ran.
    Failed {
        /// Zero-based index of the failing step.
        step: usize,
        /// Why the step failed.
        reason: Failure,
    },
}

/// End offset and past-newline offset of the line starting at `from`.
fn line_bounds(bytes: &[u8], from: usize) -> (usize, usize) {
    let end = bytes[from..]
        .iter()
        .position(|&byte| byte == b'\n')
        .map_or(bytes.len(), |offset| from + offset);
    (end, (end + 1).min(bytes.len()))
}

/// Extract every declared doctest contract from a document source.
///
/// Only fenced blocks whose info string starts with `doctest` declare
/// contracts; every other fence — and every byte outside them — stays prose
/// and is never parsed for contracts. Refusals are labeled and name the
/// offending declaration, so a broken contract can never silently run.
pub fn extract(document: &str) -> Result<Vec<Contract>, ProjectionError> {
    let bytes = document.as_bytes();
    let mut contracts: Vec<Contract> = Vec::new();
    let mut labels = std::collections::BTreeSet::new();
    let mut in_plain_fence = false;
    let mut position = 0usize;
    while position < bytes.len() {
        let (line_end, next) = line_bounds(bytes, position);
        let line = &document[position..line_end];

        if in_plain_fence {
            if line.trim_end() == "```" {
                in_plain_fence = false;
            }
            position = next;
            continue;
        }
        if let Some(info) = doctest_info(line) {
            let body_start = next;
            let mut close: Option<(usize, usize)> = None;
            let mut cursor = body_start;
            while cursor < bytes.len() {
                let (end, after) = line_bounds(bytes, cursor);
                if document[cursor..end].trim_end() == "```" {
                    close = Some((cursor, after));
                    break;
                }
                cursor = after;
            }
            let Some((body_end, after_close)) = close else {
                return Err(ProjectionError::new(
                    LABEL_DOCTEST_GRAMMAR,
                    format!(
                        "a ```doctest fence is never closed: \"{}\"",
                        line.trim_end()
                    ),
                ));
            };
            let body = &document[body_start..body_end];
            contracts.push(build_contract(info, body, contracts.len(), &mut labels)?);
            position = after_close;
            continue;
        }
        if line.trim_start().starts_with("```") {
            in_plain_fence = true;
        }
        position = next;
    }
    Ok(contracts)
}

/// The declaration info of a `doctest` fence opening line, if it is one.
fn doctest_info(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("```doctest")?;
    if rest.is_empty() || rest.starts_with(' ') {
        Some(rest.trim())
    } else {
        None
    }
}

/// Build one contract from its info string and fenced body.
///
/// The info string names the runner and optionally a `#<label>`; the body's
/// `$ `-prefixed lines are the steps and the lines between them are each
/// step's expected stdout. Output before the first step, a declaration with
/// no steps, an unknown runner, or a duplicated label is refused.
fn build_contract(
    info: &str,
    body: &str,
    ordinal: usize,
    labels: &mut std::collections::BTreeSet<String>,
) -> Result<Contract, ProjectionError> {
    let mut tokens = info.split_whitespace();
    let runner_name = tokens.next().ok_or_else(|| {
        ProjectionError::new(
            LABEL_DOCTEST_GRAMMAR,
            "a ```doctest fence declares no runner; name one of `shell`",
        )
    })?;
    let runner = Runner::parse(runner_name).ok_or_else(|| {
        ProjectionError::new(
            LABEL_DOCTEST_RUNNER,
            format!(
                "runner \"{runner_name}\" is outside the closed runner vocabulary (shell)"
            ),
        )
    })?;
    let mut declared_label = None;
    for token in tokens {
        let Some(name) = token.strip_prefix('#') else {
            return Err(ProjectionError::new(
                LABEL_DOCTEST_GRAMMAR,
                format!("unexpected doctest declaration token \"{token}\""),
            ));
        };
        if name.is_empty() {
            return Err(ProjectionError::new(
                LABEL_DOCTEST_GRAMMAR,
                "a ```doctest fence declares an empty # label".to_owned(),
            ));
        }
        declared_label = Some(name.to_owned());
    }
    let label = declared_label.unwrap_or_else(|| format!("doctest-{}", ordinal + 1));
    if !labels.insert(label.clone()) {
        return Err(ProjectionError::new(
            LABEL_DOCTEST_GRAMMAR,
            format!("doctest label \"{label}\" is declared more than once in one document"),
        ));
    }

    let mut steps: Vec<Step> = Vec::new();
    let mut current: Option<(String, Vec<&str>)> = None;
    for line in body.lines() {
        if let Some(command) = line.strip_prefix("$ ") {
            if let Some((command, expected)) = current.take() {
                steps.push(Step {
                    command,
                    expected: expected.join("\n"),
                });
            }
            current = Some((command.trim_end().to_owned(), Vec::new()));
        } else if let Some((_, expected)) = current.as_mut() {
            expected.push(line);
        } else if !line.trim().is_empty() {
            return Err(ProjectionError::new(
                LABEL_DOCTEST_GRAMMAR,
                format!(
                    "the doctest contract \"{label}\" carries output before its first $ step"
                ),
            ));
        }
    }
    if let Some((command, expected)) = current.take() {
        steps.push(Step {
            command,
            expected: expected.join("\n"),
        });
    }
    if steps.is_empty() {
        return Err(ProjectionError::new(
            LABEL_DOCTEST_GRAMMAR,
            format!("the doctest contract \"{label}\" declares no $ steps"),
        ));
    }
    Ok(Contract {
        label,
        runner,
        steps,
        body: body.to_owned(),
    })
}

/// Run one declared contract through its controlled runner.
///
/// Every step executes as `bash -c` inside a fresh temporary workspace, so a
/// run can never touch the document, the ledger, or the caller's tree; the
/// whole run shares one wall-clock deadline and a hung step is killed. Only
/// stdout is compared, under the pinned normalization policy: CRLF becomes
/// LF and trailing whitespace per line is dropped, so volatile formatting
/// never fails a deterministic expectation. A failing run returns the step
/// and reason and — by construction — never rewrites the contract's
/// expected output: the runner holds no path to the document bytes.
pub fn run(contract: &Contract, timeout: Duration) -> Outcome {
    let workspace = workspace_dir();
    fs::create_dir_all(&workspace).expect("create the isolated doctest workspace");
    let deadline = Instant::now() + timeout;
    let mut outcome = Outcome::Passed;
    for (index, step) in contract.steps.iter().enumerate() {
        match run_step(&step.command, &workspace, deadline) {
            Ok(actual) if normalize(&actual) == normalize(&step.expected) => {}
            Ok(_) => {
                outcome = Outcome::Failed {
                    step: index,
                    reason: Failure::Mismatch,
                };
                break;
            }
            Err(reason) => {
                outcome = Outcome::Failed {
                    step: index,
                    reason,
                };
                break;
            }
        }
    }
    fs::remove_dir_all(&workspace).ok();
    outcome
}

/// A unique, short-lived workspace directory for one run.
fn workspace_dir() -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|since| since.as_nanos())
        .unwrap_or_default();
    std::env::temp_dir().join(format!(
        "kaya-doctest-run-{}-{nanos}",
        std::process::id()
    ))
}

/// Execute one step, returning its stdout or why the step failed.
fn run_step(command: &str, workspace: &Path, deadline: Instant) -> Result<String, Failure> {
    let mut child = Command::new("bash")
        .arg("-c")
        .arg(command)
        .current_dir(workspace)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        // A spawn failure is the shell itself missing; report it under the
        // command-not-found convention so the fact stays a run fact.
        .map_err(|_| Failure::Exit(127))?;
    let mut stdout = child.stdout.take().expect("stdout is piped");
    let reader = std::thread::spawn(move || {
        let mut buffer = String::new();
        let _ = stdout.read_to_string(&mut buffer);
        buffer
    });
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(Failure::Timeout);
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(_) => return Err(Failure::Exit(-1)),
        }
    };
    let output = reader.join().unwrap_or_default();
    match status.code() {
        Some(0) => Ok(output),
        Some(code) => Err(Failure::Exit(code)),
        None => Err(Failure::Exit(-1)),
    }
}

/// The pinned normalization policy applied to both sides of a comparison.
fn normalize(output: &str) -> String {
    let unified = output.replace("\r\n", "\n");
    let trimmed: Vec<&str> = unified.lines().map(|line| line.trim_end()).collect();
    trimmed.join("\n").trim_end_matches('\n').to_owned()
}

/// Every `.md` document under `root`, recursively, in sorted order.
///
/// The compiled research tree nests experiment reports beside the top-level
/// documents, and a contract can be declared at either depth, so the walk
/// covers the whole tree rather than one directory level. Symlinks are
/// skipped outright: a symlinked directory can cycle the walk, and the
/// compiled tree has no need for either.
pub fn collect_documents(root: &Path) -> std::io::Result<Vec<std::path::PathBuf>> {
    let mut documents = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            if entry.file_type()?.is_symlink() {
                continue;
            }
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|extension| extension == "md") {
                documents.push(path);
            }
        }
    }
    documents.sort();
    Ok(documents)
}

/// The content hash of a contract's fenced body, as the IR record carries it.
pub fn contract_hash(body: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(body.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Build the human-recorded supersede event for a doctest's expected output.
///
/// Only a human may declare what a human verified: a tool or agent actor is
/// refused. The event names the IR record and the hash the expected output
/// changes to, and — when this is not the first supersede — the supersede
/// event it supersedes, so the ledger keeps the chain auditable. No run
/// result ever constructs this event.
pub fn supersede_event(
    record_id: &str,
    to_hash: &str,
    previous: Option<&str>,
    actor: &Actor,
) -> Result<Event, ProjectionError> {
    if actor.r#type != ActorType::Human {
        return Err(ProjectionError::new(
            LABEL_DOCTEST_TOOL_SUPERSEDE,
            format!(
                "actor \"{}\" of type {} may not supersede doctest expected output; only a human can declare what a human verified",
                actor.name,
                actor.r#type.as_str()
            ),
        ));
    }
    let mut data = serde_json::json!({
        "record": record_id,
        "to_hash": to_hash,
    });
    if let Some(previous) = previous {
        data["supersedes"] = serde_json::json!(previous);
    }
    Ok(Event {
        id: new_id(),
        schema: SCHEMA_V0.to_owned(),
        r#type: "verification.doctest.superseded".to_owned(),
        timestamp: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        actor: actor.clone(),
        subject: Subject {
            r#type: "Doctest".to_owned(),
            id: record_id.to_owned(),
        },
        data,
        provenance: Provenance {
            tool: "kaya".to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            reference: record_id.to_owned(),
        },
        artifacts: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};
    use std::{fs, path::PathBuf, process};

    fn fence(runner_info: &str, body: &str) -> String {
        format!("```doctest {runner_info}\n{body}```\n")
    }

    fn authored(fence_block: &str) -> String {
        format!("# Notes\n\nProse.\n\n{fence_block}\n")
    }

    fn temp_dir(name: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("kaya-doctest-{name}-{}-{nanos}", process::id()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    // --- grammar and extraction ---

    #[test]
    fn extract_parses_declared_contract() {
        let document = authored(&fence(
            "shell",
            "$ echo hello\nhello\n$ echo bye\nbye\n",
        ));
        let contracts = extract(&document).expect("the declared contract extracts");
        assert_eq!(contracts.len(), 1);
        let contract = &contracts[0];
        assert_eq!(contract.label, "doctest-1");
        assert_eq!(contract.runner, Runner::Shell);
        assert_eq!(contract.steps.len(), 2);
        assert_eq!(contract.steps[0].command, "echo hello");
        assert_eq!(contract.steps[0].expected, "hello");
        assert_eq!(contract.steps[1].command, "echo bye");
        assert_eq!(contract.steps[1].expected, "bye");
        assert_eq!(contract.body, "$ echo hello\nhello\n$ echo bye\nbye\n");
    }

    #[test]
    fn declared_label_is_used_verbatim() {
        let document = authored(&fence(
            "shell #build-check",
            "$ echo hello\nhello\n",
        ));
        let contracts = extract(&document).expect("the declared contract extracts");
        assert_eq!(contracts[0].label, "build-check");
    }

    #[test]
    fn undeclared_fences_are_ignored() {
        let document = authored("```bash\n$ echo hello\n```\n");
        let contracts = extract(&document).expect("plain fences never refuse");
        assert!(contracts.is_empty());
    }

    #[test]
    fn extract_refuses_unterminated_fence() {
        let document = authored("```doctest shell\n$ echo hello\n");
        let error = extract(&document).expect_err("an unterminated fence refuses");
        assert!(error.label.contains("doctest"), "label: {}", error.label);
        assert!(!error.message.is_empty());
    }

    #[test]
    fn extract_refuses_unknown_runner() {
        let document = authored(&fence("python", "$ print('hi')\nhi\n"));
        let error = extract(&document).expect_err("an unlisted runner refuses");
        assert!(error.message.contains("python"), "message: {}", error.message);
    }

    #[test]
    fn extract_refuses_contract_without_steps() {
        let document = authored(&fence("shell", "just output\n"));
        let error = extract(&document).expect_err("a contract without steps refuses");
        assert!(error.label.contains("doctest"), "label: {}", error.label);
    }

    #[test]
    fn extract_refuses_duplicate_label() {
        let one = fence("shell #dup", "$ echo a\na\n");
        let two = fence("shell #dup", "$ echo b\nb\n");
        let document = authored(&format!("{one}\n{two}\n"));
        let error = extract(&document).expect_err("a duplicated label refuses");
        assert!(error.message.contains("dup"), "message: {}", error.message);
    }

    // --- controlled runner ---

    #[test]
    fn passing_run_completes() {
        let document = authored(&fence("shell", "$ echo hello\nhello\n"));
        let contract = &extract(&document).expect("the contract extracts")[0];
        let outcome = run(contract, Duration::from_secs(10));
        assert_eq!(outcome, Outcome::Passed);
    }

    #[test]
    fn mismatched_output_fails_with_the_step() {
        let document = authored(&fence("shell", "$ echo wrong\nhello\n"));
        let contract = &extract(&document).expect("the contract extracts")[0];
        let outcome = run(contract, Duration::from_secs(10));
        assert_eq!(
            outcome,
            Outcome::Failed {
                step: 0,
                reason: Failure::Mismatch
            }
        );
    }

    #[test]
    fn nonzero_exit_fails_with_the_step() {
        let document = authored(&fence("shell", "$ exit 3\nnever\n"));
        let contract = &extract(&document).expect("the contract extracts")[0];
        let outcome = run(contract, Duration::from_secs(10));
        assert_eq!(
            outcome,
            Outcome::Failed {
                step: 0,
                reason: Failure::Exit(3)
            }
        );
    }

    #[test]
    fn timeout_kills_a_hung_step() {
        let document = authored(&fence("shell", "$ sleep 30\nnever\n"));
        let contract = &extract(&document).expect("the contract extracts")[0];
        let started = Instant::now();
        let outcome = run(contract, Duration::from_millis(200));
        assert_eq!(
            outcome,
            Outcome::Failed {
                step: 0,
                reason: Failure::Timeout
            }
        );
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "the timeout must actually bound the run"
        );
    }

    #[test]
    fn normalization_ignores_trailing_whitespace_and_crlf() {
        let document = authored(&fence("shell", "$ printf 'hello  \\r\\n'\nhello\n"));
        let contract = &extract(&document).expect("the contract extracts")[0];
        let outcome = run(contract, Duration::from_secs(10));
        assert_eq!(outcome, Outcome::Passed);
    }

    #[test]
    fn failing_run_never_rewrites_expected_output() {
        let dir = temp_dir("no-rewrite");
        let path = dir.join("index.md");
        let document = authored(&fence("shell", "$ echo wrong\nhello\n"));
        fs::write(&path, &document).unwrap();
        let read_back = fs::read_to_string(&path).unwrap();
        let contract = &extract(&read_back).expect("the contract extracts")[0];
        let outcome = run(contract, Duration::from_secs(10));
        assert!(matches!(outcome, Outcome::Failed { .. }));
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            document,
            "a failed run must leave the document bytes identical"
        );
        fs::remove_dir_all(&dir).ok();
    }

    // --- facts ---

    #[test]
    fn doctest_counts_under_test_runner() {
        let dir = temp_dir("facts");
        let report = dir.join("doctest-report.jsonl");
        fs::write(
            &report,
            "{\"type\":\"test\",\"event\":\"ok\"}\n{\"type\":\"test\",\"event\":\"ok\"}\n{\"type\":\"test\",\"event\":\"failed\"}\n",
        )
        .unwrap();
        let event =
            capture_with_kind(&report, "kaya", Some("doctest")).expect("the fact captures");
        assert_eq!(event.r#type, "experiment.run.completed");
        assert_eq!(event.data.get("kind"), Some(&serde_json::json!("doctest")));
        assert_eq!(event.data.get("passed"), Some(&serde_json::json!(2)));
        assert_eq!(event.data.get("failed"), Some(&serde_json::json!(1)));
        let replay =
            capture_with_kind(&report, "kaya", Some("doctest")).expect("the replay captures");
        assert_eq!(event.id, replay.id, "replaying a report is deterministic");

        let documents = std::collections::BTreeMap::new();
        let facts = crate::verification::facts::Facts::derive(
            &[event],
            "digest".to_string(),
            &documents,
            &[],
        )
        .expect("the facts derive");
        assert_eq!(facts.events_by_source.get("test-runner"), Some(&1));
        let named: usize = facts.events_by_source.values().sum();
        assert_eq!(named, 1, "no other source names the doctest run");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn libtest_capture_carries_no_kind() {
        let dir = temp_dir("no-kind");
        let report = dir.join("libtest.jsonl");
        fs::write(&report, "{\"type\":\"test\",\"event\":\"ok\"}\n").unwrap();
        let event = crate::adapters::testrunner::capture(&report, "kaya")
            .expect("the libtest capture works");
        assert!(event.data.get("kind").is_none());
        fs::remove_dir_all(&dir).ok();
    }

    // --- supersede ---

    #[test]
    fn document_gate_refuses_a_malformed_doctest() {
        let document = authored(&fence("python", "$ print('hi')\nhi\n"));
        let error = crate::projection::validate_generated_blocks(&document)
            .expect_err("the document gate refuses an unlisted runner");
        assert!(
            error.message.contains("python"),
            "message: {}",
            error.message
        );
        let healthy = authored(&fence("shell", "$ echo hello\nhello\n"));
        crate::projection::validate_generated_blocks(&healthy)
            .expect("the document gate admits a well-formed contract");
    }

    #[test]
    fn document_walk_is_recursive_and_sorted() {
        let dir = temp_dir("walk");
        let nested = dir.join("experiments");
        fs::create_dir_all(&nested).unwrap();
        fs::write(dir.join("b.md"), "b").unwrap();
        fs::write(nested.join("c.md"), "c").unwrap();
        fs::write(dir.join("a.txt"), "not a document").unwrap();
        let found = collect_documents(&dir).expect("the walk succeeds");
        let names: Vec<String> = found
            .iter()
            .filter_map(|path| path.file_name().map(|name| name.to_string_lossy().to_string()))
            .collect();
        assert_eq!(names, vec!["b.md", "c.md"], "nested .md found, sorted, no .txt");
        fs::remove_dir_all(&dir).ok();
    }

    #[cfg(unix)]
    #[test]
    fn document_walk_skips_symlinks() {
        use std::os::unix::fs::symlink;
        let dir = temp_dir("walk-symlink");
        fs::write(dir.join("a.md"), "a").unwrap();
        symlink(&dir, dir.join("cycle")).unwrap();
        let found = collect_documents(&dir).expect("the walk terminates despite the cycle");
        assert_eq!(found.len(), 1, "the symlinked directory is skipped");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn supersede_appends_through_the_ledger() {
        let dir = temp_dir("supersede-ledger");
        let actor = Actor {
            r#type: ActorType::Human,
            name: "sasha".to_string(),
        };
        let hash = contract_hash("$ echo hello\nhello\n");
        let first =
            supersede_event("docs/research/index.md#doctest-1", &hash, None, &actor)
                .expect("a human may supersede");
        // Ledger::open refuses a nonexistent record, exactly like the CLI does
        // before `kaya init`; an empty file is an initialized empty ledger.
        fs::File::create(dir.join("ledger.jsonl")).unwrap();
        let mut ledger =
            crate::ledger::Ledger::open(dir.join("ledger.jsonl")).expect("the ledger opens");
        assert!(matches!(
            ledger.append(&first).expect("the first supersede appends"),
            crate::ledger::AppendOutcome::Appended
        ));
        let second = supersede_event(
            "docs/research/index.md#doctest-1",
            &hash,
            Some(&first.id),
            &actor,
        )
        .expect("a chained supersede builds");
        assert!(matches!(
            ledger.append(&second).expect("the chained supersede appends"),
            crate::ledger::AppendOutcome::Appended
        ));
        drop(ledger);
        crate::ledger::Ledger::open(dir.join("ledger.jsonl"))
            .expect("the ledger reloads with both supersede events");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn supersede_records_a_human_event() {
        let actor = Actor {
            r#type: ActorType::Human,
            name: "sasha".to_string(),
        };
        let event = supersede_event(
            "docs/research/index.md#doctest-1",
            &contract_hash("$ echo hello\nhello\n"),
            Some("evt-previous"),
            &actor,
        )
        .expect("a human may supersede");
        assert_eq!(event.r#type, "verification.doctest.superseded");
        assert_eq!(event.subject.r#type, "Doctest");
        assert_eq!(event.subject.id, "docs/research/index.md#doctest-1");
        assert_eq!(
            event.data.get("to_hash"),
            Some(&serde_json::json!(contract_hash("$ echo hello\nhello\n")))
        );
        assert_eq!(event.data.get("supersedes"), Some(&serde_json::json!("evt-previous")));
    }

    #[test]
    fn supersede_without_previous_names_no_event() {
        let actor = Actor {
            r#type: ActorType::Human,
            name: "sasha".to_string(),
        };
        let event = supersede_event(
            "docs/research/index.md#doctest-1",
            &contract_hash("$ echo hi\nhi\n"),
            None,
            &actor,
        )
        .expect("a first supersede needs no previous");
        assert!(event.data.get("supersedes").is_none());
    }

    #[test]
    fn supersede_refuses_a_tool_actor() {
        let actor = Actor {
            r#type: ActorType::Tool,
            name: "kaya".to_string(),
        };
        let error = supersede_event(
            "docs/research/index.md#doctest-1",
            &contract_hash("$ echo hi\nhi\n"),
            None,
            &actor,
        )
        .expect_err("a tool may not supersede expected output");
        assert!(
            error.label.contains("tool") || error.label.contains("supersede"),
            "label: {}",
            error.label
        );
    }

    #[test]
    fn supersede_refuses_an_agent_actor() {
        let actor = Actor {
            r#type: ActorType::Agent,
            name: "orchestrator".to_string(),
        };
        let error = supersede_event(
            "docs/research/index.md#doctest-1",
            &contract_hash("$ echo hi\nhi\n"),
            None,
            &actor,
        )
        .expect_err("an agent may not supersede expected output");
        assert!(
            error.label.contains("tool") || error.label.contains("supersede"),
            "label: {}",
            error.label
        );
    }
}
