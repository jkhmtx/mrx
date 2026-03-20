use std::path::{
    Path,
    PathBuf,
};

mod absolute_path_buf;
mod write_with_rollback;
pub use absolute_path_buf::*;
pub use write_with_rollback::write_with_rollback;

#[must_use]
pub fn pathbuf_if_exists(path: &str) -> Option<PathBuf> {
    let path = PathBuf::from(path);

    if std::fs::exists(&path).ok().is_some_and(|exists| exists) {
        Some(path)
    } else {
        None
    }
}

/// # Errors
/// Errors if recursively creating the directory fails for a reason insular to [`std::io::Error`].
pub fn mk_dir(path: &Path) -> Result<(), std::io::Error> {
    std::fs::DirBuilder::new().recursive(true).create(path)
}

/// # Errors
/// Errors if removing the directory or creating it after removal (see [`mk_dir`]) fails.
pub fn recreate_dir(path: &Path) -> Result<(), std::io::Error> {
    match std::fs::remove_dir_all(path) {
        Ok(()) => mk_dir(path),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => mk_dir(path),
        Err(e) => Err(e),
    }
}

pub fn is_nix(path: impl AsRef<Path>) -> bool {
    path.as_ref()
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("nix"))
}
