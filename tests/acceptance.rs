//! MVP-1 acceptance: the three-question dossier on kaya's own record.
//!
//! The first test copies kaya's committed wai artifact tree, model seed, and
//! checked-in dossier into a temporary workspace, commits them, imports the
//! seed through `kaya import model`, then runs the built binary the way a user
//! would: `kaya import wai`, `kaya import git --limit 5`, `kaya build
//! --allow-uncommitted`, a second build, and `kaya status`. The second test
//! commits the ledger before building and omits the flag.
//!
//! It gates two things: the three acceptance questions are answerable from the
//! generated dossier with provenance, and compilation appends/rewrites only
//! its own generated blocks — the authored answers and a foreign tool's
//! managed block survive byte for byte.

use kaya::impact::dependency_digest;
use kaya::projection::ir::{Record, RecordKind};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use assert_cmd::Command as AssertCommand;
use serde_json::{Value, json};
use tempfile::TempDir;

const EXPERIMENT: &str = "kaya/Experiment/add-core-research-record";
const QUESTION: &str = "kaya/Question/mvp-1-acceptance";
const HYPOTHESIS: &str = "kaya/Hypothesis/H-001";
const LEDGER: &str = ".kaya/ledger.jsonl";
const SEED: &str = ".kaya/seeds/add-core-research-record.jsonl";
const INDEX: &str = "docs/research/index.md";
const REPORT: &str = "docs/research/experiments/kaya-Experiment-add-core-research-record.md";
const CHECKED_IN_REPORT: &str =
    "docs/research/experiments/kaya-Experiment-add-core-research-record.md";
const SPECIFICATION_NAME: &str =
    "exploration.experiment-specification:kaya/Experiment/add-core-research-record";
const RECORD_NAME: &str = "exploration.experiment-record:kaya/Experiment/add-core-research-record";
const SEED_EVENTS: u64 = 13;
const SOURCE_COMMIT: &str = "43b606f";

const AUTHORED_BEGIN: &str = "<!-- kaya:authored -->";
const AUTHORED_END: &str = "<!-- kaya:end authored -->";
const FOREIGN_BEGIN: &str = "<!-- other-tool:begin -->";
const FOREIGN_END: &str = "<!-- other-tool:end -->";
const INDEX_BLOCK_NAME: &str = "orientation.research-map";
const INDEX_BLOCK_BEGIN: &str = "<!-- kaya:begin generated:orientation.research-map -->";
const INDEX_BLOCK_END: &str = "<!-- kaya:end generated:orientation.research-map -->";
const SPECIFICATION_BLOCK_BEGIN: &str =
    "<!-- kaya:begin generated:exploration.experiment-specification -->";
const SPECIFICATION_BLOCK_END: &str =
    "<!-- kaya:end generated:exploration.experiment-specification -->";
const RECORD_BLOCK_BEGIN: &str = "<!-- kaya:begin generated:exploration.experiment-record -->";
const RECORD_BLOCK_END: &str = "<!-- kaya:end generated:exploration.experiment-record -->";
const REPRODUCTION_BLOCK_NAME: &str = "reuse-handoff.reproduction-package";
const REPRODUCTION_BLOCK_BEGIN: &str =
    "<!-- kaya:begin generated:reuse-handoff.reproduction-package -->";
const REPRODUCTION_BLOCK_END: &str =
    "<!-- kaya:end generated:reuse-handoff.reproduction-package -->";
const REPRODUCTION_FILE: &str = "docs/research/reproduction-package.md";
const HANDOFF_FILE: &str = "docs/research/handoff.md";
const UNDERSTANDING_FILE: &str = "docs/research/understanding.md";

/// The reproduction package lists exactly these committed Source rows, sorted
/// by node id, one `path@commit` reference each.
const REPRODUCTION_ROWS: [&str; 6] = [
    "- kaya/Source/docs-src-index.md@43b606f — docs/src/index.md@43b606f",
    "- kaya/Source/openspec-changes-add-core-research-record-proposal.md@43b606f — openspec/changes/add-core-research-record/proposal.md@43b606f",
    "- kaya/Source/openspec-changes-add-core-research-record-specs-kaya-adapters-spec.md@43b606f — openspec/changes/add-core-research-record/specs/kaya-adapters/spec.md@43b606f",
    "- kaya/Source/openspec-changes-add-core-research-record-specs-kaya-event-layer.md@43b606f — openspec/changes/add-core-research-record/specs/kaya-event/layer.md@43b606f",
    "- kaya/Source/openspec-changes-add-core-research-record-specs-kaya-promotion-spec.md@43b606f — openspec/changes/add-core-research-record/specs/kaya-promotion/spec.md@43b606f",
    "- kaya/Source/wai-projects-kaya-designs-2026-11-16-post-bootstrap-alignment-consumers-mvp-1-accep.md@43b606f — .wai/projects/kaya/designs/2026-11-16-post-bootstrap-alignment-consumers-mvp-1-accep.md@43b606f",
];

const QUESTION_HEADINGS: [&str; 3] = [
    "### Q1: Why are events not genesis envelopes?",
    "### Q2: What evidence supports \"adapters translate, never interpret\"?",
    "### Q3: What is deliberately unspecified (MVP-2+ seams)?",
];

/// Sources each answer names, so the questions are answerable with provenance.
const PROVENANCE_CITATIONS: [&str; 6] = [
    "openspec/changes/add-core-research-record/proposal.md",
    "openspec/changes/add-core-research-record/specs/kaya-event/layer.md",
    "openspec/changes/add-core-research-record/specs/kaya-adapters/spec.md",
    ".wai/projects/kaya/designs/2026-11-16-post-bootstrap-alignment-consumers-mvp-1-accep.md",
    "docs/src/index.md",
    "f39a1fd",
];

/// Repo files pinned as Source entities in the committed model seed.
const SOURCE_PATHS: [&str; 6] = [
    "openspec/changes/add-core-research-record/proposal.md",
    "openspec/changes/add-core-research-record/specs/kaya-event/layer.md",
    "openspec/changes/add-core-research-record/specs/kaya-adapters/spec.md",
    ".wai/projects/kaya/designs/2026-11-16-post-bootstrap-alignment-consumers-mvp-1-accep.md",
    "docs/src/index.md",
    "openspec/changes/add-core-research-record/specs/kaya-promotion/spec.md",
];

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Run the built binary in `workspace`, requiring exit 0 and an ok envelope.
fn run_kaya(workspace: &Path, args: &[&str]) -> Value {
    let mut command = AssertCommand::cargo_bin("kaya").expect("kaya binary is built");
    command.current_dir(workspace).args(args);
    let output = command.output().expect("kaya runs");
    assert!(
        output.status.success(),
        "kaya {args:?} exited with {:?}: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap_or_else(|err| {
        panic!(
            "kaya {args:?} did not print one JSON envelope ({err}): {}",
            String::from_utf8_lossy(&output.stdout)
        )
    });
    assert_eq!(
        envelope["ok"],
        json!(true),
        "kaya {args:?} returned an error envelope: {envelope}"
    );
    envelope
}

fn names(envelope: &Value, field: &str) -> Vec<String> {
    envelope["data"][field]
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn lists(names: &[String], expected: &str) -> bool {
    names.iter().any(|name| name == expected)
}

/// Relationships the committed model seed declares, as the map renders them.
///
/// Edge signature: `-[<relationship>]-> <to>`, unique per declared edge.
/// Endpoint ids stay asserted separately (EXPERIMENT/QUESTION/HYPOTHESIS);
/// the match starts at the marker because endpoint labels are appended after
/// each id, so a pin beginning at `- <from>` would split mid-row.
const COMMITTED_EDGES: [&str; 3] = [
    "-[predicts]-> kaya/Experiment/add-core-research-record",
    "-[realizes]-> kaya/Hypothesis/H-001",
    "-[investigates]-> kaya/Experiment/add-core-research-record",
];

/// Whether `document` carries the map's trailing `_N nodes from M events._` line.
///
/// The event count depends on what the adapters capture in the temp workspace,
/// so acceptance asserts the shape, not the numbers.
fn has_node_count_line(document: &str) -> bool {
    document.lines().any(|line| {
        line.starts_with('_') && line.contains(" nodes from ") && line.ends_with(" events._")
    })
}

/// The exact bytes from `begin` through `end` in `document`.
fn between<'a>(document: &'a str, begin: &str, end: &str) -> &'a str {
    let start = document
        .find(begin)
        .unwrap_or_else(|| panic!("document lacks begin marker {begin}"));
    let end_offset = document[start..]
        .find(end)
        .unwrap_or_else(|| panic!("document lacks end marker {end}"));
    &document[start..start + end_offset + end.len()]
}

fn copy_markdown_tree(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).expect("create artifact directory");
    for entry in fs::read_dir(source).expect("read artifact directory") {
        let entry = entry.expect("artifact directory entry");
        let path = entry.path();
        let target = destination.join(entry.file_name());
        if path.is_dir() {
            copy_markdown_tree(&path, &target);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("md") {
            fs::copy(&path, &target).expect("copy markdown artifact");
        }
    }
}

fn copy_checked_in(source: &Path, destination: &Path) {
    let parent = destination.parent().expect("destination has a parent");
    fs::create_dir_all(parent).expect("create dossier directory");
    // The checked-in dossier already carries generated blocks from earlier
    // dogfood builds. The acceptance run proves the first-build path, so the
    // copy starts from the authored/foreign bytes only.
    let text = fs::read_to_string(source).expect("read checked-in dossier document");
    fs::write(destination, strip_generated_blocks(&text))
        .expect("write pre-build dossier document");
}

/// Copy the committed model seed into the workspace so the binary imports it.
fn copy_seed(workspace: &Path, manifest: &Path) -> PathBuf {
    let seed = workspace.join(SEED);
    let parent = seed.parent().expect("seed path has a parent");
    fs::create_dir_all(parent).expect("create seed directory");
    fs::copy(manifest.join(SEED), &seed).expect("copy committed model seed");
    seed
}

/// Remove kaya generated blocks, keeping authored and foreign bytes intact.
///
/// Every generated block is named `<family>.<kind>`, so the dossier's
/// `exploration.experiment-specification` and `exploration.experiment-record`
/// blocks both fall under the same `generated:` marker rule.
fn strip_generated_blocks(document: &str) -> String {
    let mut kept = String::new();
    let mut inside = false;
    for line in document.split_inclusive('\n') {
        let trimmed = line.trim();
        if trimmed.starts_with("<!-- kaya:begin generated:") {
            inside = true;
            continue;
        }
        if inside {
            if trimmed.starts_with("<!-- kaya:end generated:") {
                inside = false;
            }
            continue;
        }
        kept.push_str(line);
    }
    kept
}

/// One deterministic commit: fixed identity, fixed dates, no user config.
fn git(workspace: &Path, args: &[&str]) {
    let output = Command::new("git")
        .arg("-C")
        .arg(workspace)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .env("GIT_AUTHOR_NAME", "kaya acceptance")
        .env("GIT_AUTHOR_EMAIL", "acceptance@example.invalid")
        .env("GIT_COMMITTER_NAME", "kaya acceptance")
        .env("GIT_COMMITTER_EMAIL", "acceptance@example.invalid")
        .env("GIT_AUTHOR_DATE", "2026-11-16T00:00:00+00:00")
        .env("GIT_COMMITTER_DATE", "2026-11-16T00:00:00+00:00")
        .args(args)
        .output()
        .expect("git runs");
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn mvp1_three_question_dossier_compiles_from_the_repository_record() {
    let workspace = TempDir::new().expect("temp workspace");
    let root = workspace.path();
    let manifest = manifest_dir();

    // The report the build must compile names its experiment by construction:
    // `docs/research/experiments/<namespace-qualified id with / as ->.md`.
    let expected_report = format!(
        "docs/research/experiments/{}.md",
        EXPERIMENT.replace('/', "-")
    );
    assert_eq!(REPORT, expected_report.as_str());

    let artifacts = root.join(".wai/projects/kaya");
    for kind in ["designs", "research"] {
        copy_markdown_tree(
            &manifest.join(".wai/projects/kaya").join(kind),
            &artifacts.join(kind),
        );
    }

    let index_path = root.join(INDEX);
    let report_path = root.join(REPORT);
    copy_checked_in(&manifest.join(INDEX), &index_path);
    copy_checked_in(&manifest.join(CHECKED_IN_REPORT), &report_path);
    copy_seed(root, &manifest);

    git(root, &["init", "--quiet"]);
    git(root, &["add", "--all"]);
    git(
        root,
        &[
            "commit",
            "--quiet",
            "--message",
            "seed the kaya acceptance workspace",
        ],
    );

    let init = run_kaya(root, &["init"]);
    assert_eq!(init["envelope_kind"], json!("init"));

    let model = run_kaya(root, &["import", "model", "--file", SEED]);
    assert_eq!(model["data"]["source"], json!("model"));
    assert_eq!(
        model["data"]["events"],
        json!(SEED_EVENTS),
        "the committed seed carries every seeded model event: {model}"
    );
    assert_eq!(model["data"]["appended"], json!(SEED_EVENTS));
    assert_eq!(model["data"]["duplicates"], json!(0));

    let wai = run_kaya(root, &["import", "wai"]);
    assert_eq!(wai["data"]["source"], json!("wai"));
    assert!(
        wai["data"]["events"].as_u64().unwrap_or(0) > 0,
        "wai import captured at least one artifact: {wai}"
    );

    let imported_git = run_kaya(root, &["import", "git", "--limit", "5"]);
    assert_eq!(imported_git["data"]["source"], json!("git"));
    assert!(
        imported_git["data"]["events"].as_u64().unwrap_or(0) > 0,
        "git import captured the seeded commit: {imported_git}"
    );

    // Byte baselines: what the checked-in documents contain before kaya runs.
    let index_before = fs::read(&index_path).expect("copied index exists");
    let report_before = fs::read(&report_path).expect("copied report exists");
    let index_source = String::from_utf8(index_before.clone()).expect("index is UTF-8");
    let report_source = String::from_utf8(report_before.clone()).expect("report is UTF-8");
    let authored_answers = between(&report_source, AUTHORED_BEGIN, AUTHORED_END);
    let foreign_block = between(&index_source, FOREIGN_BEGIN, FOREIGN_END);
    assert!(
        !index_source.contains(INDEX_BLOCK_BEGIN),
        "the checked-in index carries no generated block; the first build appends it"
    );
    assert!(
        !report_source.contains(SPECIFICATION_BLOCK_BEGIN)
            && !report_source.contains(SPECIFICATION_BLOCK_END)
            && !report_source.contains(RECORD_BLOCK_BEGIN)
            && !report_source.contains(RECORD_BLOCK_END),
        "the checked-in report carries no generated block; the first build appends it"
    );

    let build = run_kaya(root, &["build", "--allow-uncommitted"]);
    assert_eq!(build["envelope_kind"], json!("build"));
    let recomputed = names(&build, "recomputed");
    assert!(
        lists(&recomputed, INDEX_BLOCK_NAME),
        "first build compiles the research map: {build}"
    );
    assert!(
        lists(&recomputed, SPECIFICATION_NAME),
        "first build compiles the experiment specification: {build}"
    );
    assert!(
        !lists(&recomputed, RECORD_NAME),
        "the dossier has no runs, so no record block is written: {build}"
    );
    assert_eq!(
        build["data"]["omitted"],
        json!([
            {
                "family": "exploration",
                "kind": "experiment-record",
                "reason": "no-records",
            },
            {
                "family": "reuse-handoff",
                "kind": "handoff",
                "reason": "no-records",
            },
            {
                "family": "reuse-handoff",
                "kind": "knowledge-brief",
                "reason": "no-records",
            },
            {
                "family": "reuse-handoff",
                "kind": "technical-reference",
                "reason": "no-records",
            },
            {
                "family": "understanding",
                "kind": "decision-record",
                "reason": "no-records",
            },
            {
                "family": "understanding",
                "kind": "findings-and-analysis",
                "reason": "no-records",
            },
        ]),
        "every empty kind is reported once with family and reason, sorted by family then kind: {build}"
    );

    // The index: generated block appended after the authored and foreign bytes.
    let index_after_bytes = fs::read(&index_path).expect("build wrote the index");
    assert!(
        index_after_bytes.starts_with(&index_before),
        "build appends to the index without rewriting authored or foreign bytes"
    );
    let index_after = String::from_utf8(index_after_bytes.clone()).expect("index is UTF-8");
    assert!(index_after.contains(INDEX_BLOCK_BEGIN), "{index_after}");
    assert!(index_after.contains(INDEX_BLOCK_END), "{index_after}");
    let generated_index = between(&index_after, INDEX_BLOCK_BEGIN, INDEX_BLOCK_END);
    assert!(
        generated_index.contains("### Relationships\n"),
        "the generated map renders the relationship section: {generated_index}"
    );
    for edge in COMMITTED_EDGES {
        assert_eq!(
            generated_index.matches(edge).count(),
            1,
            "every declared edge renders once with both endpoints and its type ({edge}): {generated_index}"
        );
    }
    for endpoint in [EXPERIMENT, QUESTION, HYPOTHESIS] {
        assert!(
            generated_index.contains(endpoint),
            "the generated map names endpoint {endpoint}: {generated_index}"
        );
    }
    assert!(
        generated_index.contains("### Artifacts\n"),
        "the generated map keeps the Artifacts section: {generated_index}"
    );
    assert_eq!(
        generated_index
            .matches("kaya/Source/docs-src-index.md@43b606f")
            .count(),
        1,
        "an edgeless source renders once, in the Sources detail section: {generated_index}"
    );
    assert!(
        generated_index.contains("### Sources\n"),
        "the generated index renders Sources: {generated_index}"
    );
    for path in SOURCE_PATHS {
        let pinned = format!("{path}@{SOURCE_COMMIT}");
        assert!(
            generated_index.contains(&pinned),
            "the generated index pins source {pinned}: {generated_index}"
        );
    }
    assert!(
        has_node_count_line(generated_index),
        "the generated map ends with its `_N nodes from M events._` summary: {generated_index}"
    );
    assert!(
        index_after.contains(foreign_block),
        "the foreign managed block survives the rebuild byte for byte"
    );

    // The wai adapter walks direct children of each kind directory, and the
    // generated index must name every artifact it captured.
    let mut wai_artifacts: Vec<PathBuf> = Vec::new();
    for kind in ["designs", "research"] {
        let mut captured: Vec<PathBuf> = fs::read_dir(artifacts.join(kind))
            .expect("copied artifact directory")
            .map(|entry| entry.expect("artifact entry").path())
            .filter(|path| path.is_file())
            .collect();
        captured.sort();
        wai_artifacts.extend(captured);
    }
    assert!(
        !wai_artifacts.is_empty(),
        "the wai artifact tree was copied"
    );
    for artifact in &wai_artifacts {
        let relative = artifact
            .strip_prefix(root)
            .expect("artifact path is inside the workspace");
        let rendered = relative.to_string_lossy().replace('\\', "/");
        assert!(
            generated_index.contains(&rendered),
            "the generated index lists wai artifact {rendered}"
        );
    }

    // The reproduction package: one reference row per committed Source and no
    // Commands section. No handoff artifact exists, so no handoff document.
    let reproduction_after = fs::read_to_string(root.join(REPRODUCTION_FILE))
        .expect("build wrote the reproduction package");
    assert!(
        reproduction_after.contains(REPRODUCTION_BLOCK_BEGIN)
            && reproduction_after.contains(REPRODUCTION_BLOCK_END),
        "the reproduction package carries its generated block: {reproduction_after}"
    );
    let reproduction_block = between(
        &reproduction_after,
        REPRODUCTION_BLOCK_BEGIN,
        REPRODUCTION_BLOCK_END,
    );
    let reproduction_body = reproduction_block
        .strip_prefix(REPRODUCTION_BLOCK_BEGIN)
        .and_then(|rest| rest.strip_suffix(REPRODUCTION_BLOCK_END))
        .expect("the reproduction block is delimited by its markers");
    let mut expected_rows = vec!["### Sources"];
    expected_rows.extend(REPRODUCTION_ROWS);
    assert_eq!(
        reproduction_body.trim(),
        expected_rows.join("\n"),
        "the package lists exactly the six committed Source references"
    );
    assert!(
        !reproduction_after.contains("### Commands"),
        "the dossier records no run command: {reproduction_after}"
    );
    assert!(
        !root.join(HANDOFF_FILE).exists(),
        "no handoff artifact exists, so no handoff document is written"
    );
    assert!(
        !root.join(UNDERSTANDING_FILE).exists(),
        "the ledger holds no Finding or Decision records, so both understanding kinds are empty and no understanding document is written"
    );

    // The report: generated block plus the preserved authored dossier.
    let report_after_bytes = fs::read(&report_path).expect("build wrote the report");
    assert!(
        report_after_bytes.starts_with(&report_before),
        "build appends to the report without rewriting the authored answers"
    );
    let report_after = String::from_utf8(report_after_bytes.clone()).expect("report is UTF-8");
    assert!(
        report_after.contains(SPECIFICATION_BLOCK_BEGIN),
        "{report_after}"
    );
    assert!(
        report_after.contains(SPECIFICATION_BLOCK_END),
        "{report_after}"
    );
    assert!(
        !report_after.contains(RECORD_BLOCK_BEGIN) && !report_after.contains(RECORD_BLOCK_END),
        "the experiment has no runs, so the record block is omitted: {report_after}"
    );
    let generated = between(
        &report_after,
        SPECIFICATION_BLOCK_BEGIN,
        SPECIFICATION_BLOCK_END,
    );
    for heading in ["## Question", "## Hypothesis"] {
        assert!(
            generated.contains(heading),
            "the generated specification renders {heading}: {generated}"
        );
    }
    for absent in [
        "## Runs",
        "## Observations",
        "## Findings",
        "## Interpretations",
        "- _(none)_",
    ] {
        assert!(
            !generated.contains(absent),
            "an empty kind leaves no section or placeholder row: {absent}"
        );
    }
    assert!(
        generated.contains(QUESTION),
        "the generated specification names the investigated question: {generated}"
    );
    assert!(
        generated.contains(HYPOTHESIS),
        "the generated specification names the predicted hypothesis: {generated}"
    );
    assert!(
        report_after.contains(EXPERIMENT),
        "the report names the experiment it compiles"
    );
    assert!(
        report_after.contains(authored_answers),
        "the authored answers survive the rebuild byte for byte"
    );
    for heading in QUESTION_HEADINGS {
        assert!(
            report_after.contains(heading),
            "the dossier answers {heading}"
        );
    }
    for citation in PROVENANCE_CITATIONS {
        assert!(
            report_after.contains(citation),
            "the dossier names provenance source {citation}"
        );
    }

    // A second build must be a no-op on every projection.
    let rebuild = run_kaya(root, &["build", "--allow-uncommitted"]);
    assert!(
        names(&rebuild, "recomputed").is_empty(),
        "second build recomputes nothing: {rebuild}"
    );
    let unchanged = names(&rebuild, "unchanged");
    assert!(
        lists(&unchanged, INDEX_BLOCK_NAME),
        "second build reports the research map unchanged: {rebuild}"
    );
    assert!(
        lists(&unchanged, SPECIFICATION_NAME),
        "second build reports the specification unchanged: {rebuild}"
    );
    assert!(
        lists(&unchanged, REPRODUCTION_BLOCK_NAME),
        "second build reports the reproduction package unchanged: {rebuild}"
    );
    assert!(
        !lists(&unchanged, RECORD_NAME),
        "an omitted kind is never reported unchanged: {rebuild}"
    );
    assert_eq!(
        fs::read(&index_path).expect("index still exists"),
        index_after_bytes,
        "unchanged index keeps its bytes"
    );
    assert_eq!(
        fs::read(&report_path).expect("report still exists"),
        report_after_bytes,
        "unchanged report keeps its bytes"
    );

    // Status surfaces the pending promotion budget without prompting.
    let status = run_kaya(root, &["status"]);
    let pending = status["data"]["pending_proposals"]
        .as_u64()
        .unwrap_or_else(|| panic!("status reports pending_proposals: {status}"));
    assert_eq!(
        pending, 0,
        "no promotion events were seeded, so nothing is pending"
    );
}

#[test]
fn committed_ledger_builds_without_the_uncommitted_flag() {
    let workspace = TempDir::new().expect("temp workspace");
    let root = workspace.path();
    let manifest = manifest_dir();

    git(root, &["init", "--quiet"]);
    let init = run_kaya(root, &["init"]);
    assert_eq!(init["envelope_kind"], json!("init"));
    copy_seed(root, &manifest);

    let model = run_kaya(root, &["import", "model", "--file", SEED]);
    assert_eq!(model["data"]["appended"], json!(SEED_EVENTS));
    assert!(
        root.join(LEDGER).is_file(),
        "the ledger records the imported seed"
    );

    git(root, &["add", LEDGER, SEED]);
    git(
        root,
        &[
            "commit",
            "--quiet",
            "--message",
            "record the model seed before building",
        ],
    );

    // No flag: the ledger is committed, so git can verify the built bytes.
    let build = run_kaya(root, &["build"]);
    assert_eq!(build["envelope_kind"], json!("build"));
    let recomputed = names(&build, "recomputed");
    assert!(
        lists(&recomputed, INDEX_BLOCK_NAME),
        "committed build compiles the research map: {build}"
    );
    assert!(
        lists(&recomputed, SPECIFICATION_NAME),
        "committed build compiles the experiment specification: {build}"
    );

    let report = fs::read_to_string(root.join(REPORT)).expect("build wrote the report");
    let generated = between(&report, SPECIFICATION_BLOCK_BEGIN, SPECIFICATION_BLOCK_END);
    assert!(
        !report.contains(RECORD_BLOCK_BEGIN) && !report.contains(RECORD_BLOCK_END),
        "the seeded experiment has no runs, so the record block is omitted: {report}"
    );
    assert!(
        generated.contains(QUESTION) && generated.contains(HYPOTHESIS),
        "the generated report cites its question and hypothesis: {generated}"
    );
    assert!(
        !root.join(UNDERSTANDING_FILE).exists(),
        "the committed ledger holds no Finding or Decision records, so both understanding kinds are empty and no understanding document is written"
    );

    let index = fs::read_to_string(root.join(INDEX)).expect("build wrote the index");
    assert!(index.contains(INDEX_BLOCK_BEGIN), "{index}");
    assert!(index.contains(INDEX_BLOCK_END), "{index}");
    let generated_index = between(&index, INDEX_BLOCK_BEGIN, INDEX_BLOCK_END);
    assert!(
        generated_index.contains("### Relationships\n"),
        "the committed ledger's edges render: {generated_index}"
    );
    for edge in COMMITTED_EDGES {
        assert_eq!(
            generated_index.matches(edge).count(),
            1,
            "every declared edge renders once with both endpoints and its type ({edge}): {generated_index}"
        );
    }
    assert_eq!(
        generated_index
            .matches("kaya/Source/docs-src-index.md@43b606f")
            .count(),
        1,
        "an edgeless source renders once, in the Sources detail section: {generated_index}"
    );
    for path in SOURCE_PATHS {
        let pinned = format!("{path}@{SOURCE_COMMIT}");
        assert!(
            generated_index.contains(&pinned),
            "the generated index pins source {pinned}: {generated_index}"
        );
    }
    assert!(
        has_node_count_line(generated_index),
        "the generated map ends with its `_N nodes from M events._` summary: {generated_index}"
    );
}

// ---------------------------------------------------------------------------
// kaya-l46: dispositions bind evidence digests; `kaya ir --walk` explains.
// ---------------------------------------------------------------------------

/// The authored region the l46 fixture seeds into the research index; a
/// record with a citation, so its dependency digest is content-bearing.
const L46_AUTHORED_RECORD: &str = "docs/research/index.md#authored";

/// The generated research-map block the build appends to the index.
const L46_GENERATED_RECORD: &str = "docs/research/index.md#orientation.research-map";

/// The map block's rendered body opens with the Relationships heading.
const L46_BLOCK_BODY_TEXT: &str = "### Relationships\\n";

/// A disposable workspace seeded like `committed_ledger_builds_without_the_
/// uncommitted_flag`, plus an authored region citing a seed event so the
/// index carries one content-bearing record after the build.
struct L46Fixture {
    workspace: TempDir,
    /// The seed event id the authored region cites.
    cited: String,
    /// A different seed event id, for mutating the citation later.
    second: String,
}

impl L46Fixture {
    fn new() -> Self {
        let workspace = TempDir::new().expect("temp workspace");
        let root = workspace.path();
        let manifest = manifest_dir();

        git(root, &["init", "--quiet"]);
        let init = run_kaya(root, &["init"]);
        assert_eq!(init["envelope_kind"], json!("init"));
        copy_seed(root, &manifest);
        run_kaya(root, &["import", "model", "--file", SEED]);

        let seed_events = ledger_events(root);
        let cited = seed_events[0]["id"]
            .as_str()
            .expect("a seed event id")
            .to_owned();
        let second = seed_events[1]["id"]
            .as_str()
            .expect("a seed event id")
            .to_owned();
        let index = root.join(INDEX);
        fs::create_dir_all(index.parent().expect("the index has a parent"))
            .expect("create research directory");
        fs::write(
            &index,
            format!(
                "# Research index\n\n\
                 <!-- kaya:authored -->\n\
                 Authored interpretation of the compiled record.\n\
                 <!-- kaya:cites {cited} -->\n\
                 <!-- kaya:end authored -->\n"
            ),
        )
        .expect("seed the authored research index");

        git(root, &["add", "--all"]);
        git(
            root,
            &["commit", "--quiet", "--message", "seed the workspace"],
        );
        run_kaya(root, &["build"]);

        Self {
            workspace,
            cited,
            second,
        }
    }

    fn root(&self) -> PathBuf {
        self.workspace.path().to_path_buf()
    }
}

/// Every ledger event, one JSON value per line.
fn ledger_events(root: &Path) -> Vec<Value> {
    let text = fs::read_to_string(root.join(LEDGER)).expect("the ledger is readable");
    text.lines()
        .map(|line| {
            serde_json::from_str(line)
                .unwrap_or_else(|error| panic!("a ledger line is one JSON event ({error}): {line}"))
        })
        .collect()
}

/// The IR record behind `ir_json`, rebuilt for `dependency_digest`.
///
/// The digest covers only the record's id, cited event ids, and artifact
/// references, so the untouched kind placeholder never changes the result.
fn l46_record(ir_json: &Value, id: &str) -> Record {
    let entry = &ir_json["data"]["records"][id];
    assert!(
        entry.is_object(),
        "the IR materializes record {id}: {ir_json}"
    );
    let strings = |field: &str| -> Vec<String> {
        entry[field]
            .as_array()
            .map(|items| {
                items
                    .iter()
                    .map(|item| item.as_str().expect("a string id").to_owned())
                    .collect()
            })
            .unwrap_or_default()
    };
    Record {
        id: id.to_owned(),
        kind: RecordKind::AuthoredBlock,
        content_hash: String::new(),
        provenance: strings("provenance"),
        cites: strings("cites"),
        artifacts: strings("artifacts"),
        contains: strings("contains"),
    }
}

/// Every object in `value` for which `keep` holds, searched recursively.
fn objects_where(value: &Value, keep: &dyn Fn(&Value) -> bool) -> Vec<Value> {
    let mut found = Vec::new();
    fn walk(value: &Value, keep: &dyn Fn(&Value) -> bool, found: &mut Vec<Value>) {
        if keep(value) {
            found.push(value.clone());
        }
        match value {
            Value::Array(items) => {
                for item in items {
                    walk(item, keep, found);
                }
            }
            Value::Object(map) => {
                for item in map.values() {
                    walk(item, keep, found);
                }
            }
            _ => {}
        }
    }
    walk(value, keep, &mut found);
    found
}

/// Record a human `reviewed` disposition for the fixture's authored record.
fn l46_review_authored(root: &Path) -> Value {
    run_kaya(
        root,
        &[
            "disposition",
            L46_AUTHORED_RECORD,
            "reviewed",
            "--actor",
            "alice",
        ],
    )
}

/// Run `kaya ir --walk`, requiring exit 0 and an ok envelope; a refusal
/// panics naming the refusal message.
#[track_caller]
fn run_kaya_walk(root: &Path) -> Value {
    let output = AssertCommand::cargo_bin("kaya")
        .expect("kaya binary is built")
        .current_dir(root)
        .args(["ir", "--walk"])
        .output()
        .expect("kaya runs");
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "kaya ir --walk did not print one JSON envelope ({error}): {}",
            String::from_utf8_lossy(&output.stdout)
        )
    });
    assert!(
        output.status.success() && envelope["ok"] == json!(true),
        "kaya ir --walk must run: {}{}",
        envelope["error"]["message"].as_str().unwrap_or(""),
        String::from_utf8_lossy(&output.stderr)
    );
    envelope
}

/// A `reviewed` disposition binds the record's evidence digest, and a
/// `waived` disposition keeps binding rationale and digest as before.
#[test]
fn reviewed_disposition_binds_evidence_digest() {
    let fixture = L46Fixture::new();
    let root = fixture.root();

    let ir = run_kaya(root.as_path(), &["ir"]);
    let expected = dependency_digest(&l46_record(&ir, L46_AUTHORED_RECORD));

    let review = l46_review_authored(&root);
    assert_eq!(
        review["data"]["event"]["type"],
        json!("verification.record.reviewed"),
        "the review is a verification.record.reviewed event: {review}"
    );
    assert_eq!(
        review["data"]["event"]["data"]["digest"],
        json!(expected),
        "a reviewed disposition binds the dependency digest of its record: {review}"
    );
    let ledger = ledger_events(&root);
    let recorded = ledger
        .iter()
        .find(|event| event["type"] == json!("verification.record.reviewed"))
        .expect("the ledger records the review");
    assert_eq!(
        recorded["data"],
        json!({
            "record": L46_AUTHORED_RECORD,
            "disposition": "reviewed",
            "digest": expected,
        }),
        "the appended review event carries record, disposition, and digest"
    );

    // Unchanged guard: waivers still bind rationale and digest.
    let waiver = run_kaya(
        root.as_path(),
        &[
            "disposition",
            L46_GENERATED_RECORD,
            "waived",
            "--rationale",
            "accepted risk",
            "--actor",
            "bob",
        ],
    );
    let waived_digest = dependency_digest(&l46_record(&ir, L46_GENERATED_RECORD));
    assert_eq!(
        waiver["data"]["event"]["data"],
        json!({
            "record": L46_GENERATED_RECORD,
            "disposition": "waived",
            "rationale": "accepted risk",
            "digest": waived_digest,
        }),
        "the waiver still binds its rationale and dependency digest: {waiver}"
    );
}

/// `kaya ir --walk` resolves provenance and the human trail; staleness
/// shows the moment the record's backing input changes without a re-review.
#[test]
fn ir_walk_resolves_provenance_and_trail() {
    let fixture = L46Fixture::new();
    let root = fixture.root();

    l46_review_authored(&root);
    let ir = run_kaya(root.as_path(), &["ir"]);
    let reviewed_digest = dependency_digest(&l46_record(&ir, L46_AUTHORED_RECORD));
    let provenance_id = ir["data"]["records"][L46_GENERATED_RECORD]["provenance"][0]
        .as_str()
        .expect("the research map cites at least one backing event id")
        .to_owned();
    let backing = ledger_events(&root)
        .into_iter()
        .find(|event| event["subject"]["id"] == json!(provenance_id))
        .expect("the provenance node id names a ledger subject");
    let backing_type = backing["type"].as_str().expect("an event type");
    let backing_actor = backing["actor"]["name"].as_str().expect("an actor name");

    let walk = run_kaya_walk(&root);
    let rendered = serde_json::to_string(&walk["data"]).expect("the walk serializes");

    // The walk carries the block body a human checks.
    assert!(
        rendered.contains(L46_BLOCK_BODY_TEXT),
        "the walk emits the rendered block body ({L46_BLOCK_BODY_TEXT:?}): {rendered}"
    );
    // The walk resolves each provenance id to its event summary.
    let resolved = objects_where(&walk["data"], &|value: &Value| {
        value["id"] == json!(provenance_id)
    });
    assert!(
        !resolved.is_empty(),
        "the walk resolves provenance id {provenance_id}: {rendered}"
    );
    assert!(
        resolved
            .iter()
            .any(|entry| entry["type"] == json!(backing_type)),
        "the resolved provenance names the backing event type {backing_type}: {resolved:?}"
    );
    assert!(
        rendered.contains(backing_actor),
        "the resolved provenance names the backing actor {backing_actor}: {rendered}"
    );
    // The walk reports the human trail with freshness.
    let fresh_trail = objects_where(&walk["data"], &|value: &Value| {
        value["disposition"] == json!("reviewed")
            && value["actor"] == json!("alice")
            && value["digest"] == json!(reviewed_digest)
            && value["fresh"] == json!(true)
    });
    assert!(
        !fresh_trail.is_empty(),
        "the walk reports the reviewed trail fresh: {rendered}"
    );

    // Change the record's backing input without a re-review: the same trail
    // entry now reports stale.
    let index = root.join(INDEX);
    let before = fs::read_to_string(&index).expect("the built index exists");
    let after = before.replace(&fixture.cited, &fixture.second);
    assert_ne!(after, before, "the fixture citation changes");
    fs::write(&index, after).expect("rewrite the citation");

    let stale_walk = run_kaya_walk(&root);
    let stale_trail = objects_where(&stale_walk["data"], &|value: &Value| {
        value["disposition"] == json!("reviewed")
            && value["actor"] == json!("alice")
            && value["digest"] == json!(reviewed_digest)
            && value["fresh"] == json!(false)
    });
    assert!(
        !stale_trail.is_empty(),
        "the walk reports the reviewed trail stale after the backing input changed: {stale_walk}"
    );
}

/// Guard: plain `kaya ir` keeps its shape and never carries walk-only
/// trail or freshness fields.
#[test]
fn ir_plain_output_unchanged() {
    let fixture = L46Fixture::new();
    let root = fixture.root();

    l46_review_authored(&root);
    let plain = run_kaya(root.as_path(), &["ir"]);
    let rendered = serde_json::to_string(&plain["data"]).expect("the IR serializes");

    assert!(
        plain["data"]["records"].is_object(),
        "plain ir still materializes records: {plain}"
    );
    assert!(
        !rendered.contains("\"fresh\""),
        "plain ir carries no walk-only freshness fields: {rendered}"
    );
    assert!(
        !rendered.contains("\"disposition\""),
        "plain ir carries no walk-only trail fields: {rendered}"
    );
    assert!(
        !rendered.contains("\"trail\""),
        "plain ir carries no trail key: {rendered}"
    );
}

/// An acknowledgement names the document and carries the block in its data,
/// so `kaya ir --walk` still finds it for the record it covers: the trail
/// entry carries no digest and no freshness verdict.
#[test]
fn ir_walk_includes_acknowledged_blocks() {
    let fixture = L46Fixture::new();
    let root = fixture.root();

    let acknowledged = run_kaya(
        root.as_path(),
        &[
            "acknowledge",
            INDEX,
            "--block",
            "authored-1",
            "--events",
            fixture.cited.as_str(),
            "--actor",
            "carol",
        ],
    );
    assert_eq!(
        acknowledged["data"]["appended"],
        json!(true),
        "the acknowledgment appends its event: {acknowledged}"
    );

    let walk = run_kaya_walk(&root);
    let authored = objects_where(&walk["data"], &|value: &Value| {
        value["id"] == json!(L46_AUTHORED_RECORD)
    });
    assert_eq!(
        authored.len(),
        1,
        "the walk lists the authored record once: {walk}"
    );
    assert_eq!(
        authored[0]["trail"],
        json!([{
            "actor": "carol",
            "disposition": "acknowledged",
            "digest": Value::Null,
            "fresh": Value::Null,
        }]),
        "the acknowledged block's trail carries the acknowledgement entry: {walk}"
    );
}
