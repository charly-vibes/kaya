//! File helpers shared by the publication build and its plan.
//!
//! Purpose: give every read, write, and rename one error shape.
//! Responsibilities: read text and bytes treating absence as absence, write
//! beside the target and rename into place, and render projection and io
//! failures as labeled publication refusals naming the file.
//! Rationale: the build writes only through one atomic path, so a reader of
//! the tree sees previous bytes or complete new ones — never a truncated page.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::projection::ProjectionError;

use super::error::PublicationError;
use super::vocabulary::LABEL_IO;

/// Read a page, treating an absent file as an absent document.
pub(super) fn read_text(path: &Path) -> Result<Option<String>, PublicationError> {
    match fs::read_to_string(path) {
        Ok(document) => Ok(Some(document)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(PublicationError::io("read", path, error)),
    }
}

/// Read a target byte-for-byte, treating an absent file as absent bytes.
pub(super) fn read_bytes(path: &Path) -> Result<Option<Vec<u8>>, PublicationError> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(PublicationError::io("read", path, error)),
    }
}

/// Write `bytes` beside `path` and rename them into place.
///
/// The rename is the only visible step: a reader of the tree sees the previous
/// bytes or the complete new ones, never a truncated file.
pub(super) fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), PublicationError> {
    let mut temporary = path.as_os_str().to_owned();
    temporary.push(".tmp");
    let temporary = PathBuf::from(temporary);
    fs::write(&temporary, bytes)
        .map_err(|error| PublicationError::io("write", &temporary, error))?;
    fs::rename(&temporary, path).map_err(|error| {
        PublicationError::new(
            LABEL_IO,
            format!(
                "failed to rename \"{}\" to \"{}\": {error}",
                temporary.display(),
                path.display()
            ),
        )
    })
}

/// Render a managed-block refusal as a publication refusal naming the file.
pub(super) fn compose_error(path: &Path, error: ProjectionError) -> PublicationError {
    PublicationError::new(
        LABEL_IO,
        format!(
            "failed to manage \"{}\": {}: {}",
            path.display(),
            error.label,
            error.message
        ),
    )
}
