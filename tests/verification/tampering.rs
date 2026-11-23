//! Helpers that tamper with a committed record's bytes.

use std::fs;
use std::path::Path;

use crate::common::{
    AUTHORED_PROSE, FOREIGN_BLOCK, INDEX, INDEX_BEGIN, INDEX_END, LEDGER, PUBLICATION_EXPORT,
    STRAY, git, git_output, git_stdout, git_with_stdin,
};

/// Seed the research index with authored prose and a foreign managed block
/// before the projection appends its own block.
pub(crate) fn seed_authored_index(root: &Path) {
    let path = root.join(INDEX);
    fs::create_dir_all(path.parent().expect("the index has a parent"))
        .expect("create the research directory");
    let mut document = String::from("# Research index\n\n");
    document.push_str(FOREIGN_BLOCK);
    document.push_str("\n<!-- kaya:authored -->\n");
    document.push_str(AUTHORED_PROSE);
    document.push_str("\n<!-- kaya:end authored -->\n");
    fs::write(&path, document).expect("seed the authored research index");
}

/// Rewrite the first body byte of the generated block starting at `begin` in
/// `relative`, leaving the surrounding bytes untouched.
pub(crate) fn corrupt_generated_block(root: &Path, relative: &str, begin: &str) {
    let path = root.join(relative);
    let text = fs::read_to_string(&path).expect("the generated document is readable");
    let after_marker = text
        .find(begin)
        .unwrap_or_else(|| panic!("{relative} carries the generated marker {begin}"))
        + begin.len();
    let body = text[after_marker..]
        .find('\n')
        .map(|offset| after_marker + offset + 1)
        .expect("the managed block marker ends its line");
    let mut bytes = text.into_bytes();
    bytes[body] = if bytes[body] == b'#' { b'!' } else { b'#' };
    fs::write(&path, bytes).expect("write the drifted document");
}

/// Rewrite the first body byte of the index's managed research-map block.
pub(crate) fn corrupt_index_block(root: &Path) {
    corrupt_generated_block(root, INDEX, INDEX_BEGIN);
}

/// Remove the publication export the declarations require.
pub(crate) fn remove_publication_export(root: &Path) {
    fs::remove_file(root.join(PUBLICATION_EXPORT)).expect("remove the publication export");
}

/// Delete the generated document the projection requires.
pub(crate) fn remove_generated_index(root: &Path) {
    fs::remove_file(root.join(INDEX)).expect("remove the generated index");
}

/// Add a managed document that the projection would not emit.
pub(crate) fn add_stray_managed_document(root: &Path) {
    let path = root.join(STRAY);
    fs::create_dir_all(path.parent().expect("stray path has a parent"))
        .expect("create the stray document directory");
    let document = format!("{INDEX_BEGIN}\n### Stray research map\n{INDEX_END}\n");
    fs::write(&path, document).expect("write the stray managed document");
}

/// Rewrite one byte inside the first committed ledger line, keeping it valid
/// JSON while breaking the prefix relation with `HEAD`.
pub(crate) fn corrupt_ledger_history(root: &Path) {
    let path = root.join(LEDGER);
    let text = fs::read_to_string(&path).expect("the committed ledger is readable");
    let (first, rest) = text
        .split_once('\n')
        .expect("the committed ledger holds a line");
    let tampered = first.replace("\"charly\"", "\"charlx\"");
    assert_ne!(
        first, tampered,
        "the seed's first ledger line names its actor: {first}"
    );
    fs::write(&path, format!("{tampered}\n{rest}")).expect("rewrite the ledger worktree bytes");
}

/// Stage `relative` exactly as the worktree holds it now.
pub(crate) fn stage(root: &Path, relative: &str) {
    git(root, &["add", "--", relative]);
}

/// Restore the worktree copy of `relative` to the bytes `HEAD` records.
pub(crate) fn restore_worktree_from_head(root: &Path, relative: &str) {
    let spec = format!("HEAD:{relative}");
    let bytes = git_stdout(root, &["show", &spec]);
    fs::write(root.join(relative), bytes).expect("restore the worktree document from HEAD");
}

/// The worktree bytes of `relative`.
pub(crate) fn worktree_bytes(root: &Path, relative: &str) -> Vec<u8> {
    fs::read(root.join(relative)).unwrap_or_else(|error| {
        panic!("the worktree copy of {relative} is readable: {error}");
    })
}

/// The staged (index) bytes of `relative`, as `git show :<path>` reports them.
pub(crate) fn index_bytes(root: &Path, relative: &str) -> Vec<u8> {
    let spec = format!(":{relative}");
    git_stdout(root, &["show", &spec])
}

/// The committed `HEAD` bytes of `relative`.
pub(crate) fn head_bytes(root: &Path, relative: &str) -> Vec<u8> {
    let spec = format!("HEAD:{relative}");
    git_stdout(root, &["show", &spec])
}

/// Put `relative` in the index as two unmerged stages, leaving the worktree
/// copy exactly as it was.
pub(crate) fn make_index_unmerged(root: &Path, relative: &str) {
    let spec = format!(":{relative}");
    let sha = String::from_utf8(git_stdout(root, &["rev-parse", &spec]))
        .expect("the blob id is UTF-8")
        .trim()
        .to_owned();

    git(root, &["update-index", "--force-remove", "--", relative]);
    let info = format!("100644 {sha} 1\t{relative}\n100644 {sha} 2\t{relative}\n");
    let output = git_with_stdin(root, &["update-index", "--index-info"], info.as_bytes());
    assert!(
        output.status.success(),
        "the index records two unmerged stages for {relative}: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let unmerged = git_output(root, &["show", &spec]);
    assert!(
        !unmerged.status.success(),
        "an unmerged {relative} cannot be read from the index: {}",
        String::from_utf8_lossy(&unmerged.stdout)
    );
}
