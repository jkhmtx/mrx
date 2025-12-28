use std::io::Write as _;
use std::path::Path;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum WriteWithFallbackError {
    #[error("writing tempfile failed: `{0}`")]
    Failed(std::io::Error),
    #[error("invalid destination: `{0}`")]
    InvalidDest(std::io::Error),
    #[error("Rolled back: `${0}`")]
    RolledBack(std::io::Error),
}

type WriteWithFallbackResult<T> = Result<T, WriteWithFallbackError>;

/// Makes a tempfile A and writes [`bytes`] to it.
/// Makes a tempfile B and copies [`dest`] to it.
/// If copying A to [`dest`] fails, an attempt is made to copy B to [`dest`].
/// # Errors
/// TODO
pub fn write_with_fallback(bytes: &[u8], dest: &Path) -> WriteWithFallbackResult<()> {
    use std::fs::copy;

    let mut a = tempfile::NamedTempFile::new().map_err(WriteWithFallbackError::Failed)?;
    let b = tempfile::NamedTempFile::new().map_err(WriteWithFallbackError::Failed)?;

    a.write_all(bytes).map_err(WriteWithFallbackError::Failed)?;

    copy(dest, b.path()).map_err(WriteWithFallbackError::Failed)?;

    copy(a.path(), dest)
        .map_err(|e| match (e, copy(b.path(), dest)) {
            (e, Ok(_)) => WriteWithFallbackError::RolledBack(e),
            (e, Err(_)) => WriteWithFallbackError::Failed(e),
        })
        .map(|_| {})
}
