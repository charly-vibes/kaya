//! Read-only verification of the committed record.
//!
//! Purpose: answer "does the committed record still equal its projections?"
//! without writing a byte, so a repository can gate on drift before it ships.
//! Responsibilities: read the ledger from an invocation root and refuse any
//! history git cannot verify, replan every research and publication document
//! through the same pure planners its writer uses, compare the expected bytes
//! with the worktree — or with the git index under `--index` — per managed
//! block and per document, and report the requested assertions and the stale
//! authored blocks the verified facts expose.
//! Rationale: verification is the inverse of compilation, never a second
//! implementation of it — sharing the planners means a `check` finding names
//! exactly the bytes `build` or `publish` would rewrite, and a clean run is
//! exactly a no-op rebuild.
//!
//! Drift paths are reported relative to the invocation root, `/`-separated.
//! The publication tree is verified whether or not it exists: a record that was
//! never published is drift, because every planned page is then missing.
//!
//! Findings and violations are distinct: drift, a missing document, and an
//! extra managed document are always defects; a requested requirement is a
//! defect because asking for it is the opt-in; and a stale authored block is
//! reported unless [`CheckOptions::fail_on_stale_authored`] turns it into a
//! defect too.

pub mod assertions;
pub mod facts;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Output};

use serde::Serialize;

use crate::event::{Event, parse_line};
use crate::ledger::sha256_hex;
use crate::model::{self, EntityType, Graph};
use crate::projection::{self, BlockKind, ProjectionError, index, report, reuse};
use crate::promotion::{PromotionError, PromotionState};
use crate::publication::{self, PublicationError};

use self::assertions::{Required, Stale};
use self::facts::Facts;

/// The append-only research ledger, relative to the invocation root.
const LEDGER: &str = ".kaya/ledger.jsonl";

/// Label for a malformed verification request.
pub const LABEL_USAGE: &str = "kaya.verification.usage";

/// Label for a path verification cannot read.
pub const LABEL_IO: &str = "kaya.verification.check_io";

/// Label refusing a ledger worktree that does not extend its committed bytes.
const LABEL_UNCOMMITTED: &str = "kaya.ledger.uncommitted_history";

/// Label refusing a ledger whose bytes differ from the pinned digest.
const LABEL_DIGEST: &str = "kaya.verification.digest_mismatch";

/// The compiled research documents the publication layer reads, in its order.
const RESEARCH_READ_ORDER: [&str; 5] = [
    "index.md",
    "overview.md",
    "understanding.md",
    "reproduction-package.md",
    "handoff.md",
];

/// A labeled refusal produced by verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationError {
    /// Failure class, for example `kaya.verification.usage`.
    pub label: String,
    /// Human-readable explanation naming the offending path or value.
    pub message: String,
}

impl VerificationError {
    /// Construct a labeled refusal.
    pub fn new(label: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            message: message.into(),
        }
    }
}

impl From<ProjectionError> for VerificationError {
    fn from(error: ProjectionError) -> Self {
        Self {
            label: error.label,
            message: error.message,
        }
    }
}

impl From<PublicationError> for VerificationError {
    fn from(error: PublicationError) -> Self {
        Self {
            label: error.label,
            message: error.message,
        }
    }
}

impl From<model::ModelError> for VerificationError {
    fn from(error: model::ModelError) -> Self {
        Self {
            label: error.label,
            message: error.message,
        }
    }
}

impl From<PromotionError> for VerificationError {
    fn from(error: PromotionError) -> Self {
        Self {
            label: error.label,
            message: error.message,
        }
    }
}

impl std::fmt::Display for VerificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.label, self.message)
    }
}

impl std::error::Error for VerificationError {}

/// One drift finding: a document path, the managed block when the finding
/// concerns one, and the violation kind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Entry {
    /// Document path relative to the invocation root, `/`-separated.
    pub path: String,
    /// Generated block the finding concerns; `None` for a document finding.
    pub block: Option<String>,
    /// `missing`, `extra`, or `changed`.
    pub kind: &'static str,
}

/// What a check should verify beyond the always-on drift comparison.
///
/// The flags only widen what counts as a defect: drift, a missing document, or
/// an extra managed document always fails, while a requirement and the stale
/// report fail only when the caller asked for them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CheckOptions {
    /// Read the ledger and every compared document from the git index.
    pub index: bool,
    /// Dotted `family.kind` names the record must emit.
    pub require_kinds: Vec<String>,
    /// Canonical capture sources the record must hold events for.
    pub require_sources: Vec<String>,
    /// Turn a non-empty stale report into exit 1.
    pub fail_on_stale_authored: bool,
}

/// The result of one check: every finding, sorted by path.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Outcome {
    /// Drift findings, sorted by `(path, block, kind)`.
    pub drift: Vec<Entry>,
    /// Requested kinds or sources the record does not satisfy.
    pub required: Vec<Required>,
    /// Authored blocks whose cited evidence changed, sorted by
    /// `(path, block, evidence)`.
    pub stale: Vec<Stale>,
    /// What the check observed in the verified record.
    pub facts: Facts,
    /// Whether a non-empty stale report fails the check.
    pub enforce_stale: bool,
}

impl Outcome {
    /// The verdict the exit code carries: 0 clean, 1 drift or an unmet
    /// assertion the caller enabled.
    pub fn exit_code(&self) -> u8 {
        let violated = !self.drift.is_empty() || !self.required.is_empty();
        let stale = self.enforce_stale && !self.stale.is_empty();
        if violated || stale { 1 } else { 0 }
    }
}

/// The tree a check reads: document text and Markdown directory listings.
///
/// Worktree mode reads the files a developer sees. Index mode reads the blobs
/// a commit would record (`:<path>`), so a partially staged tree verifies
/// exactly as the commit would leave it. Index mode never falls back to the
/// worktree: an entry the index cannot serve is an I/O failure.
#[derive(Debug, Clone)]
pub enum Source {
    /// Read the worktree under the root.
    Worktree(PathBuf),
    /// Read staged blobs from the git index under the root.
    Index(PathBuf),
}

impl Source {
    /// Read the worktree under `root`.
    pub fn worktree(root: &Path) -> Self {
        Self::Worktree(root.to_path_buf())
    }

    /// Read the git index under `root`.
    pub fn index(root: &Path) -> Self {
        Self::Index(root.to_path_buf())
    }

    /// The root both modes resolve paths against.
    fn root(&self) -> &Path {
        match self {
            Self::Worktree(root) | Self::Index(root) => root,
        }
    }

    /// Read a document, treating absence as absence rather than failure.
    pub fn read_text(&self, path: &Path) -> Result<Option<String>, VerificationError> {
        match self {
            Self::Worktree(_) => match fs::read_to_string(path) {
                Ok(text) => Ok(Some(text)),
                Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
                Err(error) => Err(read_failure(path, error)),
            },
            Self::Index(_) => {
                let relative = self.index_path(path)?;
                match self.read_index(&relative)? {
                    Some(bytes) => String::from_utf8(bytes).map(Some).map_err(|_| {
                        index_failure(&relative, "the staged bytes are not valid UTF-8")
                    }),
                    None => Ok(None),
                }
            }
        }
    }

    /// Read a target byte-for-byte, treating absence as absence rather than
    /// failure.
    pub fn read_bytes(&self, path: &Path) -> Result<Option<Vec<u8>>, VerificationError> {
        match self {
            Self::Worktree(_) => match fs::read(path) {
                Ok(bytes) => Ok(Some(bytes)),
                Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
                Err(error) => Err(read_failure(path, error)),
            },
            Self::Index(_) => {
                let relative = self.index_path(path)?;
                self.read_index(&relative)
            }
        }
    }

    /// The Markdown files directly under `dir`, sorted; an absent directory
    /// lists nothing.
    pub fn list_markdown(&self, dir: &Path) -> Result<Vec<PathBuf>, VerificationError> {
        match self {
            Self::Worktree(_) => {
                let entries = match fs::read_dir(dir) {
                    Ok(entries) => entries,
                    Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
                    Err(error) => {
                        return Err(VerificationError::new(
                            LABEL_IO,
                            format!("failed to list \"{}\": {error}", dir.display()),
                        ));
                    }
                };
                let mut paths = Vec::new();
                for entry in entries {
                    let entry = entry.map_err(|error| {
                        VerificationError::new(
                            LABEL_IO,
                            format!("failed to list \"{}\": {error}", dir.display()),
                        )
                    })?;
                    let path = entry.path();
                    if path.extension().is_some_and(|extension| extension == "md") {
                        paths.push(path);
                    }
                }
                paths.sort();
                Ok(paths)
            }
            Self::Index(_) => self.list_index_markdown(dir),
        }
    }

    /// The staged Markdown files directly under `dir`, sorted, as index paths.
    ///
    /// `git ls-files` lists the index recursively, so paths below a nested
    /// directory are dropped: the worktree listing this mirrors is shallow. An
    /// absent directory lists nothing because no staged path carries its
    /// prefix, and the worktree is never consulted.
    fn list_index_markdown(&self, dir: &Path) -> Result<Vec<PathBuf>, VerificationError> {
        let relative = self.index_path(dir)?;
        let pathspec = if relative.is_empty() {
            ".".to_owned()
        } else {
            format!(":(literal){relative}")
        };
        let output =
            git(self.root(), &["ls-files", "-z", "--", pathspec.as_str()]).map_err(|error| {
                index_failure(&relative, &format!("`git ls-files` could not run: {error}"))
            })?;
        if !output.status.success() {
            return Err(index_failure(&relative, "`git ls-files` failed"));
        }
        let prefix = if relative.is_empty() {
            String::new()
        } else {
            format!("{relative}/")
        };
        let mut paths = Vec::new();
        for entry in output.stdout.split(|byte| *byte == 0) {
            if entry.is_empty() {
                continue;
            }
            let Ok(text) = std::str::from_utf8(entry) else {
                return Err(index_failure(
                    &relative,
                    "a path listed by git is not valid UTF-8",
                ));
            };
            let Some(name) = text.strip_prefix(prefix.as_str()) else {
                continue;
            };
            // Direct children only: a nested path would not be a file the
            // shallow worktree listing returns either.
            if name.contains('/') {
                continue;
            }
            let path = Path::new(name);
            if path.extension().is_some_and(|extension| extension == "md") {
                paths.push(dir.join(path));
            }
        }
        paths.sort();
        Ok(paths)
    }

    /// A compared path as a `/`-separated path relative to the root.
    fn index_path(&self, path: &Path) -> Result<String, VerificationError> {
        let root = self.root();
        let outside = || {
            VerificationError::new(
                LABEL_IO,
                format!(
                    "failed to read \"{}\" from the git index: the path is outside the root \
                     \"{}\"",
                    path.display(),
                    root.display()
                ),
            )
        };
        let Ok(relative) = path.strip_prefix(root) else {
            return Err(outside());
        };
        let mut parts: Vec<String> = Vec::new();
        for component in relative.components() {
            match component {
                Component::Normal(part) => parts.push(part.to_string_lossy().into_owned()),
                Component::CurDir => {}
                Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                    return Err(outside());
                }
            }
        }
        Ok(parts.join("/"))
    }

    /// The staged bytes at a root-relative path: `None` when the index does
    /// not record it, an I/O refusal when it cannot be served.
    fn read_index(&self, relative: &str) -> Result<Option<Vec<u8>>, VerificationError> {
        // `:(literal)` keeps a document name holding `*` or `?` a plain path
        // while still resolving against the reader's root.
        let pathspec = format!(":(literal){relative}");
        // `--stage -z` answers presence and merge state in one call: every
        // entry is a NUL-terminated `<mode> <object> <stage>\t<path>`, and a
        // conflict is the same path recorded at stages 1..3 instead of 0.
        let staged = git(
            self.root(),
            &["ls-files", "--stage", "-z", "--", pathspec.as_str()],
        )
        .map_err(|error| {
            index_failure(
                relative,
                &format!("`git ls-files --stage` could not run: {error}"),
            )
        })?;
        if !staged.status.success() {
            return Err(index_failure(relative, "`git ls-files --stage` failed"));
        }
        let mut present = false;
        for entry in staged.stdout.split(|byte| *byte == 0) {
            if entry.is_empty() {
                continue;
            }
            present = true;
            let stage = index_entry_stage(entry).ok_or_else(|| {
                index_failure(relative, "git printed an index entry kaya cannot parse")
            })?;
            if stage != 0 {
                return Err(index_failure(
                    relative,
                    "the index entry is unmerged; resolve the conflict before checking",
                ));
            }
        }
        if !present {
            return Ok(None);
        }

        // `:./<path>` resolves against the reader's root, which `git -C` makes
        // the working directory of every command here.
        let spec = format!(":./{relative}");
        let shown = git(self.root(), &["show", &spec]).map_err(|error| {
            index_failure(
                relative,
                &format!("`git show {spec}` could not run: {error}"),
            )
        })?;
        if !shown.status.success() {
            return Err(index_failure(
                relative,
                &format!("`git show {spec}` failed"),
            ));
        }
        Ok(Some(shown.stdout))
    }

    /// A path as reported: relative to the root, `/`-separated.
    pub fn relative(&self, path: &Path) -> String {
        let Ok(relative) = path.strip_prefix(self.root()) else {
            return path.to_string_lossy().replace('\\', "/");
        };
        let mut parts: Vec<String> = Vec::new();
        for component in relative.components() {
            match component {
                Component::Normal(part) => parts.push(part.to_string_lossy().into_owned()),
                Component::ParentDir => parts.push("..".to_owned()),
                Component::CurDir | Component::RootDir | Component::Prefix(_) => {}
            }
        }
        parts.join("/")
    }

    /// One drift entry rooted at this source.
    fn entry(&self, path: &Path, block: Option<String>, kind: &'static str) -> Entry {
        Entry {
            path: self.relative(path),
            block,
            kind,
        }
    }
}

/// Verify the record under `root` without writing anything.
///
/// The ledger is read once, its committed history is verified (a worktree that
/// does not extend `HEAD` is refused before any finding), and every projection
/// is replanned from the verified events. `out` and `publish_out` are
/// directories relative to `root`. With `options.index`, the ledger and every
/// compared document come from the git index, so the verdict is the one a
/// commit of the staged tree would earn. The requested assertions and the
/// stale report are always evaluated; `options` only decides which of them the
/// exit code treats as a defect.
pub fn check(
    root: &Path,
    out: &Path,
    publish_out: &Path,
    expect_digest: Option<&str>,
    options: &CheckOptions,
) -> Result<Outcome, VerificationError> {
    assertions::validate_request(&options.require_kinds, &options.require_sources)?;

    let expect_digest = match expect_digest {
        Some(digest) if valid_digest(digest) => Some(digest),
        Some(digest) => {
            return Err(VerificationError::new(
                LABEL_USAGE,
                format!(
                    "--expect-digest \"{digest}\" is not a 64-character hexadecimal sha256 digest"
                ),
            ));
        }
        None => None,
    };

    let source = if options.index {
        Source::index(root)
    } else {
        Source::worktree(root)
    };
    let out = root.join(out);
    let publish_out = root.join(publish_out);

    let (events, ledger_digest) = read_ledger(&source, expect_digest)?;
    let graph = model::build(&events)?;
    let promotion = PromotionState::from_events(&events)?;

    let (expected, omitted) = plan_research(&source, &out, &events, &graph, &promotion)?;

    let mut drift = Vec::new();
    compare_research(&source, &out, &expected, &mut drift)?;
    compare_publication(&source, &out, &publish_out, &expected, &mut drift)?;

    drift.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then_with(|| left.block.cmp(&right.block))
            .then_with(|| left.kind.cmp(right.kind))
    });
    // Two comparisons may name the same bytes — a planned page the publication
    // layer lumps with an omission, for example — and a finding is a fact, so
    // the reported drift counts it once.
    drift.dedup();
    let facts = Facts::derive(&events, ledger_digest, &expected, &omitted)?;
    let required = assertions::required(&facts, &options.require_kinds, &options.require_sources);
    let stale = assertions::stale(&source, &expected, &events);
    Ok(Outcome {
        drift,
        required,
        stale,
        facts,
        enforce_stale: options.fail_on_stale_authored,
    })
}

/// Whether `digest` is exactly 64 hexadecimal characters.
fn valid_digest(digest: &str) -> bool {
    digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit())
}

/// Read and verify the ledger, returning the events it records and the sha256
/// of the exact verified bytes.
///
/// Committed history is mandatory: no flag relaxes it. In worktree mode a
/// tracked ledger must extend its committed `HEAD` bytes on a line boundary,
/// and an untracked one has no history to verify at all, so both are refused
/// with the label `build` uses. In index mode the staged ledger is verified
/// against `HEAD` as well, except that a `HEAD` carrying no ledger copy leaves
/// the baseline empty: a staged first record is verifiable, not refused. A
/// `HEAD` that does carry the ledger but cannot serve its bytes is an I/O
/// refusal, never an empty baseline. A pinned digest narrows the accepted
/// bytes further; it never widens them.
fn read_ledger(
    source: &Source,
    expect_digest: Option<&str>,
) -> Result<(Vec<Event>, String), VerificationError> {
    let root = source.root();
    let path = root.join(LEDGER);
    let missing = || {
        VerificationError::new(
            "missing-ledger",
            match source {
                Source::Worktree(_) => {
                    format!(
                        "missing ledger {}: run `kaya init` to create it",
                        path.display()
                    )
                }
                Source::Index(_) => format!(
                    "missing ledger {}: stage the ledger before checking with --index",
                    path.display()
                ),
            },
        )
    };

    let bytes = match source {
        Source::Worktree(_) => fs::read(&path).map_err(|error| match error.kind() {
            io::ErrorKind::NotFound => missing(),
            _ => read_failure(&path, error),
        })?,
        Source::Index(_) => source.read_bytes(&path)?.ok_or_else(missing)?,
    };

    let committed = match source {
        Source::Worktree(_) => committed_history(root)?,
        Source::Index(_) => staged_history(root)?,
    };
    verify_history(&committed, &bytes, &path)?;

    let digest = sha256_hex(&bytes);
    if let Some(expected) = expect_digest
        && digest != expected
    {
        return Err(VerificationError::new(
            LABEL_DIGEST,
            format!(
                "ledger {} does not match the expected digest {expected}: the verified \
                 bytes digest {digest}",
                path.display()
            ),
        ));
    }

    let events = parse_events(&bytes, &path)?;
    Ok((events, digest))
}

/// The committed `HEAD` bytes of the ledger under `root`.
///
/// An untracked ledger or a tracked ledger absent from `HEAD` has no committed
/// history to verify against, so both are refusals: verification never falls
/// back to worktree bytes git did not record.
fn committed_history(root: &Path) -> Result<Vec<u8>, VerificationError> {
    let tracked = git(root, &["ls-files", "--error-unmatch", LEDGER]);
    match tracked {
        Ok(output) if output.status.success() => {}
        Ok(_) | Err(_) => {
            return Err(VerificationError::new(
                LABEL_UNCOMMITTED,
                format!(
                    "ledger {LEDGER} is untracked, so git cannot verify its history; commit \
                     the ledger before checking"
                ),
            ));
        }
    }

    let spec = format!("HEAD:{LEDGER}");
    let committed = git(root, &["show", &spec]).map_err(|error| {
        VerificationError::new(
            LABEL_UNCOMMITTED,
            format!("ledger {LEDGER} is tracked but `git show {spec}` could not run: {error}"),
        )
    })?;
    if !committed.status.success() {
        return Err(VerificationError::new(
            LABEL_UNCOMMITTED,
            format!(
                "ledger {LEDGER} is tracked but HEAD carries no committed copy; commit the \
                 ledger before checking"
            ),
        ));
    }
    Ok(committed.stdout)
}

/// The `HEAD` bytes of the ledger, used as the baseline for an index check.
///
/// `HEAD` is asked for the ledger by name first, so its two kinds of absence
/// stay distinct from a failure to read it: a `HEAD` that does not list the
/// ledger carries no copy — the first commit of a record — and an unborn
/// `HEAD` names no commit at all, so both leave the baseline empty and the
/// staged ledger is the whole history. A `HEAD` that lists the ledger but
/// cannot serve its bytes is an I/O refusal: verification never silently
/// substitutes an empty baseline for bytes git failed to read.
fn staged_history(root: &Path) -> Result<Vec<u8>, VerificationError> {
    let tree = git(
        root,
        &["ls-tree", "--name-only", "-z", "HEAD", "--", LEDGER],
    )
    .map_err(|error| {
        VerificationError::new(
            LABEL_IO,
            format!("failed to read {LEDGER} from HEAD: `git ls-tree HEAD` could not run: {error}"),
        )
    })?;
    if !tree.status.success() {
        if head_is_unborn(root) {
            return Ok(Vec::new());
        }
        return Err(VerificationError::new(
            LABEL_IO,
            format!(
                "failed to read {LEDGER} from HEAD: `git ls-tree HEAD` failed, so HEAD cannot \
                 be read: {}",
                String::from_utf8_lossy(&tree.stderr).trim()
            ),
        ));
    }
    let listed = tree
        .stdout
        .split(|byte| *byte == 0)
        .any(|entry| entry == LEDGER.as_bytes());
    if !listed {
        return Ok(Vec::new());
    }

    // `HEAD:./<path>` resolves from `git -C root`, so a nested root reads its
    // own ledger rather than the repository root's.
    let spec = format!("HEAD:./{LEDGER}");
    let committed = git(root, &["show", &spec]).map_err(|error| {
        VerificationError::new(
            LABEL_IO,
            format!("failed to read {LEDGER} from HEAD: `git show {spec}` could not run: {error}"),
        )
    })?;
    if !committed.status.success() {
        return Err(VerificationError::new(
            LABEL_IO,
            format!(
                "failed to read {LEDGER} from HEAD: HEAD lists the ledger but `git show {spec}` \
                 failed, so its committed history is unknown: {}",
                String::from_utf8_lossy(&committed.stderr).trim()
            ),
        ));
    }
    Ok(committed.stdout)
}

/// Whether `HEAD` names no commit yet: the state of a repository before its
/// first commit, where the record it stages is its whole history.
///
/// `HEAD` must be a symbolic ref to a branch that does not exist yet. A
/// detached `HEAD`, or a branch whose ref is present but unreadable, is a
/// broken repository rather than a new one, so the caller reports the read
/// failure instead of trusting an empty baseline.
fn head_is_unborn(root: &Path) -> bool {
    let head = match git(root, &["symbolic-ref", "--quiet", "HEAD"]) {
        Ok(output) if output.status.success() => output,
        Ok(_) | Err(_) => return false,
    };
    let Ok(branch) = std::str::from_utf8(&head.stdout) else {
        return false;
    };
    match git(root, &["show-ref", "--verify", "--quiet", branch.trim()]) {
        Ok(output) => !output.status.success(),
        Err(_) => false,
    }
}

/// Run one git command inside `root`.
fn git(root: &Path, args: &[&str]) -> io::Result<Output> {
    Command::new("git").arg("-C").arg(root).args(args).output()
}

/// Require committed bytes to prefix the current ledger and complete its lines.
fn verify_history(committed: &[u8], current: &[u8], path: &Path) -> Result<(), VerificationError> {
    let prefix_ok =
        current.starts_with(committed) && (committed.is_empty() || committed.ends_with(b"\n"));
    let growth_ok = current.len() == committed.len()
        || current[committed.len().min(current.len())..].ends_with(b"\n");
    if prefix_ok && growth_ok {
        return Ok(());
    }
    let line = first_unverified_line(committed, current);
    Err(VerificationError::new(
        LABEL_UNCOMMITTED,
        format!(
            "ledger {} does not extend its committed HEAD history: line {line} is unverified; \
             commit the ledger before checking",
            path.display()
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
fn parse_events(bytes: &[u8], path: &Path) -> Result<Vec<Event>, VerificationError> {
    let mut events = Vec::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for (index, raw) in bytes.split_inclusive(|byte| *byte == b'\n').enumerate() {
        let line = index + 1;
        let content = raw.strip_suffix(b"\n").unwrap_or(raw);
        let text = std::str::from_utf8(content).map_err(|error| {
            VerificationError::new(
                "kaya.ledger.record_invalid",
                format!("{} line {line} is not UTF-8: {error}", path.display()),
            )
        })?;
        let event = parse_line(text).map_err(|error| {
            VerificationError::new(
                error.label,
                format!("{} line {line}: {}", path.display(), error.message),
            )
        })?;
        if !seen.insert(event.id.clone()) {
            return Err(VerificationError::new(
                "kaya.ledger.id_reuse",
                format!(
                    "{} line {line} reuses event id \"{}\"; ids are never reused — record a \
                     correction naming the id in data.supersedes",
                    path.display(),
                    event.id
                ),
            ));
        }
        events.push(event);
    }
    Ok(events)
}

/// The research documents the projections emit, keyed by absolute path, and
/// the dotted `family.kind` names a planner omitted for having no records.
///
/// Every planner receives the document already at its path — worktree or
/// staged bytes, whichever mode reads — so the map is the fixed point a clean
/// `build` would leave: generated bodies are replanned while authored prose,
/// foreign blocks, and marker placement survive untouched. Planning from
/// absent bytes would instead reduce each document to its generated blocks and
/// report the human's bytes as drift. An omission is collected even when its
/// plan leaves no document: the empty kind is why the bytes are absent.
fn plan_research(
    source: &Source,
    out: &Path,
    events: &[Event],
    graph: &Graph,
    promotion: &PromotionState,
) -> Result<(BTreeMap<PathBuf, String>, Vec<String>), VerificationError> {
    let mut expected = BTreeMap::new();
    let mut omitted = Vec::new();

    let index_path = out.join("index.md");
    let existing = source.read_text(&index_path)?;
    let (document, _summary) = index::plan(graph, &index_path, existing.as_deref(), events.len())?;
    expected.insert(index_path, document);

    let experiments = out.join("experiments");
    for node in graph
        .nodes()
        .filter(|node| node.entity == EntityType::Experiment)
    {
        let path = experiments.join(format!("{}.md", node.id.replace('/', "-")));
        let existing = source.read_text(&path)?;
        let planned = report::plan(graph, &node.id, existing.as_deref(), promotion)?;
        // An omitted kind is a fact even when the plan leaves no document:
        // the omission is why the bytes are absent.
        omitted.extend(planned.omitted.iter().map(omitted_kind));
        if !planned.document.trim().is_empty() {
            expected.insert(path, planned.document);
        }
    }

    let understanding_path = out.join("understanding.md");
    let existing = source.read_text(&understanding_path)?;
    let planned = report::plan_understanding(graph, existing.as_deref())?;
    omitted.extend(planned.omitted.iter().map(omitted_kind));
    if !planned.document.trim().is_empty() {
        expected.insert(understanding_path, planned.document);
    }

    let overview_path = out.join("overview.md");
    let existing = source.read_text(&overview_path)?;
    let planned = report::plan_overview(graph, existing.as_deref())?;
    omitted.extend(planned.omitted.iter().map(omitted_kind));
    if !planned.document.trim().is_empty() {
        expected.insert(overview_path, planned.document);
    }

    let inputs = reuse::inputs(graph)?;
    let mut existing = BTreeMap::new();
    for (_, path) in reuse::documents(out) {
        let document = source.read_text(&path)?;
        existing.insert(path, document);
    }
    for planned in reuse::plan(&inputs, out, &existing)? {
        omitted.extend(planned.omitted.iter().map(omitted_kind));
        if let Some(document) = planned.document {
            expected.insert(planned.path, document);
        }
    }

    omitted.sort();
    omitted.dedup();
    Ok((expected, omitted))
}

/// One planner omission as a dotted `family.kind` name.
fn omitted_kind(omitted: &report::Omitted) -> String {
    format!("{}.{}", omitted.family, omitted.kind)
}

/// Compare every expected research document with the source's bytes.
fn compare_research(
    source: &Source,
    out: &Path,
    expected: &BTreeMap<PathBuf, String>,
    drift: &mut Vec<Entry>,
) -> Result<(), VerificationError> {
    for (path, document) in expected {
        match source.read_text(path)? {
            Some(actual) => compare_document(source, path, document, &actual, drift)?,
            None => drift.push(source.entry(path, None, "missing")),
        }
    }

    // A managed document the projection does not emit is not ours to keep:
    // the projection is the authority on what exists under `--out`.
    for path in existing_documents(source, out)? {
        if expected.contains_key(&path) {
            continue;
        }
        let Some(document) = source.read_text(&path)? else {
            continue;
        };
        if is_managed(&document)? {
            drift.push(source.entry(&path, None, "extra"));
        }
    }
    Ok(())
}

/// The documents under `out` a projection may own.
fn existing_documents(source: &Source, out: &Path) -> Result<Vec<PathBuf>, VerificationError> {
    let mut paths = vec![
        out.join("index.md"),
        out.join("overview.md"),
        out.join("understanding.md"),
    ];
    for (_, path) in reuse::documents(out) {
        paths.push(path);
    }
    paths.extend(source.list_markdown(&out.join("experiments"))?);
    Ok(paths)
}

/// Compare the publication tree the declarations compose with the source.
///
/// The plan carries the bytes `kaya publish` would leave, computed over the
/// source's own files, so a page or export the plan would rewrite is
/// `changed`, one it would create is `missing`, and one it would unlink is
/// `extra`. An absent publication directory is drift rather than a silent
/// pass: every planned page is then missing.
fn compare_publication(
    source: &Source,
    out: &Path,
    publish_out: &Path,
    expected: &BTreeMap<PathBuf, String>,
    drift: &mut Vec<Entry>,
) -> Result<(), VerificationError> {
    let documents = publication::read_documents_from(publication_inputs(expected, out))?;
    let declarations = publication::default_declarations();
    let plan = publication::plan(
        &declarations,
        &documents,
        publish_out,
        |path| source.read_text(path).map_err(as_publication_error),
        |path| source.read_bytes(path).map_err(as_publication_error),
    )?;

    for page in &plan.pages {
        match source.read_text(&page.path)? {
            Some(actual) => compare_document(source, &page.path, &page.document, &actual, drift)?,
            None => drift.push(source.entry(&page.path, None, "missing")),
        }
    }

    for removal in &plan.removals {
        match (source.read_text(&removal.path)?, &removal.document) {
            (Some(actual), Some(document)) => {
                compare_document(source, &removal.path, document, &actual, drift)?;
            }
            (Some(_), None) => drift.push(source.entry(&removal.path, None, "extra")),
            (None, Some(_)) => drift.push(source.entry(&removal.path, None, "missing")),
            (None, None) => {}
        }
    }

    match source.read_bytes(&plan.export_path)? {
        Some(actual) => {
            if actual != plan.export {
                drift.push(source.entry(&plan.export_path, None, "changed"));
            }
        }
        None => drift.push(source.entry(&plan.export_path, None, "missing")),
    }

    let planned_blocks: BTreeSet<&str> =
        plan.pages.iter().map(|page| page.block.as_str()).collect();
    // A page an omission plans to strip or unlink is already accounted for
    // above: reporting it again as a stray would make one stale page two
    // findings for the same bytes.
    let removal_paths: BTreeSet<&Path> = plan
        .removals
        .iter()
        .map(|removal| removal.path.as_path())
        .collect();
    for path in source.list_markdown(publish_out)? {
        if plan.pages.iter().any(|page| page.path == path) || removal_paths.contains(path.as_path())
        {
            continue;
        }
        let Some(document) = source.read_text(&path)? else {
            continue;
        };
        let stray = generated_bodies(&document)?.keys().any(|name| {
            name.starts_with("publication.") && !planned_blocks.contains(name.as_str())
        });
        if stray {
            drift.push(source.entry(&path, None, "extra"));
        }
    }
    Ok(())
}

/// The expected research documents in the publication layer's read order.
fn publication_inputs(expected: &BTreeMap<PathBuf, String>, out: &Path) -> Vec<(PathBuf, String)> {
    let mut pairs = Vec::new();
    for name in RESEARCH_READ_ORDER {
        let path = out.join(name);
        if let Some(document) = expected.get(&path) {
            pairs.push((path, document.clone()));
        }
    }
    let experiments = out.join("experiments");
    for (path, document) in expected {
        if path.parent() == Some(experiments.as_path()) {
            pairs.push((path.clone(), document.clone()));
        }
    }
    pairs
}

/// Compare one document with its expected bytes, per block and per document.
///
/// A generated block whose body differs is one `changed` finding naming the
/// block. Bytes outside generated blocks — markers, authored prose, block
/// order — are compared with every generated body removed, so a body-only
/// change stays a single block finding and anything else is one document-level
/// `changed` finding.
fn compare_document(
    source: &Source,
    path: &Path,
    expected: &str,
    actual: &str,
    drift: &mut Vec<Entry>,
) -> Result<(), VerificationError> {
    let expected_blocks = generated_bodies(expected)?;
    let actual_blocks = generated_bodies(actual)?;
    let names: BTreeSet<&String> = expected_blocks.keys().chain(actual_blocks.keys()).collect();
    for name in names {
        if expected_blocks.get(name) != actual_blocks.get(name) {
            drift.push(source.entry(path, Some(name.clone()), "changed"));
        }
    }

    if normalized(expected)? != normalized(actual)? {
        drift.push(source.entry(path, None, "changed"));
    }
    Ok(())
}

/// Every generated block body in a document, keyed by block name.
fn generated_bodies(document: &str) -> Result<BTreeMap<String, String>, VerificationError> {
    let mut bodies = BTreeMap::new();
    for block in projection::scan(document)? {
        if let BlockKind::Generated(name) = block.kind {
            bodies.entry(name).or_insert(block.body);
        }
    }
    Ok(bodies)
}

/// A document with every generated block body removed, markers kept.
fn normalized(document: &str) -> Result<String, VerificationError> {
    let mut text = document.to_owned();
    for block in projection::scan(document)? {
        if let BlockKind::Generated(name) = block.kind {
            text = projection::replace_generated(&text, &name, "")?;
        }
    }
    Ok(text)
}

/// Whether a document carries at least one kaya-managed block.
fn is_managed(document: &str) -> Result<bool, VerificationError> {
    Ok(projection::scan(document)?
        .iter()
        .any(|block| matches!(&block.kind, BlockKind::Generated(_))))
}

/// A worktree read failure as the verification I/O class.
fn read_failure(path: &Path, error: io::Error) -> VerificationError {
    VerificationError::new(
        LABEL_IO,
        format!("failed to read \"{}\": {error}", path.display()),
    )
}

/// An index read failure as the verification I/O class, naming the path.
///
/// The path is the root-relative form git reports, which is also the form the
/// drift report and its fixes use; the failing read is named here so a
/// partially staged tree never falls back to worktree bytes.
fn index_failure(relative: &str, detail: &str) -> VerificationError {
    VerificationError::new(
        LABEL_IO,
        format!("failed to read \"{relative}\" from the git index: {detail}"),
    )
}

/// The stage number from one `git ls-files --stage -z` entry
/// (`<mode> <object> <stage>\t<path>`), or `None` when git's line is not in
/// that shape.
fn index_entry_stage(entry: &[u8]) -> Option<u32> {
    let tab = entry.iter().position(|byte| *byte == b'\t')?;
    let metadata = std::str::from_utf8(&entry[..tab]).ok()?;
    metadata.split(' ').nth(2)?.parse().ok()
}

/// The same labeled refusal in the publication layer's error type, which is
/// the one a plan's read closures report.
fn as_publication_error(error: VerificationError) -> PublicationError {
    PublicationError {
        label: error.label,
        message: error.message,
    }
}
