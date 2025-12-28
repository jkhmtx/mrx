use std::path::{
    Path,
    PathBuf,
};

mod absolute_file_path_buf;
mod write_with_fallback;
pub use absolute_file_path_buf::*;
pub use write_with_fallback::{
    WriteWithFallbackError,
    write_with_fallback,
};

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
/// TODO
pub fn mk_dir(path: &Path) -> Result<(), std::io::Error> {
    std::fs::DirBuilder::new().recursive(true).create(path)
}

/// # Errors
/// TODO
pub fn recreate_dir(path: &Path) -> Result<(), std::io::Error> {
    match std::fs::remove_dir_all(path) {
        Ok(()) => mk_dir(path),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => mk_dir(path),
        Err(e) => Err(e),
    }
}
