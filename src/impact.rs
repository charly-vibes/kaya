//! The documentation impact gate (`kaya.impact`): the change set a commit or
//! range introduces, mapped over document-IR dependency edges to the records
//! it impacts, dispositioned from a closed vocabulary, checked mechanically,
//! and gated into a deterministic `pass` / `review` / `fail` verdict.
//!
//! Responsibilities: change-set mapping with edge and rule attribution from
//! a repository-maintained policy file, dependency digests for waiver
//! binding, disposition derivation (waiver, then reviewed, then updated,
//! then validated), stale authored blocks as unmet obligations, and the
//! deterministic report.
//!
//! Rationale: hard blockers are reproducible results only — a failed
//! mechanical check, a missing or invalid disposition, an unacknowledged
//! stale block — while advisory rule findings produce review obligations,
//! because an unreviewable gate trains developers to bypass it.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::time::Duration;

use serde::Serialize;

use crate::doctest::{self, Failure, Outcome};
use crate::event::Event;
use crate::ledger::{changed_event_ids, sha256_hex};
use crate::projection::ir::{DocumentIr, Record, RecordKind, named_blocks};
use crate::projection::report;

/// Events recording a human's review of one impacted record. The
/// `verification.` family is ignored by the research graph, and the ledger
/// grammar requires at least three dotted segments.
pub const REVIEWED_EVENT: &str = "verification.record.reviewed";
/// Events recording a human's waiver of one impacted record.
pub const WAIVED_EVENT: &str = "verification.record.waived";
/// Refusal for a disposition recorded by anything but a human.
pub const LABEL_HUMAN_ACTOR: &str = "kaya.impact.human_actor_required";
/// Refusal when the repository carries no impact policy.
pub const LABEL_POLICY_MISSING: &str = "kaya.impact.policy_missing";
/// The repository-maintained policy file the gate reads, relative to the
/// working directory.
pub const IMPACT_RULES: &str = "impact-rules.toml";

/// A broken or unreadable impact policy.
#[derive(Debug)]
pub struct PolicyError {
    /// The labeled refusal class.
    pub label: String,
    /// The human-readable cause.
    pub message: String,
}

impl PolicyError {
    fn new(label: &str, message: impl Into<String>) -> Self {
        Self {
            label: label.to_owned(),
            message: message.into(),
        }
    }
}

/// How binding a fired rule is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    /// Its impacts demand dispositions.
    Mandatory,
    /// Its impacts are advisory findings: reported, never blocking.
    Advisory,
}

impl Severity {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "mandatory" => Some(Self::Mandatory),
            "advisory" => Some(Self::Advisory),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Mandatory => "mandatory",
            Self::Advisory => "advisory",
        }
    }
}

/// One explicit project policy rule: change paths matching `matches` impact
/// records under document paths matching `targets`.
#[derive(Clone, Debug)]
pub struct Rule {
    /// The stable rule id the report attributes impacts to.
    pub id: String,
    /// Glob over change-set paths that fires the rule.
    pub matches: String,
    /// Glob over record document paths the fired rule selects.
    pub targets: String,
    /// How binding the rule is.
    pub severity: Severity,
}

/// A rule under construction while the policy text parses. `severity` stays
/// absent until the text assigns it, so an unnamed severity is a refusal —
/// never a silent default that turns an advisory rule into a hard blocker.
#[derive(Default)]
struct PartialRule {
    id: Option<String>,
    matches: Option<String>,
    targets: Option<String>,
    severity: Option<Severity>,
}

/// The repository-maintained impact policy: the rules beyond dependency
/// edges, in file order.
#[derive(Clone, Debug, Default)]
pub struct Policy {
    /// The declared rules, in declaration order.
    pub rules: Vec<Rule>,
}

impl Policy {
    /// Load the policy from `path`. A missing file is a refusal: impact
    /// policy is explicit, never implicit.
    pub fn load(path: &Path) -> Result<Self, PolicyError> {
        let text = std::fs::read_to_string(path).map_err(|error| {
            PolicyError::new(
                LABEL_POLICY_MISSING,
                format!("cannot read the impact policy {}: {error}", path.display()),
            )
        })?;
        Self::parse(&text)
    }

    /// Parse the policy text: `[[rule]]` tables of `id`, `match`, `targets`,
    /// and `severity` string assignments; full-line `#` comments and `#`
    /// comments after a quoted value.
    pub fn parse(text: &str) -> Result<Self, PolicyError> {
        let mut rules: Vec<Option<PartialRule>> = Vec::new();
        for (number, raw) in text.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if line == "[[rule]]" {
                rules.push(None);
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                return Err(Self::refuse(
                    number,
                    raw,
                    "is not a `key = \"value\"` assignment",
                ));
            };
            let value = Self::assignment_value(number, raw, value.trim())?;
            let Some(rule) = rules.last_mut() else {
                return Err(Self::refuse(
                    number,
                    raw,
                    "assigns outside any [[rule]] table",
                ));
            };
            let field = key.trim();
            let bound = rule.get_or_insert_with(PartialRule::default);
            match field {
                "id" => bound.id = Some(value),
                "match" => bound.matches = Some(value),
                "targets" => bound.targets = Some(value),
                "severity" => {
                    bound.severity = Some(Severity::parse(&value).ok_or_else(|| {
                        PolicyError::new(
                            "kaya.impact.policy_severity",
                            format!(
                                "line {}: severity \"{value}\" is not one of \
                                 {{mandatory, advisory}}",
                                number + 1
                            ),
                        )
                    })?);
                }
                _ => {
                    return Err(Self::refuse(
                        number,
                        raw,
                        "names a field outside {id, match, targets, severity}",
                    ));
                }
            }
        }

        let mut complete = Vec::new();
        let mut ids = BTreeSet::new();
        for partial in rules.into_iter().flatten() {
            let Some(id) = partial.id else {
                return Err(PolicyError::new(
                    "kaya.impact.policy_field",
                    "a rule declares no id",
                ));
            };
            let Some(matches) = partial.matches else {
                return Err(PolicyError::new(
                    "kaya.impact.policy_field",
                    format!("rule \"{id}\" declares no match"),
                ));
            };
            let Some(targets) = partial.targets else {
                return Err(PolicyError::new(
                    "kaya.impact.policy_field",
                    format!("rule \"{id}\" declares no targets"),
                ));
            };
            let Some(severity) = partial.severity else {
                return Err(PolicyError::new(
                    "kaya.impact.policy_field",
                    format!(
                        "rule \"{id}\" declares no severity: name `mandatory` \
                         or `advisory` explicitly"
                    ),
                ));
            };
            if !ids.insert(id.clone()) {
                return Err(PolicyError::new(
                    "kaya.impact.policy_field",
                    format!("rule id \"{id}\" is declared more than once"),
                ));
            }
            complete.push(Rule {
                id,
                matches,
                targets,
                severity,
            });
        }
        Ok(Self { rules: complete })
    }

    /// One assignment's value: a quoted string with an optional trailing `#`
    /// comment, or a bare word ending at a comment. Anything after the
    /// closing quote that is not a comment is refused, so a typo can never
    /// silently become part of a glob.
    fn assignment_value(number: usize, raw: &str, value: &str) -> Result<String, PolicyError> {
        let Some(rest) = value.strip_prefix('"') else {
            let bare = value.split('#').next().unwrap_or("").trim();
            if bare.is_empty() {
                return Err(Self::refuse(number, raw, "assigns an empty value"));
            }
            return Ok(bare.to_owned());
        };
        let Some(closing) = rest.find('"') else {
            return Err(Self::refuse(number, raw, "assigns an unterminated string"));
        };
        let inner = &rest[..closing];
        let trailing = rest[closing + 1..].trim();
        if !trailing.is_empty() && !trailing.starts_with('#') {
            return Err(Self::refuse(
                number,
                raw,
                "carries text after the closing quote that is not a `#` comment",
            ));
        }
        Ok(inner.to_owned())
    }

    fn refuse(number: usize, raw: &str, cause: &str) -> PolicyError {
        PolicyError::new(
            "kaya.impact.policy_grammar",
            format!("line {}: \"{raw}\" {cause}", number + 1),
        )
    }
}

/// What identified an impacted record: the dependency edge or the policy
/// rule that pulled it in.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    /// The dependency edge (`path:<p>`, `event:<id>`, `contains:<record>`).
    Edge(String),
    /// The policy rule id that fired.
    Rule(String),
}

/// The impact set: every record the change set touches, with its reason.
#[derive(Debug, Default)]
pub struct ImpactMap {
    /// Impacted record ids, each naming what identified it.
    pub impacts: BTreeMap<String, Reason>,
}

/// A change set: the paths and ledger event ids a commit or range introduces.
#[derive(Debug, Default, Serialize)]
pub struct ChangeSet {
    /// Changed repository paths.
    pub paths: BTreeSet<String>,
    /// Introduced ledger event ids.
    pub event_ids: BTreeSet<String>,
}

/// Map a change set over the document IR: the records whose dependency edges
/// intersect it, directly or transitively, plus the mandatory policy rules
/// that fired.
///
/// A record's dependency edges are its document path, its source artifact
/// references, and the event ids it cites or carries provenance for;
/// containment composes — a record depends on every record it contains.
/// Direct edges win over rule attribution, which wins over transitive
/// containment.
pub fn map_impacts(ir: &DocumentIr, changes: &ChangeSet, policy: &Policy) -> ImpactMap {
    let mut impacts: BTreeMap<String, Reason> = BTreeMap::new();

    for (id, record) in &ir.records {
        let path = document_path(id);
        if changes.paths.contains(path) {
            impacts.insert(id.clone(), Reason::Edge(format!("path:{path}")));
            continue;
        }
        if let Some(artifact) = record
            .artifacts
            .iter()
            .find(|artifact| changes.paths.contains(*artifact))
        {
            impacts.insert(id.clone(), Reason::Edge(format!("path:{artifact}")));
            continue;
        }
        if let Some(event) = record
            .cites
            .iter()
            .chain(record.provenance.iter())
            .find(|event| changes.event_ids.contains(*event))
        {
            impacts.insert(id.clone(), Reason::Edge(format!("event:{event}")));
        }
    }

    for rule in &policy.rules {
        if rule.severity != Severity::Mandatory {
            continue;
        }
        if !changes
            .paths
            .iter()
            .any(|path| glob_match(&rule.matches, path))
        {
            continue;
        }
        let targets: Vec<String> = ir
            .records
            .keys()
            .filter(|id| glob_match(&rule.targets, document_path(id)))
            .cloned()
            .collect();
        for id in targets {
            impacts
                .entry(id)
                .or_insert_with(|| Reason::Rule(rule.id.clone()));
        }
    }

    loop {
        let mut grew = false;
        for (id, record) in &ir.records {
            if impacts.contains_key(id) {
                continue;
            }
            let pulled = record.contains.iter().find(|contained| {
                matches!(
                    impacts.get(*contained),
                    Some(Reason::Edge(_)) | Some(Reason::Rule(_))
                )
            });
            if let Some(contained) = pulled {
                impacts.insert(id.clone(), Reason::Edge(format!("contains:{contained}")));
                grew = true;
            }
        }
        if !grew {
            break;
        }
    }

    ImpactMap { impacts }
}

/// The document path of a record id: everything before the last `#`, or the
/// whole id for composed page ids.
fn document_path(id: &str) -> &str {
    id.rsplit_once('#').map_or(id, |(path, _)| path)
}

/// The digest binding a waiver: SHA-256 over the record's canonical
/// dependency edges (its document path, artifact references, and cited
/// event ids). A waiver lapses when this digest changes.
pub fn dependency_digest(record: &Record) -> String {
    let mut edges: Vec<String> = vec![format!("path:{}", document_path(&record.id))];
    edges.extend(
        record
            .artifacts
            .iter()
            .map(|artifact| format!("path:{artifact}")),
    );
    edges.extend(record.cites.iter().map(|cite| format!("event:{cite}")));
    edges.sort();
    edges.dedup();
    sha256_hex(edges.join("\n").as_bytes())
}

/// Minimal glob matching over `/`-separated paths: `*` within a segment,
/// `**` across segments, everything else literal.
fn glob_match(pattern: &str, value: &str) -> bool {
    let (head, rest) = split_segment(pattern);
    match head {
        Some("**") => {
            let Some(rest) = rest else { return true };
            let mut candidate = value;
            loop {
                if glob_match(rest, candidate) {
                    return true;
                }
                match candidate.split_once('/') {
                    Some((_, tail)) => candidate = tail,
                    None => return false,
                }
            }
        }
        Some(head) => {
            let (value_head, value_rest) = split_segment(value);
            let Some(value_head) = value_head else {
                return false;
            };
            if !segment_match(head, value_head) {
                return false;
            }
            match (rest, value_rest) {
                (None, None) => true,
                (Some(rest), Some(value_rest)) => glob_match(rest, value_rest),
                _ => false,
            }
        }
        None => value.is_empty(),
    }
}

fn split_segment(pattern: &str) -> (Option<&str>, Option<&str>) {
    match pattern.split_once('/') {
        Some((head, rest)) => (Some(head), Some(rest)),
        None => (Some(pattern), None),
    }
}

fn segment_match(pattern: &str, value: &str) -> bool {
    match pattern.split_once('*') {
        None => pattern == value,
        Some((prefix, suffix)) => {
            value.len() >= prefix.len() + suffix.len()
                && value.starts_with(prefix)
                && value.ends_with(suffix)
        }
    }
}

/// One mechanical check the gate ran against an impacted record.
#[derive(Serialize)]
pub struct CheckView {
    /// The checked record id.
    pub record: String,
    /// The check kind (today only the doctest run).
    pub check: &'static str,
    /// `passed` or `failed`.
    pub outcome: &'static str,
    /// Why a check failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// One waiver and whether it still holds.
#[derive(Serialize)]
pub struct WaiverView {
    /// The dependency digest the waiver bound.
    pub digest: String,
    /// Whether the record's dependencies changed since.
    pub lapsed: bool,
}

/// One advisory finding: a rule that fired without blocking.
#[derive(Serialize)]
pub struct Finding {
    /// The advisory rule id.
    pub rule: String,
    /// The record ids the rule's targets selected.
    pub records: Vec<String>,
}

/// One fired policy rule.
#[derive(Serialize)]
pub struct RuleFire {
    /// The rule id.
    pub rule: String,
    /// How binding the rule is.
    pub severity: &'static str,
    /// The record ids the rule's targets selected.
    pub records: Vec<String>,
}

/// One impacted record and what identified it.
#[derive(Serialize)]
pub struct ImpactView {
    /// The impacted record id.
    pub record: String,
    /// The edge or rule that identified it.
    pub identified_by: Reason,
}

/// One unmet obligation.
#[derive(Serialize)]
pub struct UnmetView {
    /// The record owing the obligation.
    pub record: String,
    /// `missing_disposition`, `stale_block`, `lapsed_waiver`, or
    /// `failed_check`.
    pub reason: String,
}

/// The deterministic gate report: no run timestamps, no environment details.
#[derive(Serialize)]
pub struct Report<'a> {
    /// The change set the gate mapped.
    pub change_set: &'a ChangeSet,
    /// Every policy rule that fired.
    pub rules_fired: Vec<RuleFire>,
    /// Mechanical checks over impacted records, with results.
    pub checks: Vec<CheckView>,
    /// The effective disposition per dispositioned record.
    pub dispositions: BTreeMap<String, String>,
    /// Every recorded waiver over the ledger's records, with lapsed state.
    pub waivers: BTreeMap<String, WaiverView>,
    /// Advisory findings pending.
    pub findings: Vec<Finding>,
    /// The impact set, each record naming its identifying edge or rule.
    pub impacts: Vec<ImpactView>,
    /// Every unmet mandatory obligation.
    pub unmet: Vec<UnmetView>,
    /// `pass`, `review`, or `fail`.
    pub result: &'static str,
}

/// Everything one gate evaluation reads.
pub struct GateInput<'a> {
    /// The document IR of the current state.
    pub ir: &'a DocumentIr,
    /// The current document sources as `(path, bytes)`.
    pub sources: &'a [(String, String)],
    /// The document sources at the change-set base, as `(path, bytes)`.
    pub base_docs: &'a BTreeMap<String, String>,
    /// The ledger events.
    pub events: &'a [Event],
    /// The change set.
    pub changes: &'a ChangeSet,
    /// The repository impact policy.
    pub policy: &'a Policy,
    /// Wall-clock budget for one doctest run.
    pub timeout: Duration,
}

/// Evaluate the gate: map, disposition, check, and grade.
///
/// Precedence per impact: a stale authored block is an unmet obligation
/// regardless of anything; a valid waiver holds; a lapsed waiver is an unmet
/// obligation again; a recorded review holds; regenerated-different bytes
/// establish `updated`; green mandatory checks establish `validated`;
/// anything else is a missing disposition.
pub fn evaluate<'a>(input: &'a GateInput<'a>) -> Report<'a> {
    let mapped = map_impacts(input.ir, input.changes, input.policy);
    let changed_ids = changed_event_ids(input.events);

    let mut rules_fired = Vec::new();
    let mut findings = Vec::new();
    for rule in &input.policy.rules {
        if !input
            .changes
            .paths
            .iter()
            .any(|path| glob_match(&rule.matches, path))
        {
            continue;
        }
        let records: Vec<String> = input
            .ir
            .records
            .keys()
            .filter(|id| glob_match(&rule.targets, document_path(id)))
            .cloned()
            .collect();
        if rule.severity == Severity::Advisory {
            findings.push(Finding {
                rule: rule.id.clone(),
                records: records.clone(),
            });
        }
        rules_fired.push(RuleFire {
            rule: rule.id.clone(),
            severity: rule.severity.as_str(),
            records,
        });
    }

    // Staleness is ledger state, not change-set state: the same whole-ledger
    // view build and check use, narrowed to acknowledgments that no longer
    // cover the changed cited ids.
    let mut stale_ids = BTreeSet::new();
    for (path, bytes) in input.sources {
        let acknowledged = report::acknowledged(input.events, path);
        let stale = report::stale_authored(bytes, &changed_ids, &acknowledged);
        for block in report::unacknowledged(stale, &acknowledged) {
            stale_ids.insert(format!("{path}#{}", ir_block_name(&block.block)));
        }
    }

    let mut checks = Vec::new();
    let mut validated: BTreeSet<String> = BTreeSet::new();
    for id in mapped.impacts.keys() {
        let Some(record) = input.ir.records.get(id) else {
            continue;
        };
        if !matches!(record.kind, RecordKind::Doctest { .. }) {
            continue;
        }
        let (outcome, reason) = run_doctest(id, input.sources, input.timeout);
        if outcome == "passed" {
            validated.insert(id.clone());
        }
        checks.push(CheckView {
            record: id.clone(),
            check: "doctest",
            outcome,
            reason,
        });
    }

    // Every recorded waiver, graded against the record's current dependency
    // digest; the latest event per record wins.
    let mut waivers: BTreeMap<String, WaiverView> = BTreeMap::new();
    for event in input.events {
        if event.r#type != WAIVED_EVENT || event.subject.r#type != report::DOCUMENT_SUBJECT {
            continue;
        }
        let Some(record_id) = event.data.get("record").and_then(serde_json::Value::as_str) else {
            continue;
        };
        let digest = event
            .data
            .get("digest")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_owned();
        let lapsed = input
            .ir
            .records
            .get(record_id)
            .map(|record| dependency_digest(record) != digest)
            .unwrap_or(true);
        waivers.insert(
            record_id.to_owned(),
            WaiverView {
                digest: digest.clone(),
                lapsed,
            },
        );
    }

    // The latest recorded review per record.
    let mut reviewed: BTreeSet<String> = BTreeSet::new();
    for event in input.events {
        if event.r#type != REVIEWED_EVENT || event.subject.r#type != report::DOCUMENT_SUBJECT {
            continue;
        }
        if let Some(record_id) = event.data.get("record").and_then(serde_json::Value::as_str) {
            reviewed.insert(record_id.to_owned());
        }
    }

    let mut dispositions = BTreeMap::new();
    // Stale authored blocks are ledger state, not change-set state: they
    // block the gate even when the change set touches only the ledger.
    let mut unmet: Vec<UnmetView> = stale_ids
        .iter()
        .map(|id| UnmetView {
            record: id.clone(),
            reason: "stale_block".to_owned(),
        })
        .collect();
    let failed_checks: BTreeSet<String> = checks
        .iter()
        .filter(|check| check.outcome == "failed")
        .map(|check| check.record.clone())
        .collect();
    unmet.extend(failed_checks.iter().map(|id| UnmetView {
        record: id.clone(),
        reason: "failed_check".to_owned(),
    }));
    for id in mapped.impacts.keys() {
        if stale_ids.contains(id) || failed_checks.contains(id) {
            // Already reported as an unmet obligation; no disposition can
            // stand in for a red check or an uncovered stale block.
            continue;
        }
        if let Some(waiver) = waivers.get(id) {
            if !waiver.lapsed {
                dispositions.insert(id.clone(), "waived".to_owned());
            } else {
                unmet.push(UnmetView {
                    record: id.clone(),
                    reason: "lapsed_waiver".to_owned(),
                });
            }
            continue;
        }
        if reviewed.contains(id) {
            dispositions.insert(id.clone(), "reviewed".to_owned());
            continue;
        }
        let record = &input.ir.records[id];
        if regenerates_differently(record, input.sources, input.base_docs) {
            dispositions.insert(id.clone(), "updated".to_owned());
            continue;
        }
        if validated.contains(id) {
            dispositions.insert(id.clone(), "validated".to_owned());
            continue;
        }
        unmet.push(UnmetView {
            record: id.clone(),
            reason: "missing_disposition".to_owned(),
        });
    }

    unmet.sort_by(|left, right| (&left.record, &left.reason).cmp(&(&right.record, &right.reason)));
    let result = if !unmet.is_empty() {
        "fail"
    } else if !findings.is_empty() {
        "review"
    } else {
        "pass"
    };

    Report {
        change_set: input.changes,
        rules_fired,
        checks,
        dispositions,
        waivers,
        findings,
        impacts: mapped
            .impacts
            .into_iter()
            .map(|(record, identified_by)| ImpactView {
                record,
                identified_by,
            })
            .collect(),
        unmet,
        result,
    }
}

/// Run one doctest contract through the controlled runner.
///
/// The record id names its document path and contract label; a contract the
/// current sources no longer declare is a failed check, never a silent pass.
fn run_doctest(
    record_id: &str,
    sources: &[(String, String)],
    timeout: Duration,
) -> (&'static str, Option<String>) {
    let Some((path, name)) = record_id.rsplit_once('#') else {
        return ("failed", Some("missing_contract".to_owned()));
    };
    let Some((_, bytes)) = sources.iter().find(|(source, _)| source == path) else {
        return ("failed", Some("missing_contract".to_owned()));
    };
    let contract = doctest::extract(bytes).ok().and_then(|contracts| {
        contracts
            .into_iter()
            .find(|contract| contract.label == name)
    });
    let Some(contract) = contract else {
        return ("failed", Some("missing_contract".to_owned()));
    };
    match doctest::run(&contract, timeout) {
        Outcome::Passed => ("passed", None),
        Outcome::Failed { reason, .. } => {
            let reason = match reason {
                Failure::Exit(code) => format!("exit:{code}"),
                Failure::Mismatch => "mismatch".to_owned(),
                Failure::Timeout => "timeout".to_owned(),
            };
            ("failed", Some(reason))
        }
    }
}

/// The ledger spellings of an IR record's block name.
///
/// The ledger's staleness and acknowledgment layer names authored regions
/// `authored-1`, `authored-2`, …, while the IR names its first authored span
/// `authored` and the rest `authored-2`, …; the first block's two names are
/// bridged here. Consumers translating between the vocabularies go through
/// these helpers — [`ir_block_name`] collapses ledger → IR, this one expands
/// IR → every ledger spelling — one seam, kept in one place, until the
/// naming convention is unified.
pub fn ledger_block_names(ir_name: &str) -> Vec<String> {
    if ir_name == "authored" {
        vec!["authored".to_owned(), "authored-1".to_owned()]
    } else {
        vec![ir_name.to_owned()]
    }
}

/// The IR name of an authored block the staleness layer reports.
///
/// The reverse of [`ledger_block_names`]: the gate reports IR record ids,
/// so the ledger's first-block ordinal collapses to the IR spelling.
fn ir_block_name(block: &str) -> String {
    if block == "authored-1" {
        "authored".to_owned()
    } else {
        block.to_owned()
    }
}

/// Whether the record's block regenerated with bytes different from its
/// state at the change-set base.
fn regenerates_differently(
    record: &Record,
    sources: &[(String, String)],
    base_docs: &BTreeMap<String, String>,
) -> bool {
    let Some((path, name)) = record.id.rsplit_once('#') else {
        return false;
    };
    let Some((_, current)) = sources.iter().find(|(source, _)| source == path) else {
        return false;
    };
    let Some(base) = base_docs.get(path) else {
        return false;
    };
    match &record.kind {
        RecordKind::Doctest { .. } => {
            let body = |document: &str| {
                doctest::extract(document).ok().and_then(|contracts| {
                    contracts
                        .into_iter()
                        .find(|contract| contract.label == name)
                        .map(|contract| contract.body)
                })
            };
            match (body(current), body(base)) {
                (Some(current), Some(base)) => current != base,
                _ => false,
            }
        }
        _ => {
            let body = |document: &str| {
                named_blocks(document)
                    .into_iter()
                    .find(|(block, _)| block == name)
                    .map(|(_, body)| body)
            };
            match (body(current), body(base)) {
                (Some(current), Some(base)) => current != base,
                _ => false,
            }
        }
    }
}
