use std::io::Write as _;
use std::path::Path;

/// Makes a tempfile A and writes [`bytes`] to it.
/// Makes a tempfile B and copies [`dest`] to it.
/// If copying A to [`dest`] fails, an attempt is made to copy B to [`dest`].
///
/// # Errors
/// Errors if tempfile creation fails, or if copying buffer contents from or to a tempfile fails.
/// A failure that initiates a rollback still returns an error, even when the rollback succeeds.
pub fn write_with_rollback(bytes: &[u8], dest: &Path) -> Result<(), std::io::Error> {
    use std::fs::copy;

    let mut a = tempfile::NamedTempFile::new()?;
    let b = tempfile::NamedTempFile::new()?;

    a.write_all(bytes)?;

    copy(dest, b.path())?;

    if let Err(e) = copy(a.path(), dest) {
        let _ = copy(b.path(), dest);

        Err(e)
    } else {
        Ok(())
    }
}
