use std::{
    fmt::Display,
    fs::{
        self,
        Metadata,
    },
    ops::Deref,
    path::{
        Path,
        PathBuf,
    },
};

use thiserror::Error;

use crate::attr::PathAttr;

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub enum AbsolutePathBuf {
    File(PathBuf),
    Directory(PathBuf),
}

impl Deref for AbsolutePathBuf {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        match &self {
            Self::File(p) | Self::Directory(p) => p,
        }
    }
}

#[derive(Debug, Error)]
pub enum AbsolutePathBufError {
    #[error("Path not found: {0}")]
    NotFound(PathBuf),
    #[error("Path is not a file or directory: {0}")]
    NotSupported(PathBuf),
    #[error("Io error: {0}")]
    Io(PathBuf, std::io::Error),
}

fn canonicalize(path: &Path) -> Result<PathBuf, AbsolutePathBufError> {
    fs::canonicalize(path).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => AbsolutePathBufError::NotFound(path.to_path_buf()),
        _ => AbsolutePathBufError::Io(path.to_path_buf(), e),
    })
}

fn metadata(path: &Path) -> Result<Metadata, AbsolutePathBufError> {
    path.metadata().map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => AbsolutePathBufError::NotFound(path.to_path_buf()),
        _ => AbsolutePathBufError::Io(path.to_path_buf(), e),
    })
}

impl TryFrom<&Path> for AbsolutePathBuf {
    type Error = AbsolutePathBufError;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let default_nix = path.join("default.nix");
        let path = if default_nix.is_file() {
            &default_nix
        } else {
            path
        };

        let metadata = metadata(path)?;

        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            canonicalize(path)?
        };

        Ok(Self::new(path, &metadata))
    }
}

impl TryFrom<&PathAttr> for AbsolutePathBuf {
    type Error = AbsolutePathBufError;

    fn try_from(value: &PathAttr) -> Result<Self, Self::Error> {
        AbsolutePathBuf::try_from(value.as_ref())
    }
}

impl AsRef<Path> for AbsolutePathBuf {
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}

impl AbsolutePathBuf {
    /// Creates a new [`AbsolutePathBuf`].
    #[allow(clippy::unnecessary_debug_formatting)]
    #[must_use]
    pub fn new(path: PathBuf, metadata: &Metadata) -> Self {
        debug_assert!(
            path.has_root(),
            "AbsolutePathBuf::new - must be an absolute path: {path:?}"
        );

        if metadata.is_file() {
            return Self::File(path);
        }

        debug_assert!(
            metadata.is_dir(),
            "AbsolutePathBuf::new - path must be a file or directory, on disk: {path:?}"
        );

        Self::Directory(path)
    }

    /// # Errors
    /// TODO
    /// # Panics
    /// TODO
    pub fn try_from_relative(
        path: &Path,
        relative_to: &Path,
    ) -> Result<Self, AbsolutePathBufError> {
        let relative_to_abs = Self::try_from(relative_to)?;
        let mut parent = relative_to_abs
            .parent()
            .expect("This should only fail when 'relative_to' is the filesystem root '/'");
        let (up_traversing, components): (Vec<_>, Vec<_>) = path
            .components()
            .partition(|s| s.as_os_str() == ".." || s.as_os_str() == ".");
        for _ in up_traversing.iter().filter(|s| s.as_os_str() != ".") {
            parent = parent
                .parent()
                .ok_or(AbsolutePathBufError::NotFound(parent.join("../").clone()))?;
        }

        let mut path = PathBuf::new();
        path.extend(components);
        let path = parent.join(path);

        Self::try_from(path.as_path())
    }

    /// # Errors
    /// Returns an error if [`parent`] is not a directory, or is not a parent of [`self`]
    pub fn as_relative_to_parent(&self, parent: &Path) -> Result<PathBuf, RelativeToParentError> {
        if parent.is_file() {
            return Err(RelativeToParentError::InvalidParent(format!(
                "path {} is not a directory",
                parent.to_string_lossy()
            )));
        }

        if !parent.exists() {
            return Err(RelativeToParentError::InvalidParent(format!(
                "path {} does not exist",
                parent.to_string_lossy()
            )));
        }

        let relative_to = if parent.is_absolute() {
            parent.to_path_buf()
        } else {
            canonicalize(parent).map_err(|e| match e {
                AbsolutePathBufError::Io(path_buf, error) => {
                    RelativeToParentError::Io(path_buf, error)
                }
                AbsolutePathBufError::NotFound(_) | AbsolutePathBufError::NotSupported(_) => {
                    unreachable!()
                }
            })?
        };

        let relative: Option<PathBuf> = {
            let mut path_components = self.components();
            let mut parent_components = relative_to.components();
            let suffix;
            loop {
                let mut iter_next = path_components.clone();

                let path_next = iter_next.next();
                let parent_next = parent_components.next();

                match (path_next, parent_next) {
                    (Some(ref x), Some(ref y)) if x == y => (),
                    // The directory iterator is exhausted,
                    // and we did not encounter break, which means:
                    // path_components contains the remainder after the common path
                    (Some(_), None) => {
                        suffix = Some(path_components);
                        break;
                    }
                    _ => {
                        suffix = None;
                        break;
                    }
                }
                path_components = iter_next;
            }

            suffix.map(|components| {
                let mut path = PathBuf::from("./");
                path.extend(components);

                path
            })
        };

        relative.ok_or_else(|| {
            RelativeToParentError::InvalidParent(format!(
                "'{}' is not a parent of '{}' ",
                parent.display(),
                self.display()
            ))
        })
    }
}

impl Display for AbsolutePathBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let display_str = self
            .as_path()
            .to_str()
            .ok_or_else(std::fmt::Error::default)?;

        f.write_str(display_str)
    }
}

#[derive(Debug, Error)]
pub enum RelativeToParentError {
    #[error("Invalid parent: {0}")]
    InvalidParent(String),
    #[error("Io error: {0}")]
    Io(PathBuf, std::io::Error),
}
