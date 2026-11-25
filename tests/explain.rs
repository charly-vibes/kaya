//! Integration tests for `kaya explain`: the read-only agent-experience
//! command that names every publication omission and the path that un-omits
//! it. Nothing here writes to the tree beyond `kaya init` on a fresh ledger.

use assert_cmd::Command;
use serde_json::{Value, json};
use tempfile::TempDir;

fn kaya(dir: &TempDir) -> Command {
    let mut command = Command::cargo_bin("kaya").expect("kaya binary is built");
    command.current_dir(dir.path());
    command
}

fn init(dir: &TempDir) {
    let output = kaya(dir).arg("init").output().unwrap();
    assert!(
        output.status.success(),
        "init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn envelope(output: &std::process::Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|err| {
        panic!(
            "expected one JSON envelope on stdout ({err}); stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

#[test]
fn explain_names_kind_feeder_seed_path_and_dependents() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);

    let output = kaya(&dir)
        .arg("explain")
        .arg("technical-reference")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "explain technical-reference failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let env = envelope(&output);
    assert_eq!(env["ok"], json!(true));
    assert_eq!(env["envelope_kind"], json!("explain"));

    let data = &env["data"];
    assert_eq!(
        data["kind"],
        json!("reuse-handoff.technical-reference"),
        "the bare kind name resolves to its canonical block name"
    );
    assert_eq!(
        data["feeder"]["entity"],
        json!("Source"),
        "the block renders from Source nodes"
    );
    assert_eq!(
        data["feeder"]["attribute"],
        json!("spec"),
        "the required attribute is named"
    );
    assert!(
        data["seed_example"].is_string() && !data["seed_example"].as_str().unwrap().is_empty(),
        "an unadaptered kind ships a copy-pasteable import model seed line"
    );
    assert!(
        data["seed_example"]
            .as_str()
            .unwrap()
            .contains("import model"),
        "the seed example names the ingestion command"
    );
    let backs: Vec<&str> = data["backs"]
        .as_array()
        .expect("backs is a list")
        .iter()
        .map(|v| v.as_str().expect("back names are strings"))
        .collect();
    assert!(
        backs.contains(&"how-to") && backs.contains(&"reference"),
        "the admission-table products this kind backs: {backs:?}"
    );
    assert_eq!(
        data["collections"],
        json!(["api-catalogue"]),
        "the collection keyed on this block is named"
    );
    assert_eq!(
        data["status"],
        json!("unpopulated"),
        "a fresh ledger has no Source(spec) nodes, so the kind is unpopulated"
    );
}

#[test]
fn explain_without_argument_lists_omissions_with_reasons() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);

    let output = kaya(&dir).arg("explain").output().unwrap();
    assert!(
        output.status.success(),
        "bare explain failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let env = envelope(&output);
    assert_eq!(env["ok"], json!(true));
    assert_eq!(env["envelope_kind"], json!("explain"));

    let omitted = env["data"]["omitted"]
        .as_array()
        .expect("omitted is a list");
    let names: Vec<&str> = omitted
        .iter()
        .map(|entry| entry["name"].as_str().expect("omission names are strings"))
        .collect();
    assert!(
        names.contains(&"how-to") && !names.contains(&"reference"),
        "how-to stays omitted on the unpopulated technical-reference; reference emits on the \
         always-rendering research-map: {names:?}"
    );
    assert!(
        names.contains(&"api-catalogue"),
        "the collection keyed on an unpopulated block is omitted: {names:?}"
    );
    for entry in omitted {
        assert_eq!(
            entry["reason"],
            json!("no-backing-document"),
            "every omission carries the pinned reason"
        );
        assert!(
            entry["admits"].is_array() && !entry["admits"].as_array().unwrap().is_empty(),
            "every omission names the kinds that would back it"
        );
    }
}

#[test]
fn explain_refuses_unknown_name_with_closed_vocabularies() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);

    let output = kaya(&dir).arg("explain").arg("nonsense").output().unwrap();
    assert!(!output.status.success(), "an unknown name must be refused");
    let env = envelope(&output);
    assert_eq!(env["ok"], json!(false));
    let label = env["error"]["label"].as_str().expect("labeled error");
    assert!(
        !label.is_empty(),
        "the refusal carries a stable error label"
    );
    let message = env["error"]["message"].as_str().expect("error message");
    assert!(
        message.contains("overview") && message.contains("glossary"),
        "the refusal names the closed product and collection vocabularies: {message}"
    );
    assert!(
        message.contains("orientation.project-overview"),
        "the refusal names the closed document-kind vocabulary: {message}"
    );
}

#[test]
fn explain_product_view_agrees_with_the_bare_census() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);

    // One Question populates the exploration brief, and brief.md is in both
    // read lists, so tutorial reads emitted from the named view and the
    // census alike.
    let output = kaya(&dir)
        .args(["question", "does explain match publish", "--actor", "probe"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "capture question failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output = kaya(&dir).args(["explain", "tutorial"]).output().unwrap();
    let env = envelope(&output);
    assert_eq!(env["ok"], json!(true));
    assert_eq!(
        env["data"]["status"],
        json!("emitted"),
        "the populated brief backs tutorial: brief.md is in both read lists"
    );
    assert!(
        env["data"]["note"].is_null(),
        "no structural caveat remains once the reader set reads the brief"
    );

    let output = kaya(&dir).arg("explain").output().unwrap();
    let env = envelope(&output);
    let names: Vec<&str> = env["data"]["omitted"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["name"].as_str().expect("omission names are strings"))
        .collect();
    assert!(
        !names.contains(&"tutorial"),
        "the census agrees with the named view: tutorial is emitted: {names:?}"
    );
}

#[test]
fn brief_backed_tutorial_publishes() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);

    let output = kaya(&dir)
        .args(["question", "read the brief", "--actor", "probe"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "capture question failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output = kaya(&dir)
        .args(["build", "--allow-uncommitted"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "build failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let output = kaya(&dir).arg("publish").output().unwrap();
    assert!(
        output.status.success(),
        "publish failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let page = dir.path().join("docs/public/tutorial.md");
    assert!(
        page.exists(),
        "a populated exploration brief backs tutorial: publish writes the page"
    );
}

#[test]
fn explain_covers_every_document_kind() {
    let dir = tempfile::tempdir().unwrap();
    init(&dir);

    // Vocabulary parity: the contract table must know every kind the
    // publication source map admits, or the census silently narrows.
    assert_eq!(
        kaya::publication::DOCUMENT_KINDS.len(),
        11,
        "the pinned kind vocabulary"
    );
    for kind in kaya::publication::DOCUMENT_KINDS {
        let output = kaya(&dir).arg("explain").arg(kind).output().unwrap();
        let env = envelope(&output);
        assert_eq!(
            env["ok"],
            json!(true),
            "explain must know kind {kind} ({})",
            env["error"]
        );
        assert_eq!(
            env["data"]["kind"],
            json!(kind),
            "every document kind answers by its block name"
        );
    }
}
