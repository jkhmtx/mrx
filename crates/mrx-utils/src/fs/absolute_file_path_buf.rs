use std::{
    fs,
    ops::Deref,
    path::{
        Path,
        PathBuf,
    },
};

use thiserror::Error;

use crate::attr::PathAttr;

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct AbsoluteFilePathBuf(PathBuf);

impl Deref for AbsoluteFilePathBuf {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
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

fn canonicalize(path: &Path) -> Result<PathBuf, AbsoluteFilePathBufError> {
    fs::canonicalize(path).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => AbsoluteFilePathBufError::NotFound(path.to_path_buf()),
        _ => AbsoluteFilePathBufError::Io(path.to_path_buf(), e),
    })
}

impl TryFrom<&Path> for AbsoluteFilePathBuf {
    type Error = AbsoluteFilePathBufError;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let default_nix = path.join("default.nix");
        let path = if default_nix.is_file() {
            &default_nix
        } else {
            path
        };

        canonicalize(path).map(Self)
    }
}

impl TryFrom<&PathAttr> for AbsoluteFilePathBuf {
    type Error = AbsoluteFilePathBufError;

    fn try_from(value: &PathAttr) -> Result<Self, Self::Error> {
        AbsoluteFilePathBuf::try_from(value.as_path())
    }
}

impl AbsoluteFilePathBuf {
    /// # Errors
    /// TODO
    /// # Panics
    /// TODO
    pub fn try_from_relative(
        path: &Path,
        relative_to: &Path,
    ) -> Result<Self, AbsoluteFilePathBufError> {
        match (path.is_file(), path.is_absolute()) {
            (true, true) => Ok(Self(path.to_path_buf())),
            (true, false) => Self::try_from(path),
            (false, false) => {
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
                let path = parent.join(path);

                Self::try_from(path.as_path())
            }
            (false, true) => Err(AbsoluteFilePathBufError::NotAFile(path.to_path_buf())),
        }
    }

    #[must_use]
    pub fn is_nix(&self) -> bool {
        self.extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("nix"))
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
                AbsoluteFilePathBufError::Io(path_buf, error) => {
                    RelativeToParentError::Io(path_buf, error)
                }
                AbsoluteFilePathBufError::NotFound(_) | AbsoluteFilePathBufError::NotAFile(_) => {
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
                parent.to_string_lossy(),
                self.as_path().to_string_lossy()
            ))
        })
    }
}

#[derive(Debug, Error)]
pub enum RelativeToParentError {
    #[error("Invalid parent: {0}")]
    InvalidParent(String),
    #[error("Io error: {0}")]
    Io(PathBuf, std::io::Error),
}
