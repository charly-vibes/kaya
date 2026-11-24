//! Property: status/build envelope parity (nx4 `status-build-parity`).
//!
//! For a fixed ledger byte sequence `B` in a fixed git state, one `kaya
//! status` run and one `kaya build --out <fresh tmp>` run over the same
//! bytes either both accept — then `status.digest == build.ledger_digest`,
//! `status.bytes == |B|`, and `status.pending_proposals ==
//! build.pending_proposals` — or the ledger lies outside build's acceptance
//! set, in which case `build.ok == false` with a non-empty `error.label`.
//! Neither run may change `B` on disk or write anything outside `--out`.
//!
//! The alphabet is exhaustive and deterministic: 5 atomic ledger mutations
//! × 2 git profiles (committed, untracked) = 10 cells, each a fresh
//! workspace seeded by `kaya init` + `kaya import model --file` with a fixed
//! 3-event ledger. Every cell runs one full observation step (status + build
//! spawns) in one pass, so the sweep stays meaningful under `CI=true`
//! derandomization.
//!
//! Scope: parity is asserted only inside the build-acceptance classifier's
//! domain, where in-domain `status.events == build.events` parity and the
//! build-envelope event count hold by construction. The known divergences
//! outside that domain are ticketed (kaya-g11 status.events accounting on
//! refused ledgers, kaya-7ma digest unification) and remain unasserted
//! outside it; kaya-8yw is resolved by these assertions.
//!
//! Machine form: a plain `#[hegel::test]` that iterates all 10 cells
//! sequentially (deterministic exhaustive sweep; the state-machine macro
//! adds nothing over a fully enumerated alphabet).

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;

use assert_cmd::Command;
use hegel::TestCase;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tempfile::TempDir;

use kaya::event::parse_line;

const LEDGER: &str = ".kaya/ledger.jsonl";

/// Fixed 3-event `model.*` seed: one Question, one Hypothesis, one
/// Experiment. Byte-fixed so every cell sees identical starting bytes.
const SEED_LINES: [&str; 3] = [
    r#"{"id":"evt-b64818d83f41e6892fda19b1fc","schema":"kaya.event/v0","type":"model.entity.created","timestamp":"2026-11-16T00:00:00Z","actor":{"type":"human","name":"charly"},"subject":{"type":"Question","id":"kaya/Question/parity-seed"},"data":{"attributes":{}},"provenance":{"tool":"kaya","version":"0.1.0","reference":"kaya/Question/parity-seed"},"artifacts":[]}"#,
    r#"{"id":"evt-11d19bc35e44a3ef1c50f3bd49","schema":"kaya.event/v0","type":"model.entity.created","timestamp":"2026-11-16T00:00:00Z","actor":{"type":"human","name":"charly"},"subject":{"type":"Hypothesis","id":"kaya/Hypothesis/parity-seed"},"data":{"attributes":{"title":"parity seed hypothesis"}},"provenance":{"tool":"kaya","version":"0.1.0","reference":"kaya/Hypothesis/parity-seed"},"artifacts":[]}"#,
    r#"{"id":"evt-38c873ed860bd67c3cac87c07e","schema":"kaya.event/v0","type":"model.entity.created","timestamp":"2026-11-16T00:00:00Z","actor":{"type":"human","name":"charly"},"subject":{"type":"Experiment","id":"kaya/Experiment/parity-seed"},"data":{"attributes":{"title":"add-core-research-record"}},"provenance":{"tool":"kaya","version":"0.1.0","reference":"kaya/Experiment/parity-seed"},"artifacts":[]}"#,
];

/// A fresh valid event appended by `AppendValidEvent`. Distinct id and
/// subject so the appended record stays inside build's acceptance set.
const APPENDED_LINE: &str = r#"{"id":"evt-38c873ed860bd67c3cac87c07f","schema":"kaya.event/v0","type":"model.entity.created","timestamp":"2026-11-16T00:00:01Z","actor":{"type":"human","name":"charly"},"subject":{"type":"Question","id":"kaya/Question/parity-appended"},"data":{"attributes":{"title":"parity appended question"}},"provenance":{"tool":"kaya","version":"0.1.0","reference":"kaya/Question/parity-appended"},"artifacts":[]}"#;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Profile {
    /// `git init` + `git add` + `git commit` of the seed at setup; growth is
    /// committed again after `AppendValidEvent` so it extends HEAD.
    Committed,
    /// No git repository; build runs with `--allow-uncommitted`.
    Untracked,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mutation {
    /// Serialize one generated valid event line + `\n`.
    AppendValidEvent,
    /// Byte-copy of the seed's first line, ids included.
    AppendDuplicateLine,
    /// Flip one byte inside the final line's `"provenance"` key, keep the
    /// `\n`.
    CorruptLastLine,
    /// Drop one byte mid-line (inside a string value; may stay valid).
    DeleteByte,
    /// Drop the last k bytes of the final line → unterminated tail.
    TruncateMidLine,
}

const CELLS: [(Mutation, Profile); 10] = [
    (Mutation::AppendValidEvent, Profile::Committed),
    (Mutation::AppendDuplicateLine, Profile::Committed),
    (Mutation::CorruptLastLine, Profile::Committed),
    (Mutation::DeleteByte, Profile::Committed),
    (Mutation::TruncateMidLine, Profile::Committed),
    (Mutation::AppendValidEvent, Profile::Untracked),
    (Mutation::AppendDuplicateLine, Profile::Untracked),
    (Mutation::CorruptLastLine, Profile::Untracked),
    (Mutation::DeleteByte, Profile::Untracked),
    (Mutation::TruncateMidLine, Profile::Untracked),
];

#[hegel::test]
fn status_build_envelope_parity(_tc: TestCase) {
    for &(mutation, profile) in &CELLS {
        run_cell(mutation, profile);
    }
}

fn run_cell(mutation: Mutation, profile: Profile) {
    let ws = tempfile::tempdir().expect("workspace tempdir");
    let out = tempfile::tempdir().expect("build --out tempdir");
    let ledger_path = ws.path().join(LEDGER);

    seed_workspace(&ws, profile);

    let bytes_before = fs::read(&ledger_path).expect("seeded ledger reads");

    let bytes = apply_mutation(&bytes_before, mutation);

    // `AppendValidEvent` under the committed profile re-commits so the
    // growth extends HEAD on a line boundary; committed HEAD bytes are
    // tracked for the classifier.
    let committed: Option<Vec<u8>> = match (mutation, profile) {
        (Mutation::AppendValidEvent, Profile::Committed) => {
            fs::write(&ledger_path, &bytes).expect("appended ledger writes");
            git(&ws, &["add", LEDGER]);
            git(
                &ws,
                &["commit", "--quiet", "--message", "append valid event"],
            );
            Some(bytes.clone())
        }
        (_, Profile::Committed) => Some(bytes_before.clone()),
        (_, Profile::Untracked) => None,
    };
    // For the committed-growth cell the file is already in place; write for
    // every other cell (idempotent bytes for the committed-refused cells).
    if !matches!(
        (mutation, profile),
        (Mutation::AppendValidEvent, Profile::Committed)
    ) {
        fs::write(&ledger_path, &bytes).expect("mutated ledger writes");
    }

    // The listing baseline is taken here, after all setup (seed, mutation,
    // committed-profile git bookkeeping): everything between this point and
    // the checks is kaya's responsibility, nothing else's.
    let listing_before = listing(ws.path());

    // ---- oracle 1 (part 1) + oracle 3: `kaya status` --------------------
    let status_output = kaya(&ws)
        .args(["status"])
        .output()
        .expect("kaya status runs");
    let status = envelope(&status_output, "status");
    assert_immutable(&ws, &listing_before, &ledger_path, &bytes, "after status");

    // ---- oracle 1 (part 2) + oracle 3: `kaya build` ---------------------
    let mut build_command = kaya(&ws);
    build_command.args(["build", "--out"]).arg(out.path());
    if profile == Profile::Untracked {
        build_command.arg("--allow-uncommitted");
    }
    let build_output = build_command.output().expect("kaya build runs");
    let build = envelope(&build_output, "build");
    assert_immutable(&ws, &listing_before, &ledger_path, &bytes, "after build");

    // ---- oracle 2: classifier agreement ---------------------------------
    let accepts = build_accepts(&bytes, committed.as_deref());
    if accepts {
        assert_eq!(
            build["ok"],
            json!(true),
            "cell {mutation:?} × {profile:?}: build must accept a classified ledger"
        );
        assert_eq!(
            status["data"]["digest"], build["data"]["ledger_digest"],
            "cell {mutation:?} × {profile:?}: status/build digest parity"
        );
        assert_eq!(
            status["data"]["bytes"],
            json!(bytes.len() as u64),
            "cell {mutation:?} × {profile:?}: status byte count"
        );
        assert_eq!(
            status["data"]["pending_proposals"], build["data"]["pending_proposals"],
            "cell {mutation:?} × {profile:?}: pending-proposal parity"
        );
        assert_eq!(
            status["data"]["events"], build["data"]["events"],
            "cell {mutation:?} × {profile:?}: event-count parity on accepted ledgers"
        );
    } else {
        assert_eq!(
            build["ok"],
            json!(false),
            "cell {mutation:?} × {profile:?}: build must refuse an unclassified ledger"
        );
        let label = build["error"]["label"].as_str().unwrap_or_default();
        assert!(
            !label.is_empty(),
            "cell {mutation:?} × {profile:?}: refusal must carry a non-empty error.label"
        );
    }
}

// ---- seeding ------------------------------------------------------------

/// `kaya init` + fixed 3-event seed via `kaya import model --file`, plus the
/// profile's git state. Never touches the developer worktree.
fn seed_workspace(ws: &TempDir, profile: Profile) {
    let seed = ws.path().join("seed.jsonl");
    fs::write(&seed, seed_bytes()).expect("seed file writes");

    let init_envelope = envelope(
        &kaya(ws).args(["init"]).output().expect("kaya init runs"),
        "init",
    );
    assert_eq!(init_envelope["ok"], json!(true), "kaya init must succeed");

    let import_envelope = envelope(
        &kaya(ws)
            .args(["import", "model", "--file"])
            .arg(&seed)
            .output()
            .expect("kaya import model runs"),
        "import",
    );
    assert_eq!(
        import_envelope["ok"],
        json!(true),
        "seed import must succeed"
    );
    assert_eq!(
        import_envelope["data"]["appended"],
        json!(3),
        "seed appends 3 events"
    );

    if profile == Profile::Committed {
        git(ws, &["init", "--quiet"]);
        git(ws, &["add", LEDGER]);
        git(ws, &["commit", "--quiet", "--message", "record ledger"]);
    }
}

fn seed_bytes() -> Vec<u8> {
    let mut bytes = Vec::new();
    for line in &SEED_LINES {
        bytes.extend_from_slice(line.as_bytes());
        bytes.push(b'\n');
    }
    bytes
}

fn kaya(dir: &TempDir) -> Command {
    let mut command = Command::cargo_bin("kaya").expect("kaya binary is built");
    command.current_dir(dir.path());
    command
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null");
    command
}

/// Git inside the machine's own tempdir repo, with pinned identity so the
/// commit never depends on ambient or global config.
fn git(dir: &TempDir, args: &[&str]) {
    let output = StdCommand::new("git")
        .arg("-c")
        .arg("user.name=kaya-parity-test")
        .arg("-c")
        .arg("user.email=parity@example.invalid")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .args(args)
        .current_dir(dir.path())
        .output()
        .expect("git runs");
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

// ---- mutations -----------------------------------------------------------

/// Apply one atomic byte-level mutation to the seeded ledger bytes. Each
/// cell starts from the same fixed seed, so every mutation is deterministic.
fn apply_mutation(bytes: &[u8], mutation: Mutation) -> Vec<u8> {
    let mut bytes = bytes.to_vec();
    match mutation {
        Mutation::AppendValidEvent => {
            bytes.extend_from_slice(APPENDED_LINE.as_bytes());
            bytes.push(b'\n');
        }
        Mutation::AppendDuplicateLine => {
            // Byte-identical copy of the first line (id included).
            let first_end = bytes
                .iter()
                .position(|byte| *byte == b'\n')
                .expect("seed ends its first line");
            let first = bytes[..=first_end].to_vec();
            bytes.extend_from_slice(&first);
        }
        Mutation::CorruptLastLine | Mutation::DeleteByte => {
            // Both target a byte inside the seed's final line (its
            // "provenance" key), so the corruption never touches the
            // newline: flipping keeps the JSON recognizable but changes the
            // key; deleting a byte may leave a still-valid record — the
            // classifier decides.
            let anchor = last_line_anchor(&bytes);
            if mutation == Mutation::CorruptLastLine {
                bytes[anchor] ^= 0x01; // flip one byte inside the final line
            } else {
                bytes.remove(anchor);
            }
        }
        Mutation::TruncateMidLine => {
            // Drop the last 5 bytes of the final line: the newline plus four
            // content bytes, leaving an unterminated, unparseable tail.
            let k = 5;
            bytes.truncate(bytes.len() - k);
        }
    }
    bytes
}

/// Byte offset, within `bytes`, of the `p` in the last `"provenance"` key —
/// by construction inside the final event line (every canonical line carries
/// exactly one; arms that append lines do so before this search runs).
fn last_line_anchor(bytes: &[u8]) -> usize {
    let needle = b"\"provenance\"";
    bytes
        .windows(needle.len())
        .rposition(|window| window == needle)
        .expect("a provenance key exists in the final line")
}

// ---- classifier -----------------------------------------------------------

/// Mirror of build's acceptance domain (parse_ledger_bytes +
/// verify_committed_ledger): newline-terminated, every line a valid
/// unique-id `kaya.event/v0` record, and — for a tracked ledger — bytes
/// that extend the committed HEAD on a line boundary.
fn build_accepts(bytes: &[u8], committed: Option<&[u8]>) -> bool {
    // Every line, terminated or final unterminated tail, must parse; a
    // non-terminated tail additionally keeps the property out of the parity
    // domain (ticketed kaya-8yw), so require full termination.
    if !bytes.ends_with(b"\n") {
        return false;
    }
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let line_count = bytes.split(|byte| *byte == b'\n').count();
    for (index, raw) in bytes.split(|byte| *byte == b'\n').enumerate() {
        if raw.is_empty() {
            // split on '\n' yields one trailing empty slice; an interior
            // empty slice is a blank line, which build refuses.
            if index + 1 != line_count {
                return false;
            }
            continue;
        }
        let Ok(text) = std::str::from_utf8(raw) else {
            return false;
        };
        let Ok(event) = parse_line(text) else {
            return false;
        };
        if !seen.insert(event.id.clone()) {
            return false;
        }
    }
    match committed {
        None => true,
        Some(head) => {
            bytes.starts_with(head)
                && (bytes.len() == head.len() || bytes[head.len()..].ends_with(b"\n"))
        }
    }
}

// ---- oracles --------------------------------------------------------------

/// Oracle 1: neither command changed the ledger bytes on disk, and neither
/// wrote anything outside `--out` (the build output tempdir).
fn assert_immutable(
    ws: &TempDir,
    listing_before: &BTreeSet<PathBuf>,
    ledger_path: &Path,
    bytes: &[u8],
    when: &str,
) {
    let after = fs::read(ledger_path).expect("ledger still readable");
    assert_eq!(
        &after, bytes,
        "{when}: ledger must still hold exactly the observed bytes"
    );
    assert_eq!(
        sha256_hex(&after),
        sha256_hex(bytes),
        "{when}: ledger digest must be unchanged"
    );
    let listing_after = listing(ws.path());
    assert_eq!(
        listing_after, *listing_before,
        "{when}: nothing may be written inside the workspace"
    );
}

/// Oracle 3: exactly one pretty-JSON envelope on stdout, and
/// `exit == 0 ⇔ ok == true` (envelope-exit-code-agreement).
fn envelope(output: &std::process::Output, command: &str) -> Value {
    let stdout = &output.stdout;
    assert!(
        stdout.iter().filter(|&&byte| byte == b'\n').count() > 1,
        "{command}: envelope must be pretty-printed (multi-line JSON)"
    );
    let parsed: Value = serde_json::from_slice(stdout).unwrap_or_else(|err| {
        panic!(
            "{command}: expected exactly one JSON envelope on stdout ({err}); \
             stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    });
    assert!(
        parsed.is_object(),
        "{command}: envelope must be one JSON object"
    );
    let ok = parsed["ok"]
        .as_bool()
        .unwrap_or_else(|| panic!("{command}: envelope carries boolean ok"));
    assert_eq!(
        output.status.code().map(|code| code == 0),
        Some(ok),
        "{command}: exit code must agree with ok (0 ⇔ true)"
    );
    parsed
}

// ---- helpers --------------------------------------------------------------

fn listing(root: &Path) -> BTreeSet<PathBuf> {
    fn walk(root: &Path, dir: &Path, out: &mut BTreeSet<PathBuf>) {
        for entry in fs::read_dir(dir).expect("workspace listing reads") {
            let entry = entry.expect("workspace entry reads");
            let path = entry.path();
            if path.is_dir() {
                walk(root, &path, out);
            } else {
                out.insert(path.strip_prefix(root).expect("relative").to_path_buf());
            }
        }
    }
    let mut out = BTreeSet::new();
    walk(root, root, &mut out);
    // Git's own internals (.git/objects from auto-gc, index.lock transients)
    // are setup bookkeeping, not kaya's writes; the property holds kaya to
    // everything else. The ledger itself is still checked byte-exactly.
    out.retain(|path| !path.starts_with(".git/"));
    // The incremental-build sidecar (.kaya/build-state.json) is build output
    // by design (kaya-7r3): a content-addressed cache that must never change
    // projections, only skip their recomputation. Its deletion degrades to a
    // full deterministic render.
    out.retain(|path| path != Path::new(".kaya/build-state.json"));
    out
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

// ---- kaya-g11: status event counting --------------------------------------

/// Run `kaya status` in a workspace and return its `data` object, asserting
/// the run succeeded.
fn status_data(ws: &TempDir) -> Value {
    let envelope = envelope(
        &kaya(ws)
            .args(["status"])
            .output()
            .expect("kaya status runs"),
        "status",
    );
    assert_eq!(envelope["ok"], json!(true), "status must succeed");
    envelope["data"].clone()
}

/// A line that is well-formed JSON but not a `kaya.event/v0` record.
const NOT_AN_EVENT: &str = r#"{"note":"not a kaya event"}"#;

#[test]
fn status_counts_parsed_events_not_newlines() {
    let ws = tempfile::tempdir().expect("workspace tempdir");
    let ledger_path = ws.path().join(LEDGER);
    seed_workspace(&ws, Profile::Untracked);

    // One blank line after the terminated seed: three newline scans would
    // call this 4 events; only 3 lines parse as records.
    let mut bytes = seed_bytes();
    bytes.push(b'\n');
    fs::write(&ledger_path, &bytes).expect("blank line appends");

    let data = status_data(&ws);
    assert_eq!(
        data["events"],
        json!(3),
        "blank line must not count as an event"
    );
    assert_eq!(data["blank_lines"], json!(1), "blank line must be counted");
}

#[test]
fn status_counts_complete_unterminated_final_line() {
    let ws = tempfile::tempdir().expect("workspace tempdir");
    let ledger_path = ws.path().join(LEDGER);
    seed_workspace(&ws, Profile::Untracked);

    // Complete event line with no trailing newline: the newline count sees
    // 3; the event record exists and must count.
    let mut bytes = seed_bytes();
    bytes.extend_from_slice(APPENDED_LINE.as_bytes());
    fs::write(&ledger_path, &bytes).expect("unterminated tail writes");

    let data = status_data(&ws);
    assert_eq!(data["events"], json!(4), "complete final line must count");
    assert_eq!(
        data["replay_integrity"],
        json!("recovered-truncated-tail"),
        "unterminated tail must still be flagged"
    );
}

#[test]
fn status_excludes_unparseable_nonempty_lines() {
    let ws = tempfile::tempdir().expect("workspace tempdir");
    let ledger_path = ws.path().join(LEDGER);
    seed_workspace(&ws, Profile::Untracked);

    // Non-empty, valid JSON, but not a kaya.event/v0 record: excluded from
    // the event count, and status stays a non-failing lenient read.
    let mut bytes = seed_bytes();
    bytes.extend_from_slice(NOT_AN_EVENT.as_bytes());
    bytes.push(b'\n');
    fs::write(&ledger_path, &bytes).expect("unparseable line appends");

    let data = status_data(&ws);
    assert_eq!(data["events"], json!(3), "unparseable line must not count");
    assert_eq!(data["blank_lines"], json!(0), "non-empty line is not blank");
}
