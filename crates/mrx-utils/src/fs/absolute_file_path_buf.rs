use std::{
    ops::Deref,
    path::{
        Path,
        PathBuf,
    },
};

use thiserror::Error;

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct AbsoluteFilePathBuf(PathBuf);

impl Deref for AbsoluteFilePathBuf {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AbsoluteFilePathBuf {
    fn new(buf: PathBuf) -> Self {
        Self(buf)
    }
}

#[derive(Debug, Error)]
pub enum AbsoluteFilePathBufError {
    #[error("Path not found")]
    NotFound(PathBuf),
    #[error("Path is not a file")]
    NotAFile(PathBuf),
    #[error("Io error: {0}")]
    Io(PathBuf, std::io::Error),
}

impl TryFrom<PathBuf> for AbsoluteFilePathBuf {
    type Error = AbsoluteFilePathBufError;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        std::fs::canonicalize(&path)
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound => AbsoluteFilePathBufError::NotFound(path.clone()),
                _ => AbsoluteFilePathBufError::Io(path.clone(), e),
            })
            .map(Self::new)
    }
}

impl AbsoluteFilePathBuf {
    /// # Errors
    /// TODO
    /// # Panics
    /// TODO
    pub fn try_from_relative(
        path: &Path,
        relative_to: PathBuf,
    ) -> Result<Self, AbsoluteFilePathBufError> {
        match (path.is_file(), path.is_absolute(), path.is_relative()) {
            (true, true, _) => Ok(Self::new(path.to_path_buf())),
            (true, _, true) => Self::try_from(path.to_path_buf()),
            (_, _, true) => {
                let relative_to_abs = Self::try_from(relative_to)?;
                let mut parent = relative_to_abs
                    .parent()
                    .expect("This should only fail when 'relative_to' is the filesystem root '/'");
                let (up_traversing, components): (Vec<_>, Vec<_>) = path
                    .components()
                    .partition(|s| s.as_os_str() == ".." || s.as_os_str() == ".");
                for _ in up_traversing.iter().filter(|s| s.as_os_str() != ".") {
                    parent = parent.parent().ok_or(AbsoluteFilePathBufError::NotFound(
                        parent.join("../").clone(),
                    ))?;
                }

                let mut path = PathBuf::new();
                path.extend(components);

                Ok(Self::new(parent.join(path)))
            }
            _ => Err(AbsoluteFilePathBufError::NotAFile(path.to_path_buf())),
        }
    }
}
