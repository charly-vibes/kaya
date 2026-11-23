//! Whole-tree snapshots proving `check` writes nothing.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

/// Every directory path and every regular file's digest under `root`.
pub(crate) struct TreeSnapshot {
    directories: BTreeSet<PathBuf>,
    files: BTreeMap<PathBuf, String>,
}

pub(crate) fn snapshot_tree(root: &Path) -> TreeSnapshot {
    let mut snapshot = TreeSnapshot {
        directories: BTreeSet::new(),
        files: BTreeMap::new(),
    };
    collect_tree(root, root, &mut snapshot);
    snapshot
}

fn collect_tree(root: &Path, dir: &Path, snapshot: &mut TreeSnapshot) {
    let entries = fs::read_dir(dir).unwrap_or_else(|err| panic!("read {}: {err}", dir.display()));
    for entry in entries {
        let entry = entry.unwrap_or_else(|err| panic!("read an entry of {}: {err}", dir.display()));
        collect_entry(root, entry, snapshot);
    }
}

/// Add one tree entry: directories recurse, files record their digest.
fn collect_entry(root: &Path, entry: std::fs::DirEntry, snapshot: &mut TreeSnapshot) {
    let path = entry.path();
    let file_type = entry.file_type().expect("tree entry type");
    let relative = path
        .strip_prefix(root)
        .expect("path below the fixture root")
        .to_path_buf();
    if file_type.is_dir() {
        snapshot.directories.insert(relative);
        collect_tree(root, &path, snapshot);
        return;
    }
    if file_type.is_file() {
        record_file(&path, relative, snapshot);
    }
}

/// Record one regular file's digest, skipping git's transient lock files:
/// auto-maintenance creates and removes them without any check involvement.
fn record_file(path: &Path, relative: PathBuf, snapshot: &mut TreeSnapshot) {
    if path
        .extension()
        .is_some_and(|extension| extension == "lock")
    {
        return;
    }
    let bytes = fs::read(path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
    snapshot.files.insert(relative, sha256_hex(&bytes));
}

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub(crate) fn assert_tree_unchanged(before: &TreeSnapshot, after: &TreeSnapshot) {
    let created = after
        .directories
        .difference(&before.directories)
        .collect::<Vec<_>>();
    assert!(created.is_empty(), "check created directories: {created:?}");
    let removed = before
        .directories
        .difference(&after.directories)
        .collect::<Vec<_>>();
    assert!(removed.is_empty(), "check removed directories: {removed:?}");

    let created = after
        .files
        .keys()
        .filter(|path| !before.files.contains_key(*path))
        .collect::<Vec<_>>();
    assert!(created.is_empty(), "check created files: {created:?}");
    let removed = before
        .files
        .keys()
        .filter(|path| !after.files.contains_key(*path))
        .collect::<Vec<_>>();
    assert!(removed.is_empty(), "check removed files: {removed:?}");
    let changed = before
        .files
        .iter()
        .filter(|(path, digest)| after.files.get(*path) != Some(*digest))
        .map(|(path, _)| path)
        .collect::<Vec<_>>();
    assert!(changed.is_empty(), "check rewrote files: {changed:?}");
}
